use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read, Write};

use super::{Capability, Error, LocalUser, Message, UserConfig, Writer};
use crate::filter::{Filter, MessageFilter};

/// An event received while reading from the client
#[derive(Debug)]
pub enum Event {
    /// An IRC ready event was requested, returning the your IRC name
    IrcReady(String),
    /// A twitch ready event was requested, returning your twitch user
    TwitchReady(LocalUser),
    /// A twitch message
    Message(Message),
    /// An error
    Error(Error),
}

// TODO handle shutdowns better

/// Client for interacting with Twitch's chat.
pub struct Client<R, W> {
    reader: BufReader<R>,
    writer: Writer,
    output: W,
    write_queue: crossbeam_channel::Receiver<String>,
    filters: HashSet<Filter>,
    desired_name: String,
    client_config: ClientConfig,
}

impl<R, W> Client<R, W>
where
    R: Read + Sync + Send,
    W: Write + Sync + Send + 'static,
{
    /// Creates and registers a new Client with the IRC server.
    ///
    /// Takes a [`UserConfig`](./struct.UserConfig.html) and a [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html)/[`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) pair
    ///
    /// Returns the Client, or an error if it cannot write to the `W`
    pub fn register<U>(config: U, read: R, write: W) -> Result<Self, Error>
    where
        U: std::borrow::Borrow<UserConfig>,
    {
        let (writer, rx) = Writer::new();

        let config = config.borrow();
        use super::userconfig::JUSTINFAN1234 as ANON;
        // check for anonymous login ('justinfan1234')
        let is_anonymous = config.nick == ANON && config.token == ANON;

        let want_tags = config.caps.contains(&Capability::Tags) && !is_anonymous;
        for cap in config.caps.iter().filter_map(|c| c.get_command()) {
            writer.write_line(cap)?;
        }

        log::trace!("registering");
        writer.write_line(format!("PASS {}", config.token))?;
        writer.write_line(format!("NICK {}", config.nick))?;
        log::trace!("registered");

        Ok(Self {
            reader: BufReader::new(read),
            writer,
            output: write,
            write_queue: rx,
            filters: HashSet::new(),
            desired_name: config.nick.to_string(),
            client_config: ClientConfig {
                want_tags,
                is_anonymous,
            },
        })
    }

    /// Add this filter to the iterator
    ///
    /// When this type of `command` is received, it'll be *produced* by the `Client Iterator` as a [`Event::Message`](./enum.Event.html#variant.Message)
    ///
    /// A MessageFilter is basically a magic type that represents a [`Command`](./commands/index.html)
    ///
    /// To use this simply:
    /// ```ignore
    /// use twitchchat::commands::*;
    /// // client is created by Client::register()
    /// client.filter::<PrivMsg>() // add a PrivMsg
    ///       .filter::<Join>() // add a Join
    /// // and so forth
    /// ```
    /// The 'Command' type is used as a turbofish argument    
    pub fn filter<F: MessageFilter>(mut self) -> Self {
        let filter = F::to_filter();
        log::trace!("adding filter: {:?}", filter);
        let _ = self.filters.insert(filter);
        self
    }

    /// Remove this filter
    ///    
    /// Returns whether this fitler was present
    ///
    /// **note** This type isn't chainable
    ///
    /// ```ignore
    /// let client client.filter::<PrivMsg>();
    /// assert!(client.remove_filter::<PrivMsg>());
    /// assert!(!client.remove_filter::<PrivMsg());
    /// ```
    pub fn remove_filter<F: MessageFilter>(&mut self) -> bool {
        let filter = F::to_filter();
        log::trace!("removing filter: {:?}", filter);
        self.filters.remove(&filter)
    }

    /// Get a clonable writer from the client
    pub fn writer(&self) -> Writer {
        log::trace!("cloning writer");
        self.writer.clone()
    }

    /// Manually try to read a message from the connection in a non-blocking fashion
    ///
    /// Returns [`Some(Message)`](./enum.Message.html) if it read a message
    ///
    /// Returns `None` if it couldn't, but didn't run into an error
    pub fn read_message(&mut self) -> Result<Option<Message>, Error> {
        match self.try_read() {
            Production::Produce(msg) => Ok(Some(msg)),
            Production::Yield => Ok(None),
            Production::Error(err) => Err(err),
        }
    }

    /// Returns a blocking iterator over the [`Events`](./enum.Event.html) produced by the [`Client`](./struct.Client.html)
    pub fn iter(self) -> BlockingClientIter<R, W> {
        self.into_iter()
    }

    /// Returns a non-blocking iterator over the [`Events`](./enum.Event.html) produced by the [`Client`](./struct.Client.html)
    ///
    /// This will produce an [`Option<Event>`](./enum.Event.html)
    /// * when a message was received: [`Some(Event)`](./enum.Event.html)
    /// * when no message was ready: `None`
    pub fn nonblocking_iter(self) -> ClientIter<R, W> {
        ClientIter {
            client: ClientState {
                client_config: self.client_config,
                client: self,
                state: TaggedState::Start,
                caps: vec![],
                done: false,
            },
        }
    }

    fn try_read(&mut self) -> Production<Message, Error> {
        let mut line = String::new(); // reuse this

        match self.reader.read_line(&mut line) {
            Ok(0) => {
                log::warn!("cannot read (amount was empty)");
                // TODO this should be EOF / Disconnected
                return Production::Error(Error::CannotRead);
            }
            Err(err) => {
                use std::io::ErrorKind as E;
                (match err.kind() {
                    E::WouldBlock => {
                        const READ_AT_MOST: std::time::Duration =
                            std::time::Duration::from_millis(10);
                        let now = std::time::Instant::now();
                        // try to read all of the write queue, if 10
                        // milliseconds pass we'll bail leaving the rest in the
                        // queue if there is nothing to write, we'll bail early
                        for ts in std::iter::repeat(std::time::Instant::now()) {
                            match self.write_queue.try_recv() {
                                Ok(msg) => {
                                    if let Err(err) = self.output.write_all(msg.as_bytes()) {
                                        log::debug!("cannot write: {}", err);
                                        return Production::Error(Error::NotConnected);
                                    }
                                }
                                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                                    return Production::Error(Error::NotConnected)
                                }
                                _ => break,
                            }
                            if ts - now > READ_AT_MOST {
                                break;
                            }
                        }
                        return Production::Yield;
                    }
                    // TODO these should be special-cased
                    // E::TimedOut | E::NotConnected
                    _ => return Production::Error(Error::CannotRead),
                })
            }
            _ => {}
        }

        let _ = line.remove(line.len() - 1);
        assert!(!line.is_empty(), "line should not be just a '\r'");

        log::trace!("<- {}", line);
        let msg = match crate::irc::Message::parse(&line) {
            Some(msg) => msg,
            None => return Production::Error(Error::InvalidMessage(line)),
        };

        match &msg {
            crate::irc::Message::Unknown {
                prefix,
                head,
                args,
                tail,
                ..
            } => {
                if let (Some(crate::irc::Prefix::Server { host }), Some(data)) = (prefix, tail) {
                    if head == "NOTICE"
                        && host == "tmi.twitch.tv"
                        && data == "Improperly formatted auth"
                        && args.get(0).map(|k| k.as_str()) == Some("*")
                    {
                        log::warn!("got a registration error");
                        return Production::Error(Error::InvalidRegistration);
                    }
                }
                Production::Produce(Message::parse(msg))
            }
            crate::irc::Message::Ping { token } => {
                match self.writer.write_line(format!("PONG :{}", token)) {
                    Err(err) => Production::Error(err),
                    Ok(_) => Production::Produce(Message::Irc(Box::new(msg))),
                }
            }
            _ => Production::Produce(Message::Irc(Box::new(msg))),
        }
    }
}

impl<R, W> IntoIterator for Client<R, W>
where
    R: Read + Sync + Send,
    W: Write + Sync + Send + 'static,
{
    type Item = Event;
    type IntoIter = BlockingClientIter<R, W>;
    fn into_iter(self) -> Self::IntoIter {
        BlockingClientIter {
            client: ClientState {
                client_config: self.client_config,
                client: self,
                state: TaggedState::Start,
                caps: vec![],
                done: false,
            },
        }
    }
}

pub struct BlockingClientIter<R, W> {
    client: ClientState<R, W>,
}

impl<R, W> Iterator for BlockingClientIter<R, W>
where
    R: Read + Sync + Send,
    W: Write + Sync + Send + 'static,
{
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {
        const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10);

        // TODO don't spin wait here, be smarter and use a scheduler
        loop {
            if let Some(ev) = self.client.try_read()? {
                return Some(ev);
            }
            // TODO enqueue and drain from it
            std::thread::park_timeout(READ_TIMEOUT)
        }
    }
}

pub struct ClientIter<R, W> {
    client: ClientState<R, W>,
}

impl<R, W> Iterator for ClientIter<R, W>
where
    R: Read + Sync + Send,
    W: Write + Sync + Send + 'static,
{
    type Item = Option<Event>;
    fn next(&mut self) -> Option<Self::Item> {
        self.client.try_read()
    }
}

#[derive(Debug, Copy, Clone)]
struct ClientConfig {
    want_tags: bool,
    is_anonymous: bool,
}

#[derive(Debug)]
enum Production<T, E> {
    Produce(T),
    Error(E),
    Yield,
}

struct ClientState<R, W> {
    client: Client<R, W>,
    state: TaggedState,
    caps: Vec<Capability>,
    client_config: ClientConfig,
    done: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum TaggedState {
    Start = 0,
    TwitchReady = 1,
    Rest = 2,
}

impl<R, W> ClientState<R, W>
where
    R: Read + Send + Sync,
    W: Write + Sync + Send + 'static,
{
    // TODO this definitely needs testing to ensure the state transitions are
    // correct
    #[allow(clippy::option_option)]
    fn try_read(&mut self) -> Option<Option<Event>> {
        use crate::irc::Message as I;
        use crate::Message as M;

        macro_rules! produce {
            ($ev:expr) => {
                return Some(Some($ev));
            };
        };

        macro_rules! error {
            ($err:expr) => {{
                self.done = true;
                return Some(Some(Event::Error($err)));
            }};
        }

        loop {
            let msg = match self.client.try_read() {
                Production::Produce(msg) => msg,
                Production::Error(err) => error!(err),
                Production::Yield => return Some(None),
            };

            match self.state {
                // if we're at the start
                TaggedState::Start => {
                    if let M::Irc(msg) = &msg {
                        match &**msg {
                            I::Cap {
                                acknowledge: true,
                                cap,
                            } => match cap.as_str() {
                                "twitch.tv/tags" => self.caps.push(Capability::Tags),
                                "twitch.tv/membership" => self.caps.push(Capability::Membership),
                                "twitch.tv/commands" => self.caps.push(Capability::Commands),
                                unk => log::warn!("unknown capability: {}", unk),
                            },
                            // this an 001, so we can transition to the next state
                            I::Connected { name } => {
                                self.state = TaggedState::TwitchReady;
                                produce!(Event::IrcReady(name.clone()));
                            }
                            _ => { /* fallthrough */ }
                        }
                    }
                }

                // if we've reached 001
                TaggedState::TwitchReady => {
                    // this is the MOTD, check out caps
                    match &msg {
                        M::Irc(msg) => {
                            if let I::Ready { .. } = &**msg {
                                if !self.client_config.is_anonymous || self.client_config.want_tags
                                {
                                    let mut bad = vec![];
                                    match (
                                        self.caps.contains(&Capability::Tags),
                                        self.caps.contains(&Capability::Commands),
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
                                        error!(Error::CapabilityRequired(bad))
                                    }
                                }
                                self.state = TaggedState::Rest;
                            }
                        }

                        M::GlobalUserState(state) => {
                            self.state = TaggedState::Rest;
                            if !self.client_config.is_anonymous || self.client_config.want_tags {
                                produce!(Event::TwitchReady(LocalUser::from_global_user_state(
                                    state,
                                    self.client.desired_name.clone(),
                                    self.caps.clone()
                                )));
                            }
                        }
                        _ => { /* fallthrough */ }
                    }
                }
                TaggedState::Rest => {
                    let filter = msg.what_filter();
                    if self.client.filters.contains(&filter) {
                        log::debug!("dispatching to: {:?}", filter);
                        produce!(Event::Message(msg))
                    }
                    // so we don't fallthrough
                    continue;
                }
            };

            produce!(Event::Message(msg))
        }
    }
}
