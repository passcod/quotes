use chrono::{DateTime, UTC};
use diesel;
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

#[derive(Queryable, Serialize)]
pub struct Quote {
    pub id: i32,
    pub authors: serde_json::Value,
    pub body: String,
    pub created_at: DateTime<UTC>,
}

#[derive(Insertable)]
#[table_name="quotes"]
struct NewQuote<'a> {
    pub authors: serde_json::Value,
    pub body: &'a str,
}

pub fn list(conn: &PgConnection) -> QueryResult<Vec<Quote>> {
    use self::quotes::dsl::*;

    quotes.load::<Quote>(conn)
}

pub fn get(conn: &PgConnection, qid: i32) -> QueryResult<Quote> {
    use self::quotes::dsl::*;

    quotes.find(qid).first::<Quote>(conn)
}

pub fn create(conn: &PgConnection, authors: Vec<String>, body: String) -> Result<Quote, String> {
    let new_quote = NewQuote {
        authors: serde_json::to_value(authors)
            .map_err(|e| format!("Error saving new quote: {}", e))?,
        body: &body,
    };

    diesel::insert(&new_quote).into(quotes::table)
        .get_result(conn)
        .map_err(|e| format!("Error saving new quote: {}", e))
}
