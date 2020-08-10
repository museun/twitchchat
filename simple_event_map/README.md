# simple_event_map

This allows you to register types that'll be broadcast to receivers

### An example
```rust
#[derive(Clone, Debug, PartialEq)]
struct Message { data: String }

let mut map = EventMap::new();
// nothing is registered by default
assert_eq!(map.is_empty::<i32>(), true);
assert_eq!(map.is_empty::<String>(), true);
assert_eq!(map.is_empty::<Message>(), true);

// register two subscriptions for the message
// you can get a blocking iterator
let mut m1 = map.register_iter::<Message>();
// or you can get an async stream
let mut m2 = map.register_stream::<Message>();

let msg = Message{ data: String::from("hello world") };
// send the message, will return a bool if any messages were sent
assert_eq!(map.send(msg.clone()), true);
// we should have 2 still active
assert_eq!(map.active::<Message>(), 2);

assert_eq!(m1.next().unwrap(), msg);
// m2 is a stream, so we have to await it (and use StreamExt::next)
assert_eq!(m2.next().await.unwrap(), msg);

// drop a subscription (will be cleaned up in the eventmap on next send)
drop(m1);

let msg = Message{ data: String::from("testing") };
assert_eq!(map.send(msg.clone()), true);
// we only have 1 active now
assert_eq!(map.active::<Message>(), 1);
```

## License
`simple_event_map` is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
