use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::{Capability, Error, LocalUser, Message, UserConfig, Writer};
use crate::filter::{Filter, MessageFilter};

// TODO handle shutdowns better

/// Client for interacting with Twitch's chat.
pub struct Client<R> {
    reader: BufReader<R>,
    writer: Writer,
    handle: std::thread::JoinHandle<()>,
    filters: HashSet<Filter>,

    desired_name: String,
    caps: Vec<Capability>,
    has_error: bool,

    ready_state: Option<ReadyState>,
    state: ClientState,

    want_tags: bool,
    irc_ready: bool,

    quit: Arc<AtomicBool>,
}

impl<R> Client<R>
where
    R: Read + Sync + Send,
{
    /// Creates and registers a new Client with the IRC server.
    ///
    /// Takes a [`UserConfig`](./struct.UserConfig.html) and a [`Read`](https://doc.rust-lang.org/std/io/trait.Read.html)/[`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) pair
    ///
    /// Returns the Client, or an error if it cannot write to the `W`
    pub fn register<U, W>(config: U, read: R, write: W) -> Result<Self, Error>
    where
        U: std::borrow::Borrow<UserConfig>,
        W: Write + Sync + Send + 'static,
    {
        let quit = Arc::new(AtomicBool::new(false));
        let (writer, rx) = Writer::new(Arc::clone(&quit));

        let config = config.borrow();
        // check for anonymous login ('justinfan1234')
        let is_anonymous = config.nick == super::userconfig::JUSTINFAN1234
            && config.token == super::userconfig::JUSTINFAN1234;

        let want_tags = config.caps.contains(&Capability::Tags) && !is_anonymous;
        for cap in config.caps.iter().filter_map(|c| c.get_command()) {
            writer.write_line(cap)?;
        }

        log::trace!("registering");
        writer.write_line(format!("PASS {}", config.token))?;
        writer.write_line(format!("NICK {}", config.nick))?;
        log::trace!("registered");

        let handle = std::thread::spawn(move || {
            log::trace!("starting write loop");
            let mut w = write;
            for msg in rx {
                if w.write_all(msg.as_bytes()).is_err() {
                    break;
                }
            }
            log::trace!("ending write loop");
        });

        Ok(Self {
            reader: BufReader::new(read),
            writer,
            handle,
            filters: HashSet::new(),

            desired_name: config.nick.to_string(),
            caps: vec![],
            has_error: false,

            ready_state: None,
            state: ClientState::Start,

            want_tags,
            irc_ready: is_anonymous,

            quit,
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

    /// Sets the iterator to start when we've received the 'ok' from the irc server
    ///
    /// When this event happens, a [`Event::IrcReady`](enum.Event.html#variant.IrcReady) is produced by the iterator
    ///
    /// - If the `tags` capability was not set, then this is automatically set.
    /// - If this is not set and the `tags` capability was set, then a [`Event::TwitchReady`](enum.Event.html#variant.TwitchReady) is produced instead
    ///
    /// It will try to be smart and produce a `TwitchReady` event if desired, otherwise `IrcReady` was produced
    pub fn when_irc_ready(mut self) -> Self {
        self.irc_ready = true;
        self
    }

    /// Get a clonable writer from the client
    pub fn writer(&self) -> Writer {
        log::trace!("cloning writer");
        self.writer.clone()
    }

    /// This is useful to synchronize the closing of the 'Read'
    pub fn wait_for_close(self) {
        log::trace!("waiting for thread to join");
        let _ = self.handle.join();
        log::trace!("thread joined");
    }

    fn read_message(&mut self) -> Result<Option<Message>, Error> {
        if self.quit.load(Ordering::SeqCst) {
            log::trace!("quitting");
            return Ok(None);
        }

        let mut line = String::new(); // reuse this
        if self.reader.read_line(&mut line).map_err(|err| {
            log::warn!("failed to read: {}", err);
            Error::CannotRead
        })? == 0
        {
            log::warn!("cannot read (amount was empty)");
            return Err(Error::CannotRead);
        }

        if self.quit.load(Ordering::SeqCst) {
            log::trace!("quitting");
            return Ok(None);
        }

        let _ = line.remove(line.len() - 1);
        assert!(!line.is_empty(), "line should not be just a '\r'");
        log::trace!("<- {}", line);
        let msg = crate::irc::Message::parse(&line).ok_or_else(|| Error::InvalidMessage(line))?;;

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
                        return Err(Error::InvalidRegistration);
                    }
                }
                Ok(Some(Message::parse(msg)))
            }
            crate::irc::Message::Ping { token } => {
                self.writer.write_line(format!("PONG :{}", token))?;
                Ok(Some(Message::Irc(Box::new(msg))))
            }
            _ => Ok(Some(Message::Irc(Box::new(msg)))),
        }
    }
}

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

// TODO this could be a lot simpler
impl<R> Iterator for Client<R>
where
    R: Read + Sync + Send,
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if self.quit.load(Ordering::SeqCst) {
            log::trace!("quitting");
            return None;
        }

        if self.has_error {
            return None;
        }

        macro_rules! error {
            ($err:expr) => {{
                self.has_error = true;
                return Some(Event::Error($err));
            }};
        }

        macro_rules! read {
            () => {
                match self.read_message() {
                    Ok(Some(msg)) => {
                        if self.quit.load(Ordering::SeqCst) {
                            log::trace!("quitting");
                            return None;
                        }
                        msg
                    }
                    Ok(None) => return None,
                    Err(err) => error!(err),
                }
            };
        }

        if self.ready_state.is_none() {
            let _ = self
                .ready_state
                .replace(match (self.irc_ready, self.want_tags) {
                    (true, false) | (false, false) | (true, true) => ReadyState::Irc,
                    (false, true) => ReadyState::Twitch,
                });
        }
        let ready = self.ready_state.unwrap();

        match self.state {
            ClientState::Start => {
                log::trace!("state is: {:?}", self.state);
                let msg = read!();
                match &msg {
                    Message::Irc(msg) => match &**msg {
                        crate::irc::Message::Cap {
                            acknowledge: true,
                            cap,
                        } => match cap.as_str() {
                            "twitch.tv/tags" => self.caps.push(Capability::Tags),
                            "twitch.tv/membership" => self.caps.push(Capability::Membership),
                            "twitch.tv/commands" => self.caps.push(Capability::Commands),
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                };

                self.state.next(ready);
                return Some(Event::Message(msg));
            }
            ClientState::IrcReady => loop {
                log::trace!("state is: {:?}", self.state);
                match read!() {
                    Message::Irc(msg) => {
                        if let crate::irc::Message::Ready { name } = *msg {
                            self.state.next(ready);
                            return Some(Event::IrcReady(name));
                        }
                    }
                    _ => continue,
                }
            },
            ClientState::TwitchReady => loop {
                log::trace!("state is: {:?}", &self.state);
                match read!() {
                    Message::Irc(msg) => match *msg {
                        crate::irc::Message::Cap {
                            acknowledge: true,
                            cap,
                        } => match cap.as_str() {
                            "twitch.tv/tags" => self.caps.push(Capability::Tags),
                            "twitch.tv/membership" => self.caps.push(Capability::Membership),
                            "twitch.tv/commands" => self.caps.push(Capability::Commands),
                            _ => {}
                        },
                        crate::irc::Message::Ready { .. } => {
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
                        _ => {}
                    },
                    Message::GlobalUserState(state) => {
                        self.state.next(ready);
                        return Some(Event::TwitchReady(LocalUser {
                            user_id: dbg!(&state).user_id(),
                            display_name: state.display_name().map(ToString::to_string),
                            name: self.desired_name.clone(),
                            color: state.color(),
                            badges: state.badges(),
                            emote_sets: state.emote_sets(),
                            caps: self.caps.clone(),
                        }));
                    }
                    _ => continue,
                }
            },
            ClientState::Go => loop {
                let msg = read!();
                let filter = msg.what_filter();
                if self.filters.contains(&filter) {
                    log::debug!("dispatching to a : {:?}", filter);
                    return Some(Event::Message(msg));
                }
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ReadyState {
    Irc,
    Twitch,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ClientState {
    Start = 0,
    IrcReady = 1,
    TwitchReady = 2,
    Go = 3,
}

impl ClientState {
    fn next(&mut self, ready: ReadyState) {
        let _ = match self {
            ClientState::Start if ready == ReadyState::Irc => {
                std::mem::replace(self, ClientState::IrcReady)
            }
            ClientState::Start => std::mem::replace(self, ClientState::TwitchReady),
            _ => std::mem::replace(self, ClientState::Go),
        };
    }
}
