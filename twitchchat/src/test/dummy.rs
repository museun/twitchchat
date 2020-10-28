use futures::{Sink, Stream};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::decoder::{DecodeError, ReadMessage};

#[derive(Copy, Clone, Debug)]
pub struct Dummy;

impl Sink<String> for Dummy {
    type Error = DecodeError;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        unimplemented!()
    }
    fn start_send(self: Pin<&mut Self>, _: String) -> Result<(), Self::Error> {
        unimplemented!()
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        unimplemented!()
    }
    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        unimplemented!()
    }
}

impl ReadMessage for String {
    fn read_string(self) -> Result<String, DecodeError> {
        Ok(self)
    }
}

impl Stream for Dummy {
    type Item = Result<String, std::io::Error>;
    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unimplemented!()
    }
}
