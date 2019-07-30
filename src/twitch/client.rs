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
    irc_ready: bool, // other is implied if this is false

    quit: Arc<AtomicBool>,
}

// TODO quit channel

impl<R> Client<R>
where
    R: Read + Sync + Send,
{
    /// Create a new Client
    pub fn register<U, W>(config: U, read: R, write: W) -> Result<Self, Error>
    where
        U: std::borrow::Borrow<UserConfig>,
        W: Write + Sync + Send + 'static,
    {
        let quit = Arc::new(AtomicBool::new(false));
        let (writer, rx) = Writer::new(Arc::clone(&quit));

        let config = config.borrow();
        let want_tags = config.caps.contains(&Capability::Tags);
        for cap in config.caps.iter().filter_map(|c| c.get_command()) {
            writer.write_line(cap)?;
        }

        writer.write_line(format!("PASS {}", config.token))?;
        writer.write_line(format!("NICK {}", config.nick))?;

        // TODO keep this join handle around
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
            irc_ready: false,

            quit,
        })
    }

    /// Add this filter to the iterator
    pub fn filter<F: MessageFilter>(mut self) -> Self {
        let _ = self.filters.insert(F::to_filter());
        self
    }

    /// Sets the iterator to start when we've received the 'ok' from the irc server
    pub fn when_irc_ready(mut self) -> Self {
        self.irc_ready = true;
        self
    }

    /// Get a clonable writer from the client
    pub fn writer(&self) -> Writer {
        self.writer.clone()
    }

    /// This is useful to synchronize the closing of the 'Read'
    pub fn wait_for_close(self) {
        let _ = self.handle.join();
    }

    fn read_message(&mut self) -> Result<Option<Message>, Error> {
        if self.quit.load(Ordering::SeqCst) {
            return Ok(None);
        }

        let mut line = String::new(); // reuse this
        if self
            .reader
            .read_line(&mut line)
            .map_err(|_| Error::CannotRead)?
            == 0
        {
            return Err(Error::CannotRead);
        }

        if self.quit.load(Ordering::SeqCst) {
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
                        log::trace!("got a registration error");
                        return Err(Error::InvalidRegistration);
                    }
                }
                Ok(Some(Message::parse(msg)))
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

                // TODO this is 'smart' but we should report this as a configration error
                // if the user doesn't want tags and doesn't set irc_ready then we should error out
                self.state.next(ready);
                return Some(Event::Message(msg));
            }
            ClientState::IrcReady => loop {
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
                if self.filters.contains(&msg.what_filter()) {
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
