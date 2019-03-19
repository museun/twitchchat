use log::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::Arc;

use super::filter::{FilterMap, MessageFilter, Token};
use super::{commands, Capability, Error, LocalUser, Message, MutexWrapper as Mutex, Writer};

use crate::irc::types::Message as IrcMessage;
use crate::UserConfig;

// 20 per 30 seconds	Users sending commands or messages to channels in which they do not have Moderator or Operator status
// 100 per 30 seconds	Users sending commands or messages to channels in which they have Moderator or Operator status

/// Client for interacting with Twitch's chat.
///
/// It wraps a [Read](https://doc.rust-lang.org/std/io/trait.Read.html),
/// [Write](https://doc.rust-lang.org/std/io/trait.Write.html) pair
///
/// ```no_run
/// use twitchchat::{helpers::TestStream, Client};
/// let stream = TestStream::new();
/// let (r,w) = (stream.clone(), stream.clone());
/// let mut client = Client::new(r,w); // moves the r,w
/// // register, join, on, etc
/// client.run().unwrap();
/// ```
// TODO write usage
pub struct Client<R, W> {
    reader: BufReader<R>,
    filters: FilterMap<W>,
    writer: Writer<W>,
}

impl<R: Read, W: Write> Client<R, W> {
    /// Create a new Client from a
    /// [Read](https://doc.rust-lang.org/std/io/trait.Read.html),
    /// [Write](https://doc.rust-lang.org/std/io/trait.Write.html) pair
    ///
    /// This client is clonable, and thread safe.
    pub fn new(read: R, write: W) -> Self {
        Self {
            reader: BufReader::new(read),
            filters: FilterMap::default(),
            writer: Writer(Arc::new(Mutex::new(write))),
        }
    }

    /// Runs, consuming all messages.
    ///
    /// This also pumping them through
    /// [`Client::on`](./struct.Client.html#method.on) filters
    pub fn run(mut self) -> Result<(), Error> {
        loop {
            match self.read_message() {
                Ok(..) => (),
                Err(Error::InvalidMessage(msg)) => {
                    warn!("read an invalid message: {}", msg);
                    continue;
                }
                Err(err) => return Err(err),
            }
        }
    }

    /// Registers with the server uses the provided [`UserConfig`](./struct.UserConfig.html)
    ///
    /// This is a **very** useful step, after you make the client and set up your initial filters
    ///
    /// You should call this to send your `OAuth token` and `Nickname`
    ///
    /// This also sends the [`Capabilities`](./enum.Capability.html) in the correct order
    ///
    /// Usage
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, Client, UserConfig};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = (stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// let config = UserConfig::builder()
    ///                 .token(std::env::var("MY_PASSWORD").unwrap())
    ///                 .nick("museun")
    ///                 .build()
    ///                 .unwrap();
    /// client.register(config).unwrap();
    /// // we should be connected now
    /// // this'll block until everything is read
    /// let _ = client.wait_for_ready().unwrap();
    /// ```
    pub fn register<U>(&mut self, config: U) -> Result<(), Error>
    where
        U: std::borrow::Borrow<UserConfig>,
    {
        let config = config.borrow();
        for cap in config.caps.iter().filter_map(|c| c.get_command()) {
            self.writer.write_line(cap)?;
        }

        self.writer.write_line(&format!("PASS {}", config.token))?;
        self.writer.write_line(&format!("NICK {}", config.nick))
    }

    /// Waits for the
    /// [`GLOBALUSERSTATE`](./commands/struct.GlobalUserState.html) before
    /// continuing, discarding any messages received
    ///
    /// Returns some [`useful information`](./struct.LocalUser.html) about your user
    ///
    /// This blocks until the twitch registration is completed, this relies on
    /// the [`Tags Capability`](./enum.Capability.html#variant.Tags) being sent.
    ///
    /// Usage:
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, Client};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = (stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// match client.wait_for_ready() {
    ///     Ok(user) => println!("user id: {}", user.user_id),
    ///     Err(err) => panic!("failed to finish registration: {}", err)
    /// };
    /// // we can be sure that we're ready to join
    /// client.writer().join("some_channel").unwrap();
    /// ```
    pub fn wait_for_ready(&mut self) -> Result<LocalUser, Error> {
        use crate::irc::types::Message as IRCMessage;
        let mut caps = vec![];

        loop {
            match self.read_message()? {
                Message::Irc(IRCMessage::Cap {
                    acknowledge: true,
                    cap,
                }) => match cap.as_str() {
                    "twitch.tv/tags" => caps.push(Capability::Tags),
                    "twitch.tv/membership" => caps.push(Capability::Membership),
                    "twitch.tv/commands" => caps.push(Capability::Commands),
                    _ => {}
                },

                Message::Irc(IRCMessage::Ready { .. }) => {
                    let mut bad = vec![];
                    match (
                        caps.contains(&Capability::Tags),
                        caps.contains(&Capability::Commands),
                    ) {
                        (true, true) => continue,

                        (false, true) => bad.push(Capability::Tags),
                        (true, false) => bad.push(Capability::Commands),
                        _ => {
                            bad.push(Capability::Tags);
                            bad.push(Capability::Commands);
                        }
                    };

                    if !bad.is_empty() {
                        return Err(Error::CapabilityRequired(bad));
                    }
                }

                Message::GlobalUserState(state) => {
                    return Ok(LocalUser {
                        user_id: state.user_id(),
                        display_name: state.display_name().map(ToString::to_string),
                        color: state.color(),
                        badges: state.badges(),
                        emote_sets: state.emote_sets(),
                        caps,
                    });
                }
                _ => continue,
            }
        }
    }

    /// Like [`wait_for_ready`](./struct.Client.html#method.wait_for_ready) but waits for the end of the IRC MOTD
    ///
    /// This will generally happen before `GLOBALUSERSTATE` but don't rely on that
    ///
    /// Returns the username assigned to you by the server
    ///
    /// Usage:
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, Client};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = (stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// match client.wait_for_irc_ready() {
    ///     Ok(name) => println!("end of motd, our name is: {}", name),
    ///     Err(err) => panic!("failed to finish registration: {}", err),
    /// };
    /// // we can be sure that we're ready to join
    /// client.writer().join("some_channel").unwrap();
    /// ```
    pub fn wait_for_irc_ready(&mut self) -> Result<String, Error> {
        use crate::irc::types::Message as IrcMessage;
        loop {
            match self.read_message()? {
                Message::Irc(IrcMessage::Ready { name }) => return Ok(name),
                _ => continue,
            }
        }
    }

    /// Reads a [`Message`](./enum.Message.html#variants)
    ///
    /// This will automatically handle some *tedious* messages, like the _heartbeat_ (PING)
    ///
    /// This also 'pumps' the messages through the filter
    ///
    /// Using this will drive the client (blocking for a read, then producing messages).
    /// Usage:
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, Client};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = (stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// // block the thread (i.e. wait for the client to close down)    
    /// while let Ok(msg) = client.read_message() {
    ///     // match msg {
    ///     // .. stuff
    ///     // }
    /// }
    ///
    /// // or incrementally calling `client.read_message()`
    /// // when you want the next message
    /// ```
    pub fn read_message(&mut self) -> Result<Message, Error> {
        // TODO provide an internal buffer to prevent this dumb allocation
        // using https://docs.rs/bytes/0.4.11/bytes/

        let mut buf = String::new();
        let len = self.reader.read_line(&mut buf).map_err(Error::Read)?;
        // 0 == EOF
        if len == 0 {
            return Err(Error::CannotRead);
        }

        let buf = buf.trim_end();
        if buf.is_empty() {
            return Err(Error::CannotRead);
        }

        trace!("trying to parse message");
        let msg = IrcMessage::parse(&buf) //
            .ok_or_else(|| Error::InvalidMessage(buf.to_string()))?;

        // handle PINGs automatically
        if let IrcMessage::Ping { token } = &msg {
            self.writer.write_line(&format!("PONG :{}", token))?;
        }

        // sanity check, doing it here instead of after its been re-parsed to fail early
        if let IrcMessage::Unknown {
            prefix,
            head,
            args,
            tail,
            ..
        } = &msg
        {
            if let (Some(crate::irc::types::Prefix::Server { host }), Some(data)) = (prefix, tail) {
                if head == "NOTICE"
                    && host == "tmi.twitch.tv"
                    && data == "Improperly formatted auth"
                    // excellent
                    && args.get(0) == Some(&"*".into())
                {
                    trace!("got a registartion error");
                    return Err(Error::InvalidRegistration);
                }
            }
        }

        let msg = commands::parse(&msg).unwrap_or_else(|| Message::Irc(msg));
        trace!("<- {:?}", msg);
        {
            let w = self.writer();
            let key = msg.what_filter();
            if let Some(filters) = self.filters.get_mut(key) {
                for filter in filters.iter_mut() {
                    trace!("sending msg to filter (id: {}): {:?}", (filter.1).0, key);
                    (filter.0)(msg.clone(), w.clone()) // when in doubt
                }
            }
        }

        trace!("read a message");
        Ok(msg)
    }
}

impl<R, W: Write> Client<R, W> {
    /**
    When a message is received run this function with it and a clone of the Writer.

    The type of the closure determines what is filtered

    Usage:
    ```no_run
    # use twitchchat::{helpers::TestStream, Client, Writer};
    # let mut stream = TestStream::new();
    # let (r, w) = (stream.clone(), stream.clone());
    # let mut client = Client::new(r, w);
    use twitchchat::commands::*;
    let pm_tok = client.on(|msg: PrivMsg, w: Writer<_>| {
        // msg is now a `twitchchat::commands::PrivMsg`
    });
    let join_tok = client.on(|msg: Join, w: Writer<_>| {
        // msg is now a `twitchchat::commands::Join`
    });

    // if a PRIVMSG or JOIN is parsed here
    // the corresponding closure, above, will run
    client.read_message();
    ```

    The available filters are the same names as the structs in
    [commands](./commands/index.html#structs)

    When [`Client::read_message`](./struct.Client.html#method.read_message)
    is called, it'll send a copy of the matching message to these filters.

    Multiple filters can be 'registered' for the same type

    Use the returned token to remove the filter, by passing it to the
    [`Client::off`](./struct.Client.html#method.off) method
    */
    pub fn on<F, T>(&mut self, mut f: F) -> Token
    where
        F: FnMut(T, Writer<W>) + 'static + Send + Sync, // hmm
        T: From<Message>,
        T: MessageFilter,
    {
        let filter = T::to_filter();
        self.filters
            .insert(filter, Box::new(move |msg, w| f(msg.into(), w)))
    }

    /// Remove a previously registered message filter, using the token returned by `on`
    ///
    /// Returns true if this filter existed
    pub fn off(&mut self, tok: Token) -> bool {
        self.filters.try_remove(tok)
    }

    /// Get a clone of the internal writer
    pub fn writer(&self) -> Writer<W> {
        self.writer.clone()
    }
}
