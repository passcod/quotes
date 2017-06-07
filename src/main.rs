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
extern crate serde_json;

use diesel::QueryResult;
use rocket::request::Form;
use rocket::response::Redirect;

mod db;
mod markdown;

#[get("/")]
fn index() -> QueryResult<String> {
    let conn = db::connect();
    Ok(db::list(&conn)?.iter().map(|quote| {
        format!("{}", quote.body)
    }).collect())
}

#[derive(FromForm)]
struct NewQuote {
    authors: String,
    body: String,
}

#[post("/", data = "<form>")]
fn create(form: Form<NewQuote>) -> () {
    let quote = form.get();

    let body = markdown::render(&quote.body);
    let authors = &quote.authors
        .split(',')
        .map(|a| markdown::render(a))
        .collect::<Vec<String>>();

    let conn = db::connect();
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, create])
        .launch();
}
