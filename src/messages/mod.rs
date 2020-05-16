/*!
Messages sent by the server

This can be obtained by [subscribing] to an [Event] on a [Dispatcher]

Or by using [Parse] on an [Message]

[subscribing]: ../struct.Dispatcher.html#method.subscribe
[Event]: ../events/index.html
[Dispatcher]: ../struct.Dispatcher.html
[Parse]: ../trait.Parse.html
[Message]: ../decode/struct.Message.html
*/

mod error;
pub use error::InvalidMessage;

mod expect;

#[allow(clippy::module_inception)]
mod messages;

#[doc(inline)]
pub use self::messages::*;

#[cfg(test)]
mod tests;
