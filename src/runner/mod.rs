use tokio::sync::mpsc;

pub(super) use crate::error::Error;

type Tx<T = Vec<u8>> = mpsc::Sender<T>;
type Rx<T = Vec<u8>> = mpsc::Receiver<T>;

pub mod dispatcher;
#[allow(clippy::module_inception)]
pub mod runner;
pub mod status;
pub mod stream;

pub mod control;
pub mod writer;

#[doc(hidden)]
pub use crate::events::{Event, EventMapped};

pub mod abort;
