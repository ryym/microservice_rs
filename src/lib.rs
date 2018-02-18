#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate url;

mod schema;
mod models;
mod db;
mod messages;

use hyper::{Chunk, StatusCode};
use hyper::Method::{Get, Post};
use hyper::server::{Request, Response, Service};
use hyper::header::{ContentLength, ContentType};
use futures::{future, Stream};
use futures::future::{Future, FutureResult};
use std::collections::HashMap;
use std::io;
use std::error::Error;
use models::{Message, NewMessage};
use messages::TimeRange;

pub struct Microservice;

impl Service for Microservice {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        // Establish DB connection for each request.
        let db_conn = match db::connect() {
            Some(conn) => conn,
            None => {
                return Box::new(future::ok(
                    Response::new().with_status(StatusCode::InternalServerError),
                ))
            }
        };

        match (request.method(), request.path()) {
            (&Post, "/") => {
                let future = request
                    .body()
                    .concat2() // `concat` is deprecated
                    .and_then(parse_form)
                    .and_then(move |msg| db::write_message(msg, &db_conn))
                    .then(make_post_response);
                Box::new(future)
            }
            (&Get, "/") => {
                let time_range = match request.query() {
                    Some(query) => parse_query(query),
                    None => Ok(TimeRange {
                        before: None,
                        after: None,
                    }),
                };
                let res = match time_range {
                    Ok(time_range) => make_get_response(db::query_messages(time_range)),
                    Err(error) => make_error_response(&error),
                };
                Box::new(res)
            }
            _ => Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound),
            )),
        }
    }
}

fn parse_form(form_chunk: Chunk) -> FutureResult<NewMessage, hyper::Error> {
    let mut form = url::form_urlencoded::parse(form_chunk.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();

    if let Some(message) = form.remove("message") {
        let username = form.remove("username").unwrap_or(String::from("anonymous"));
        future::ok(NewMessage { username, message })
    } else {
        future::err(hyper::Error::from(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing field 'message'",
        )))
    }
}

fn make_post_response(result: Result<i64, hyper::Error>) -> FutureResult<Response, hyper::Error> {
    match result {
        Ok(timestamp) => {
            let payload = json!({ "timestamp": timestamp }).to_string();
            let res = Response::new()
                .with_header(ContentLength(payload.len() as u64))
                .with_header(ContentType::json())
                .with_body(payload);
            debug!("{:?}", res);
            future::ok(res)
        }
        Err(error) => make_error_response(error.description()),
    }
}

fn make_get_response(msgs: Option<Vec<Message>>) -> FutureResult<Response, hyper::Error> {
    let res = match msgs {
        Some(msgs) => {
            let body = render_page(msgs);
            Response::new()
                .with_header(ContentLength(body.len() as u64))
                .with_body(body)
        }
        None => Response::new().with_status(StatusCode::InternalServerError),
    };
    debug!("{:?}", res);
    future::ok(res)
}

fn make_error_response(err_msg: &str) -> FutureResult<Response, hyper::Error> {
    let payload = json!({ "error": err_msg }).to_string();
    let res = Response::new()
        .with_status(StatusCode::InternalServerError)
        .with_header(ContentLength(payload.len() as u64))
        .with_header(ContentType::json())
        .with_body(payload);
    debug!("{:?}", res);
    future::ok(res)
}

fn parse_query(query: &str) -> Result<TimeRange, String> {
    let args = url::form_urlencoded::parse(&query.as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    // Maybe we can use error-chain to use more functional style.

    let before = args.get("before").map(|v| v.parse::<i64>());
    if let Some(ref result) = before {
        if let Err(ref error) = *result {
            return Err(format!("Error parsing 'before: {}", error));
        }
    }

    let after = args.get("after").map(|v| v.parse::<i64>());
    if let Some(ref result) = after {
        if let Err(ref error) = *result {
            return Err(format!("Error parsing 'after: {}", error));
        }
    }

    Ok(TimeRange {
        before: before.map(|b| b.unwrap()),
        after: after.map(|a| a.unwrap()),
    })
}

fn render_page(_messages: Vec<Message>) -> String {
    unimplemented!()
}
