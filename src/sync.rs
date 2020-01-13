use crate::*;
use std::io::{Read, Write};

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
pub fn register<W: ?Sized>(user_config: &UserConfig, writer: &mut W) -> std::io::Result<()>
where
    W: Write,
{
    let UserConfig {
        name,
        token,
        capabilities,
    } = user_config;

    for cap in capabilities {
        writer.write_all(cap.encode_as_str().as_bytes())?;
        writer.write_all(b"\r\n")?;
    }

    writer.write_all(format!("PASS {}\r\n", token).as_bytes())?;
    writer.write_all(format!("NICK {}\r\n", name).as_bytes())?;

    Ok(())
}

/**
Opens a ***sync*** TCP connection using the provided `UserConfig`

# Note
This doesn't support TLS because TlsStream isn't clonable.

Use the [async version][async] if you want a TLS wrapped connection

# Example
```rust
# use twitchchat::{sync::*, UserConfig};
let user_config = UserConfig::builder().anonymous().build().unwrap();
let (read, write) = connect(&user_config).unwrap();
```

[async]: ../fn.connect.html
*/
pub fn connect(user_config: &UserConfig) -> std::io::Result<(impl Read, impl Write)> {
    let mut stream = std::net::TcpStream::connect(TWITCH_IRC_ADDRESS)?;
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
```rust
# use twitchchat::{sync::*, ANONYMOUS_LOGIN};
let (nick, pass) = ANONYMOUS_LOGIN;
let (read, write) = connect_easy(&nick, &pass).unwrap();
```

[Capabilities]: ./enum.Capability.html
[async]: ../fn.connect_easy.html
*/

pub fn connect_easy(name: &str, token: &str) -> std::io::Result<(impl Read, impl Write)> {
    let config = simple_user_config(name, token).unwrap();
    connect(&config)
}
