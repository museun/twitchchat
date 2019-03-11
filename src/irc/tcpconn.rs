use super::UserConfig;
use crate::twitch::{Client, Error};

use std::net::TcpStream;

/// A simple Read/Write provider that uses a [TcpStream](std::net::TcpStream) (allowing for a TLS TcpStream)
pub struct TcpConn;

// TODO TLS feature, use https://github.com/sfackler/rust-native-tls
// irc://irc.chat.twitch.tv:6697 (TLS)
// irc://irc.chat.twitch.tv:6667
// TODO connect_timeout
impl TcpConn {
    /// Connects to the non-TLS Twitch irc endpoint and registering with  [UserConfig](UserConfig)
    ///
    /// This will block until the connection has successfully been completed, yielding the actual username    
    pub fn connect(config: &UserConfig) -> Result<Client<TcpStream, TcpStream>, Error> {
        const ADDRESS: &str = "irc.chat.twitch.tv:6667";

        let stream = TcpStream::connect(&ADDRESS).map_err(Error::Connect)?;
        let read = stream
            .try_clone()
            .expect("must be able to clone the tcp stream");
        let mut client = Client::new(read, stream);

        client
            .write_line("CAP REQ :twitch.tv/membership")
            .and_then(|_| client.write_line("CAP REQ :twitch.tv/commands"))
            .and_then(|_| client.write_line("CAP REQ :twitch.tv/tags"))
            .and_then(|_| client.write_line(&format!("PASS {}", &config.token)))
            .and_then(|_| client.write_line(&format!("NICK {}", &config.nick)))
            .map_err(Box::new) // what
            .map_err(Error::Register)?;

        Ok(client)
    }
}



