use tokio::sync::mpsc;

pub(super) use crate::error::Error;

type Tx<T = Vec<u8>> = mpsc::Sender<T>;
type Rx<T = Vec<u8>> = mpsc::Receiver<T>;

pub(crate) mod dispatcher;
pub(crate) mod runner;
pub(crate) mod status;
pub(crate) mod stream;

pub(crate) mod control;
pub(crate) mod writer;

pub(crate) mod event;
#[doc(hidden)]
pub(crate) use event::{Event, EventMapped};

pub(crate) mod abort;
