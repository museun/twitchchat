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

#[cfg(test)]
mod tests {
    use crate::encode::*;

    #[test]
    fn encode_raw() {
        let mut out = vec![];
        super::encode(&raw("PRIVMSG #test :hello world"), &mut out).unwrap();
        assert_eq!(out, b"PRIVMSG #test :hello world\r\n");
    }

    #[test]
    fn encode_ping() {
        let mut out = vec![];
        super::encode(&ping("123456789"), &mut out).unwrap();
        assert_eq!(out, b"PING 123456789\r\n");
    }

    #[test]
    fn encode_pong() {
        let mut out = vec![];
        super::encode(&pong("123456789"), &mut out).unwrap();
        assert_eq!(out, b"PONG :123456789\r\n");
    }

    #[test]
    fn encode_join() {
        let mut out = vec![];
        super::encode(&join("#museun"), &mut out).unwrap();
        assert_eq!(out, b"JOIN #museun\r\n");
    }

    #[test]
    fn encode_part() {
        let mut out = vec![];
        super::encode(&part("#museun"), &mut out).unwrap();
        assert_eq!(out, b"PART #museun\r\n");
    }

    #[test]
    fn encode_privmsg() {
        let mut out = vec![];
        super::encode(&privmsg("#museun", "this is a test of a line"), &mut out).unwrap();
        assert_eq!(
            out,
            "PRIVMSG #museun :this is a test of a line\r\n".as_bytes()
        );

        let mut out = vec![];
        super::encode(&privmsg("#museun", &"foo ".repeat(500)), &mut out).unwrap();
        assert_eq!(
            out,
            format!("PRIVMSG #museun :{}\r\n", &"foo ".repeat(500)).as_bytes()
        );
    }
}
