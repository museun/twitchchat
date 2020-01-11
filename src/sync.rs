use crate::*;

/// Encode the provided message to the [std::io::Write][Write]
///
/// [Write]: https://doc.rust-lang.org/std/io/trait.Write.html
pub fn encode<M: ?Sized, W: ?Sized>(message: &M, writer: &mut W) -> std::io::Result<()>
where
    M: Encodable,
    W: std::io::Write,
{
    message.encode(writer)?;
    writer.flush()
}

/// Write the provided `UserConfig` to the ***sync*** writer
///
/// # Example
/// ```rust
/// # use twitchchat::{UserConfig, sync::*};
/// let config = UserConfig::builder().anonymous().build().unwrap();
/// let mut writer = vec![];
/// register(&config, &mut writer).unwrap();
/// assert_eq!(
///     std::str::from_utf8(&writer).unwrap(),
///     "PASS justinfan1234\r\nNICK justinfan1234\r\n"
/// );
/// ```
pub fn register<W: ?Sized>(user_config: &UserConfig, writer: &mut W) -> std::io::Result<()>
where
    W: std::io::Write,
{
    encode(user_config, writer)
}

/// Opens a ***sync*** TCP connection using the provided `UserConfig` and `Secure` setting
pub fn connect(
    user_config: &UserConfig,
    secure: impl Into<Option<Secure>>,
) -> std::io::Result<(impl std::io::Read, impl std::io::Write)> {
    let addr = secure.into().unwrap_or_default().get_address();
    let mut stream = std::net::TcpStream::connect(addr)?;
    register(user_config, &mut stream)?;
    Ok((stream.try_clone().unwrap(), stream))
}

/// Opens a ***sync*** TCP connection using the provided `name`, `token and `Secure` setting
///
/// This enables all of the [Capabilities]
///
/// [Capabilities]: ./enum.Capability.html
pub fn connect_easy(
    name: &str,
    token: &str,
    secure: impl Into<Option<Secure>>,
) -> std::io::Result<(impl std::io::Read, impl std::io::Write)> {
    let config = simple_user_config(name, token).unwrap();
    connect(&config, secure)
}
