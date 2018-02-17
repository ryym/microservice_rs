extern crate microservice;

extern crate env_logger;
extern crate hyper;
#[macro_use]
extern crate log;

use microservice::Microservice;

fn main() {
    env_logger::init();
    let address = "127.0.0.1:8080".parse().unwrap();

    // New Microservice instance is created
    // for each new request.
    let server = hyper::server::Http::new()
        .bind(&address, || Ok(Microservice {}))
        .unwrap();

    info!("Running microservice at {}", address);
    server.run().unwrap();
}
