use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use hyper;
use models::{Message, NewMessage};
use futures::future::{self, FutureResult};
use messages::TimeRange;
use std::error::Error;
use std::env;
use std::io;

const DATABASE_URL: &'static str = "postgresql://localhost:5432/microservice_rs";

pub fn connect() -> Option<PgConnection> {
    let db_url = env::var("DATABASE_URL").unwrap_or(DATABASE_URL.to_string());
    match PgConnection::establish(&db_url) {
        Ok(conn) => Some(conn),
        Err(err) => {
            error!("Error connecting to databsae: {}", err.description());
            None
        }
    }
}

pub fn write_message(
    new_msg: NewMessage,
    db_conn: &PgConnection,
) -> FutureResult<i64, hyper::Error> {
    use schema::messages;

    let timestamp = diesel::insert_into(messages::table)
        .values(&new_msg)
        .returning(messages::timestamp)
        .get_result(db_conn);

    match timestamp {
        Ok(timestamp) => future::ok(timestamp),
        Err(err) => {
            error!("Error writing to database: {}", err.description());
            future::err(hyper::Error::from(io::Error::new(
                io::ErrorKind::Other,
                "service error",
            )))
        }
    }
}

pub fn query_messages(time_range: TimeRange, db_conn: &PgConnection) -> Option<Vec<Message>> {
    use schema::messages;
    let TimeRange { before, after } = time_range;

    // Diesel does not currently provide an easy way to gradually
    // build up a uery.
    let result = match (before, after) {
        (Some(before), Some(after)) => messages::table
            .filter(messages::timestamp.lt(before))
            .filter(messages::timestamp.gt(after))
            .load(db_conn),
        (Some(before), _) => messages::table
            .filter(messages::timestamp.lt(before))
            .load(db_conn),
        (_, Some(after)) => messages::table
            .filter(messages::timestamp.gt(after))
            .load(db_conn),
        _ => messages::table.load(db_conn),
    };

    match result {
        Ok(result) => Some(result),
        Err(err) => {
            error!("Error querying DB: {}", err);
            None
        }
    }
}
