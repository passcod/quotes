use chrono::{DateTime, UTC};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use serde_json;
use std::env;
use std::sync::Arc;

table! {
    quotes {
        id -> Integer,
        authors -> Jsonb,
        body -> Text,
        created_at -> Timestamptz,
    }
}

pub type ArcConn = Arc<PgConnection>;
pub fn connect() -> ArcConn {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let conn = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    Arc::new(conn)
}

#[derive(Queryable)]
pub struct Quote {
    pub id: i32,
    pub authors: serde_json::Value,
    pub body: String,
    pub created_at: DateTime<UTC>,
}

#[derive(Insertable)]
#[table_name="quotes"]
struct NewQuote<'a> {
    pub authors: Vec<&'a str>,
    pub body: &'a str,
}

pub fn list(conn: &PgConnection) -> QueryResult<Vec<Quote>> {
    use self::quotes::dsl::*;

    quotes.load::<Quote>(conn)
}

// pub fn create(conn: &PgConnection, authors: Vec<String>, body: String) -> Quote {
//     let new_post = NewQuote {
//         title: title,
//         body: body,
//     };
// 
//     diesel::insert(&new_post).into(posts::table)
//         .get_result(conn)
//         .expect("Error saving new post")
// }
