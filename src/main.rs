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
use rocket::request::Form;
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::Template;
use std::collections::HashMap;
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
    rocket::ignite()
        .mount("/", routes![
            assets,
            create,
            index,
            new,
            quote,
        ])
        .launch();
}
