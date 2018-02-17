extern crate env_logger;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;

use hyper::server::{Request, Response, Service};
use futures::future;
use futures::future::Future;

pub struct Microservice;

impl Service for Microservice {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Request) -> Self::Future {
        info!("Microservice receive a request: {:?}", request);
        Box::new(future::ok(Response::new()))
    }
}
