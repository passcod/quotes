#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate crowbook_text_processing;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate dotenv;
#[macro_use]
extern crate lazy_static;
extern crate pulldown_cmark;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use diesel::QueryResult;
use rocket::config::{Config, Environment};
use rocket::request::Form;
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::Template;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

mod db;
mod markdown;

#[get("/")]
fn index() -> QueryResult<Template> {
    let conn = db::connect();
    let mut context: HashMap<&str, Vec<HashMap<&str, String>>> = HashMap::new();
    context.insert("quotes", db::list(&conn)?
        .iter()
        .map(|q| {
            let mut quote = HashMap::new();
            quote.insert("id", format!("{}", q.id));
            quote.insert("authors", format!("{}", q.authors
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|a| format!("{}", a.as_str().unwrap_or("")))
                .collect::<Vec<String>>()
                .join(", ")
            ));
            quote.insert("body", format!("{}", q.body));
            quote.insert("created_at", format!("{}", q.created_at));
            quote
        })
        .collect()
    );
    Ok(Template::render("index", &context))
}

#[get("/<id>")]
fn quote(id: i32) -> QueryResult<Template> {
    let conn = db::connect();
    let q = db::get(&conn, id)?;

    let mut quote: HashMap<&str, String> = HashMap::new();
    quote.insert("id", format!("{}", q.id));
    quote.insert("authors", format!("{}", q.authors
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|a| format!("{}", a.as_str().unwrap_or("")))
        .collect::<Vec<String>>()
        .join(", ")
    ));
    quote.insert("body", format!("{}", q.body));
    quote.insert("created_at", format!("{}", q.created_at));

    Ok(Template::render("quote", &quote))
}

#[get("/assets/<file..>")]
fn assets(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("assets/").join(file)).ok()
}

#[derive(FromForm)]
struct NewQuote {
    authors: String,
    body: String,
}

#[get("/new")]
fn new() -> Template {
  Template::render("new", &())
}

#[post("/new", data = "<form>")]
fn create(form: Form<NewQuote>) -> Result<Redirect, String> {
    let quote = form.get();

    let body = markdown::render(&quote.body);
    let authors = &quote.authors
        .split(',')
        .map(|a| markdown::render(a))
        .collect::<Vec<String>>();

    let conn = db::connect();
    db::create(&conn, authors.to_vec(), body)?;
    Ok(Redirect::to("/"))
}

fn main() {
    let webenv = env::var("WEB_ENV")
        .unwrap_or("development".into());

    let port = env::var("PORT")
        .map_err(|e| format!("{}", e))
        .and_then(|p|
            u16::from_str_radix(&p, 10)
            .map_err(|e| format!("{}", e))
            .map(|p| p.into())
        )
        .unwrap_or(8000);

    let config = Config::build(match webenv.as_str() {
            "production" => Environment::Production,
            "staging" => Environment::Staging,
            _ => Environment::Development,
        })
        .address("0.0.0.0")
        .port(port)
        .finalize()
        .expect("Error configuring server");

    rocket::custom(config, true)
        .mount("/", routes![
            assets,
            create,
            index,
            new,
            quote,
        ])
        .launch();
}
