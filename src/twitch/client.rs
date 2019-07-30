use super::adapter::{ReadAdapter, WriteAdapter};
use super::filter::{FilterMap, MessageFilter};
use super::handler::Handlers;
use super::{Capability, Error, LocalUser, Message, Token, UserConfig, Writer};

/// Client for interacting with Twitch's chat.
///
/// It wraps a [Read](https://doc.rust-lang.org/std/io/trait.Read.html),
/// [Write](https://doc.rust-lang.org/std/io/trait.Write.html) pair
///
/// ```no_run
/// use twitchchat::{helpers::TestStream, Client, sync_adapters};
/// let stream = TestStream::new();
/// // create a synchronous read and write adapter (shorthand for SyncReadAdapter::new() and SyncWriteAdapter::new())
/// let (r, w) = sync_adapters(stream.clone(), stream.clone());
/// let mut client = Client::new(r,w); // moves the r,w
/// // register, join, on, etc
/// client.run().unwrap();
/// ```
pub struct Client<R> {
    reader: R,

    filters: FilterMap,
    handlers: Handlers,

    writer: Writer,
}

impl<R: ReadAdapter> Client<R> {
    /// Create a new Client from a
    /// [Read](https://doc.rust-lang.org/std/io/trait.Read.html),
    /// [Write](https://doc.rust-lang.org/std/io/trait.Write.html) pair
    ///
    /// This client is clonable, and thread safe.
    pub fn new<W>(reader: R, writer: W) -> Self
    where
        W: WriteAdapter + Send + 'static,
    {
        let (writer_, rx) = Writer::new();

        let _ = std::thread::spawn(move || {
            log::trace!("starting write loop");
            let mut w = writer;
            for msg in rx {
                if w.write_line(msg.as_bytes()).is_err() {
                    break;
                }
            }
            log::trace!("ending write loop");
        });

        Self {
            reader,

            filters: FilterMap::default(),
            handlers: Handlers::default(),

            writer: writer_,
        }
    }

    /// Consumes the client, returning the reader
    pub fn into_reader(self) -> R::Reader {
        self.reader.into_inner()
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
                    log::warn!("invalid message: `{}`", msg);
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
    /// # use twitchchat::{helpers::TestStream, *};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = sync_adapters(stream.clone(), stream.clone());    
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

        self.writer.write_line(format!("PASS {}", config.token))?;
        self.writer.write_line(format!("NICK {}", config.nick))
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
    /// # use twitchchat::{helpers::TestStream, *};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = sync_adapters(stream.clone(), stream.clone());
    /// # let mut client = Client::new(r, w);
    /// match client.wait_for_ready() {
    ///     Ok(user) => println!("user id: {}", user.user_id),
    ///     Err(err) => panic!("failed to finish registration: {}", err)
    /// };
    /// // we can be sure that we're ready to join
    /// client.writer().join("some_channel").unwrap();
    /// ```
    pub fn wait_for_ready(&mut self) -> Result<LocalUser, Error> {
        use crate::irc::Message as IRCMessage;
        let mut caps = vec![];

        loop {
            match self.read_message()? {
                Message::Irc(msg) => {
                    match *msg {
                        IRCMessage::Cap {
                            // box patterns are nightly
                            acknowledge: true,
                            cap,
                        } => match cap.as_str() {
                            "twitch.tv/tags" => caps.push(Capability::Tags),
                            "twitch.tv/membership" => caps.push(Capability::Membership),
                            "twitch.tv/commands" => caps.push(Capability::Commands),
                            _ => {}
                        },
                        IRCMessage::Ready { .. } => {
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
                        _ => {}
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
    /// # use twitchchat::{helpers::TestStream, *};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = sync_adapters(stream.clone(), stream.clone());    
    /// # let mut client = Client::new(r, w);
    /// match client.wait_for_irc_ready() {
    ///     Ok(name) => println!("end of motd, our name is: {}", name),
    ///     Err(err) => panic!("failed to finish registration: {}", err),
    /// };
    /// // we can be sure that we're ready to join
    /// client.writer().join("some_channel").unwrap();
    /// ```
    pub fn wait_for_irc_ready(&mut self) -> Result<String, Error> {
        use crate::irc::Message as IrcMessage;
        loop {
            match self.read_message()? {
                Message::Irc(msg) => {
                    if let IrcMessage::Ready { name } = *msg {
                        return Ok(name);
                    }
                }
                _ => continue,
            }
        }
    }

    /// Reads a [`Message`](./enum.Message.html#variants)
    ///
    /// This 'pumps' the messages through the filter system
    ///
    /// Using this will drive the client (blocking for a read, then producing messages).
    /// Usage:
    /// ```no_run
    /// # use twitchchat::{helpers::TestStream, *};
    /// # let mut stream = TestStream::new();
    /// # let (r, w) = sync_adapters(stream.clone(), stream.clone());    
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
        let msg = self.reader.read_message()?;
        log::trace!("<- {:?}", msg);
        {
            let w = self.writer();
            if let Message::Irc(ref ircmsg) = msg {
                if let crate::irc::Message::Ping { token } = &**ircmsg {
                    return w
                        .write_line(format!("PONG :{}", token))
                        .and_then(|_| Ok(msg));
                }
            }

            let key = msg.what_filter();
            if let Some(filters) = self.filters.get_mut(key) {
                for filter in filters.iter_mut() {
                    log::trace!("sending msg to filter (id: {}): {:?}", (filter.1).0, key);
                    (filter.0)(msg.clone(), w.clone()) // when in doubt
                }
            }
        }
        log::trace!("begin dispatch");
        self.handlers.handle(msg.clone());
        log::trace!("end dispatch");
        Ok(msg)
    }
}

impl<R> Client<R> {
    /** When a message is received run this function with it and a clone of the Writer.

    The type of the closure determines what is filtered

    Usage:
    ```no_run
    # use twitchchat::{helpers::TestStream, *};
    # let mut stream = TestStream::new();
    # let (r, w) = sync_adapters(stream.clone(), stream.clone());
    # let mut client = Client::new(r, w);
    use twitchchat::commands::*;
    let pm_tok = client.on(|msg: PrivMsg, w: Writer| {
        // msg is now a `twitchchat::commands::PrivMsg`
    });
    let join_tok = client.on(|msg: Join, w: Writer| {
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
        F: FnMut(T, Writer) + 'static + Send + Sync,
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

    /**
    Add a [`Handler`](./trait.Handler.html) to the internal filtering

    When [`Client::read_message`](./struct.Client.html#method.read_message)
    is called, it'll send a RC message to the appropriate function.

    Use the returned token to remove the filter, by passing it to the
    [`Client::remove_handler`](./struct.Client.html#method.remove_handler) method
    */
    pub fn handler<H>(&mut self, handler: H) -> Token
    where
        H: super::Handler + Send + Sync + 'static,
    {
        let tok = self.handlers.add(handler);
        log::trace!("add handler, id: {}", tok);
        tok
    }

    /// Remove a previously added handler, using the returned token
    ///
    /// Returns true if this handler existed
    pub fn remove_handler(&mut self, tok: Token) -> bool {
        let ok = self.handlers.remove(tok);
        log::trace!("tried to remove handler, id: {}, status: {}", tok, ok);
        ok
    }

    /// Get a clone of the writer
    pub fn writer(&self) -> Writer {
        self.writer.clone()
    }
}

// TODO rate limit:
// 20 per 30 seconds	Users sending commands or messages to channels in which they do not have Moderator or Operator status
// 100 per 30 seconds	Users sending commands or messages to channels in which they have Moderator or Operator status
