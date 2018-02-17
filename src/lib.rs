extern crate env_logger;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;
extern crate url;

use hyper::{Chunk, StatusCode};
use hyper::Method::Post;
use hyper::server::{Request, Response, Service};
use hyper::header::{ContentLength, ContentType};
use futures::{future, Stream};
use futures::future::{Future, FutureResult};
use std::collections::HashMap;
use std::io;
use std::error::Error;

pub struct Microservice;

impl Service for Microservice {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        match (request.method(), request.path()) {
            (&Post, "/") => {
                let future = request
                    .body()
                    .concat2() // `concat` is deprecated
                    .and_then(parse_form)
                    .and_then(write_to_db)
                    .then(make_post_response);
                Box::new(future)
            }
            _ => Box::new(future::ok(
                Response::new().with_status(StatusCode::NotFound),
            )),
        }
    }
}

struct NewMessage {
    username: String,
    message: String,
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

fn write_to_db(_entry: NewMessage) -> FutureResult<i64, hyper::Error> {
    future::ok(0) // TODO
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
