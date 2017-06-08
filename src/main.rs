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
extern crate serde_json;

use diesel::QueryResult;
use rocket::request::Form;
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::Template;
use std::path::{Path, PathBuf};

mod db;
mod markdown;

#[get("/")]
fn index() -> QueryResult<String> {
    let conn = db::connect();
    Ok(db::list(&conn)?.iter().map(|quote| {
        format!("{}", quote.body)
    }).collect())
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
        .mount("/", routes![index, assets, new, create])
        .launch();
}
