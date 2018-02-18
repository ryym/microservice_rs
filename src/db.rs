use hyper;
use models::{Message, NewMessage};
use futures::future::{self, FutureResult};
use messages::TimeRange;

pub fn write_message(_entry: NewMessage) -> FutureResult<i64, hyper::Error> {
    future::ok(0) // TODO
}

pub fn query_messages(_time_range: TimeRange) -> Option<Vec<Message>> {
    unimplemented!()
}
