# twitchchat
[![Documentation][docs_badge]][docs]
[![Crates][crates_badge]][crates]
[![Actions][actions_badge]][actions]

This crate provides a way to interact with [Twitch]'s chat.

Along with parse messages as Rust types, it provides methods for sending messages.

It also provides an 'event' loop which you can use to make a bot.


## Runtime
This crate is runtime agonostic. To use..

| Read/Write provider                                        | Features                |
| ---                                                        | ---                     |
| [`async_io`](https://docs.rs/async-io/latest/async_io/)    |`async-io`               |
| [`smol`](https://docs.rs/smol/latest/smol/)                |`smol`                   |
| [`async_std`](https://docs.rs/async-std/latest/async_std/) |`async-std`              |
| [`tokio`](https://docs.rs/tokio/latest/tokio/)             |`tokio` and `tokio-util` |
### TLS
If you want TLS supports, this crate currently supports using various [`rustls`](https://docs.rs/rustls/latest/rustls/) wrappers.

Enable the above runtime and also enable the cooresponding features:
| Read/Write provider                                        | Runtime     | Features                                        |
| ---                                                        | ---         | ---                                             |
| [`async_io`](https://docs.rs/async-io/latest/async_io/)    | `async_io`  | `async-tls`                                     |
| [`smol`](https://docs.rs/smol/latest/smol/)                | `smol`      | `async-tls`                                     |
| [`async_std`](https://docs.rs/async-std/latest/async_std/) | `async_std` | `async-tls`                                     |
| [`tokio`](https://docs.rs/tokio/latest/tokio/)             | `tokio`     | `tokio-util`, `tokio-rustls` and `webpki-roots` |

## Serde support
To enable serde support, simply enable the optional `serde` feature


## Examples
#### Using async_io to connect with.. 
* [async_io_demo.rs](./examples/async_io_demo.rs)

#### Using async_std to connect with..
* [async_std_demo.rs](./examples/async_std_demo.rs)


#### Using smol to connect with..
* [smol_demo.rs](./examples/smol_demo.rs)

#### Using tokio to connect with..
* [tokio_demo.rs](./examples/tokio_demo.rs)


#### How to use the crate as just a message parser(decoder)/encoder
* [message_parse.rs](./examples/message_parse.rs)

#### An a simple example of how one could built a bot with this
* [simple_bot.rs](./examples/simple_bot.rs)



## License
`twitchchat` is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See `LICENSE-APACHE` and `LICENSE-MIT` for details.

[docs_badge]: https://docs.rs/twitchchat/badge.svg
[docs]: https://docs.rs/twitchchat
[crates_badge]: https://img.shields.io/crates/v/twitchchat.svg
[crates]: https://crates.io/crates/twitchchat
[actions_badge]: https://github.com/museun/twitchchat/workflows/Rust/badge.svg
[actions]: https://github.com/museun/twitchchat/actions

[Twitch]: https://dev.twitch.tv
