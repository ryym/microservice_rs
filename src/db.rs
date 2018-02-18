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

pub fn query_messages(_time_range: TimeRange) -> Option<Vec<Message>> {
    unimplemented!()
}
