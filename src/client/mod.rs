/*!
This contains types for reading/writing messages to Twitch.

# Event handling.
You'll create a dispatcher which'll let you subscribe and unsubscribe for specific events.

The dispatcher is clonable and the `Runner` requires a clone of it to filter the events for you.

### Example
```rust
# use twitchchat::{Dispatcher, Runner, RateLimit, events, messages::AllCommands};
# use tokio::spawn;
# use futures::stream::StreamExt as _;
# let conn = tokio_test::io::Builder::new().read(b"PING :123456789\r\n").write(b"PONG :123456789\r\n").build();
# let fut = async move {
let dispatcher = Dispatcher::new();
let mut all = dispatcher.subscribe::<events::All>();

// give our a clone of our dispatcher to the runner
let (runner, control) = Runner::new(dispatcher.clone(), RateLimit::default());

// loop over our stream
spawn(async move {
    while let Some(msg) = all.next().await {
        // its an Arc, so you can just deref/reborrow it
        match &*msg {
            AllCommands::Unknown(raw) => {},
            AllCommands::Ping(ping) => {},
            _ => { break; /* we can end the task by returning from it */ }
        }
    }

    // when the task ends/drops, the EventStream will unsubscribe itself
});

// consume the runner, driving any streams we subscribe to
// (and any writes from the 'Control')
runner.run(conn).await.unwrap();
# };
# tokio::runtime::Runtime::new().unwrap().block_on(fut);
```

### Using the dispatcher
You can [subscribe] to any number of [events][events].

```rust
# use twitchchat::{Dispatcher, events};
let dispatcher = Dispatcher::new();
// subscribe to a specific event
let join = dispatcher.subscribe::<events::Join>();
// you can do this as many times as you want
let join2 = dispatcher.subscribe::<events::Join>();
// there is a catch-all event type that is an enum of all possible events
let all = dispatcher.subscribe::<events::All>();
```

These events are [Streams][stream] which produce an one of the correlated [messages][messages].

```rust
# use twitchchat::{Dispatcher, events, messages};
# use tokio::stream::StreamExt as _;
# use futures::stream::StreamExt as _;
# async move {
let dispatcher = Dispatcher::new();

// subscribe to a specific event
let join = dispatcher.subscribe::<events::Join>();

// join is a Stream<Item = Arc<messages::Join<'static>>>
let fut = join.for_each(|item| async move {
    println!("{:?}", item)
});
# };
```

## Unsubscribing from events
Dropping this [EventStream] will unsubscribe from the events.

When the dispatcher is dropped, all of the streams will produce ***None***.

You can also clear subscriptions to cause `EventStreams` to produce `None` on their next resolvement

### Clearing subscriptions via the dispatcher
You can clear event subscriptions for [specific events][specific] or for [all events][all]

[Dispatcher]: ./struct.Dispatcher.html
[EventStream]: ./struct.EventStream.html
[subscribe]: ./struct.Dispatcher.html#method.subscribe
[messages]: ../messages/index.html
[events]: ../events/index.html
[specific]: ./struct.Dispatcher.html#method.clear_subscriptions
[all]: ./struct.Dispatcher.html#method.clear_subscriptions_all

[Stream]: https://docs.rs/tokio/0.2/tokio/stream/trait.Stream.html
*/

use futures::stream::*;
use tokio::prelude::*;
use tokio::sync::mpsc;

type Tx<T = Vec<u8>> = mpsc::Sender<T>;
type Rx<T = Vec<u8>> = mpsc::Receiver<T>;

mod status;
pub use status::Status;

mod runner;
pub use runner::Runner;

mod dispatcher;
pub use dispatcher::Dispatcher;

mod stream;
pub use stream::EventStream;

use crate::error::Error;

mod writer;
pub use writer::Writer;

mod control;
pub use control::Control;

mod event;
#[doc(hidden)]
pub use event::{Event, EventMapped};

mod abort;
