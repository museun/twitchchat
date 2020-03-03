/*! # Synchronous methods

This is just provided so you can open a `std::net::TcpStream` connection

And so you have a `register()` method that works with just `std::io::Write`

You probably would want to use the async `connect/register` methods from the crate root.
*/

use crate::{simple_user_config, UserConfig, TWITCH_IRC_ADDRESS};

use std::io::Write;
use std::net::TcpStream;

/**
Write the provided `UserConfig` to the ***sync*** writer
# Example
```rust
# use twitchchat::{UserConfig, sync::*};
let config = UserConfig::builder().anonymous().build().unwrap();

let mut writer = vec![];
register(&config, &mut writer).unwrap();

assert_eq!(
    std::str::from_utf8(&writer).unwrap(),
    "PASS justinfan1234\r\nNICK justinfan1234\r\n"
);
```
*/
pub fn register<W: ?Sized + Write>(
    user_config: &UserConfig,
    writer: &mut W,
) -> std::io::Result<()> {
    let UserConfig {
        name,
        token,
        capabilities,
    } = user_config;

    for cap in capabilities {
        writer.write_all(cap.encode_as_str().as_bytes())?;
        writer.write_all(b"\r\n")?;
    }

    let data = format!("PASS {}\r\nNICK {}\r\n", token, name);
    writer.write_all(data.as_bytes())?;
    writer.flush()
}

/**
Opens a ***sync*** TCP connection using the provided `UserConfig`

# Note
This doesn't support TLS because ***TlsStream*** isn't clonable.

Use the [async version][async] if you want a TLS wrapped connection

# Example
```rust,no_run
# use twitchchat::{sync::*, UserConfig};
let user_config = UserConfig::builder().anonymous().build().unwrap();
let (read, write) = connect(&user_config).unwrap();
```

[async]: ../fn.connect.html
*/
pub fn connect(user_config: &UserConfig) -> std::io::Result<(TcpStream, TcpStream)> {
    let mut stream = TcpStream::connect(TWITCH_IRC_ADDRESS)?;
    register(user_config, &mut stream)?;
    Ok((stream.try_clone().unwrap(), stream))
}

/**
Opens a ***sync*** TCP connection using the provided `name` and `token`

This enables all of the [Capabilities]

# Note
This doesn't support TLS because TlsStream isn't clonable.

Use the [async version][async] if you want a TLS wrapped connection

# Example
```rust,no_run
# use twitchchat::{sync::*, ANONYMOUS_LOGIN};
let (nick, pass) = ANONYMOUS_LOGIN;
let (read, write) = connect_easy(&nick, &pass).unwrap();
```

[Capabilities]: ../enum.Capability.html
[async]: ../fn.connect_easy.html
*/
pub fn connect_easy(name: &str, token: &str) -> std::io::Result<(TcpStream, TcpStream)> {
    let config = simple_user_config(name, token).unwrap();
    connect(&config)
}

// TODO: add a SyncClient if we want to keep this

#[doc(inline)]
pub use crate::encode::Encoder;
