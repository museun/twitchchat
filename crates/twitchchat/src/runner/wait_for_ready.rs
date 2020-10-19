use std::collections::HashSet;

cfg_async! { use futures_lite::{AsyncRead, AsyncWrite}; }

cfg_async! {
    use crate::{
        AsyncDecoder,
        AsyncEncoder,
    };
}

use crate::{
    commands, //
    twitch::Capability,
    Decoder,
    Encoder,
    IntoOwned as _,
    IrcMessage,
    UserConfig,
};

use super::{Capabilities, Error, Identity};

/// Register and wait until Twitch finishes the handshake
///
/// This returns your [`Identity`] and any _missed_ messages
///
/// On failure, it'll return:
///
/// | error                                 | cause                                           |
/// | ------------------------------------- | ----------------------------------------------- |
/// | [`InvalidCap`][invalid_cap]           | you provided an invalid capability              |
/// | [`BadPass`][bad_pass]                 | you provided an invalid OAuth token             |
/// | [`Io`][io]                            | an i/o error occured                            |
/// | [`ShouldReconnect`][should_reconnect] | the server was restarting, you should reconnect |
/// | [`UnexpectedEof`][unexpected_eof]     | the server closed the connection abruptly       |
///
/// # Example
///
/// ```no_run
/// # use twitchchat::{*, runner::*};
/// # let mut stream = std::io::Cursor::new(Vec::new());
/// # let user_config = UserConfig::builder().anonymous().enable_all_capabilities().build().unwrap();
///
/// // give it your stream and your user configuration
/// match runner::wait_until_ready_sync(&mut stream, &user_config) {
///     Ok((identity, missed_messages)) => {
///         // identity is your Identity
///         // this'll have things like your name/display name and capabilities were acknowledged
///
///         // missed_messages are any messages consumed while waiting for everything to be ready
///         // you can feed these back into the decoder, if you wish:
///         // your_decoder.extend(&mut missed_messages);
///     }
///     Err(err) => panic!("could not connect: {}", err)
/// };
/// ```
///
/// [invalid_cap]: crate::runner::Error::InvalidCap
/// [bad_pass]: crate::runner::Error::BadPass
/// [io]: crate::runner::Error::Io
/// [timed_out]: crate::runner::Error::TimedOut
/// [should_reconnect]: crate::runner::Error::ShouldReconnect
/// [unexpected_eof]: crate::runner::Error::UnexpectedEof
pub fn wait_until_ready_sync<IO>(
    io: &mut IO,
    user_config: &UserConfig,
) -> Result<(Identity, Vec<IrcMessage<'static>>), Error>
where
    IO: std::io::Read + std::io::Write,
{
    let mut missed_messages = Vec::new();

    let (read, write) = crate::util::split(io);
    let mut read = Decoder::new(read);
    let mut write = Encoder::new(write);

    write.encode(commands::register(user_config))?;

    let mut state = ReadyState::new(user_config);
    loop {
        let msg = match read.read_message() {
            Err(crate::DecodeError::Io(err)) if crate::util::is_blocking_error(&err) => continue,
            Err(err) => Err(err),
            Ok(ok) => Ok(ok),
        }?;

        match check_message(&msg, &mut state)? {
            StepState::Skip => continue,
            StepState::Continue => {
                missed_messages.push(msg.into_owned());
                continue;
            }
            StepState::ShouldPong(token) => write.encode(commands::pong(&token))?,
            StepState::Identity(identity) => return Ok((identity, missed_messages)),
        }
    }
}

cfg_async! {
/// Register and wait until Twitch finishes the handshake
///
/// This returns your [`Identity`] and any _missed_ messages
///
/// On failure, it'll return:
///
/// | error                                 | cause                                           |
/// | ------------------------------------- | ----------------------------------------------- |
/// | [`InvalidCap`][invalid_cap]           | you provided an invalid capability              |
/// | [`BadPass`][bad_pass]                 | you provided an invalid OAuth token             |
/// | [`Io`][io]                            | an i/o error occured                            |
/// | [`ShouldReconnect`][should_reconnect] | the server was restarting, you should reconnect |
/// | [`UnexpectedEof`][unexpected_eof]     | the server closed the connection abruptly       |
///
/// # Example
///
/// ```no_run
/// # use twitchchat::{*, runner::*};
/// # let mut stream = futures_lite::io::Cursor::new(Vec::new());
/// # let user_config = UserConfig::builder().anonymous().enable_all_capabilities().build().unwrap();
/// # let fut = async {
/// // give it your stream and your user configuration
/// match runner::wait_until_ready(&mut stream, &user_config).await {
///     Ok((identity, missed_messages)) => {
///         // identity is your Identity
///         // this'll have things like your name/display name and capabilities were acknowledged
///
///         // missed_messages are any messages consumed while waiting for everything to be ready
///         // you can feed these back into the decoder, if you wish:
///         // your_decoder.extend(&mut missed_messages);
///     }
///     Err(err) => panic!("could not connect: {}", err)
/// };
/// # };
/// ```
///
/// [invalid_cap]: crate::runner::Error::InvalidCap
/// [bad_pass]: crate::runner::Error::BadPass
/// [io]: crate::runner::Error::Io
/// [timed_out]: crate::runner::Error::TimedOut
/// [should_reconnect]: crate::runner::Error::ShouldReconnect
/// [unexpected_eof]: crate::runner::Error::UnexpectedEof
pub async fn wait_until_ready<IO>(
    io: &mut IO,
    user_config: &UserConfig,
) -> Result<(Identity, Vec<IrcMessage<'static>>), Error>
where
    IO: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
{
    let mut missed_messages = Vec::new();

    let (read, write) = futures_lite::io::split(io);
    let mut read = AsyncDecoder::new(read);
    let mut write = AsyncEncoder::new(write);

    write.encode(commands::register(user_config)).await?;

    let mut state = ReadyState::new(user_config);
    loop {
        let msg = match read.read_message().await {
            Err(crate::DecodeError::Io(err)) if crate::util::is_blocking_error(&err) => {
                futures_lite::future::yield_now().await;
                continue;
            }
            Err(err) => Err(err),
            Ok(ok) => Ok(ok),
        }?;
        match check_message(&msg, &mut state)? {
            StepState::Skip => continue,
            StepState::Continue => {
                missed_messages.push(msg.into_owned());
                continue;
            }
            StepState::ShouldPong(token) => write.encode(commands::pong(&token)).await?,
            StepState::Identity(identity) => return Ok((identity, missed_messages)),
        }
    }
}
}

fn check_message(msg: &IrcMessage<'_>, state: &mut ReadyState) -> Result<StepState, Error> {
    const FAILURE: &str = "Login authentication failed";

    use crate::{
        messages::Capability as TwitchCap, //
        messages::Commands as C,
        FromIrcMessage as _,
    };

    match C::from_irc(msg.clone()).unwrap() {
        C::Notice(msg) if msg.message() == FAILURE => Err(Error::BadPass),

        C::Ready(msg) => {
            state.our_name.replace(msg.username().to_string());

            // if we aren't going to be receiving tags, then we
            // won't be looking for any more messages

            // if we're anonymous, we won't get GLOBALUSERSTATE even
            // if we do send Tags
            if state.is_anonymous {
                return Ok(StepState::Identity(Identity::Anonymous {
                    caps: std::mem::take(&mut state.caps),
                }));
            }

            // if we're not looking for any more caps and we won't be
            // getting a GlobalUserState just give them the Basic Identity
            if state.looking_for.is_empty() && !state.will_be_getting_global_user_state_hopefully {
                return Ok(StepState::Identity(Identity::Basic {
                    name: state.our_name.take().unwrap(),
                    caps: std::mem::take(&mut state.caps),
                }));
            }

            Ok(StepState::Continue)
        }

        C::Cap(msg) => {
            match msg.capability() {
                TwitchCap::Acknowledged(name) => {
                    use crate::twitch::Capability as Cap;

                    let cap = match name.parse() {
                        Ok(cap) => cap,
                        // Twitch sent us an unknown capability
                        Err(..) => {
                            state.caps.unknown.insert(name.to_string());
                            return Ok(StepState::Continue);
                        }
                    };

                    *match cap {
                        Cap::Tags => &mut state.caps.tags,
                        Cap::Membership => &mut state.caps.membership,
                        Cap::Commands => &mut state.caps.commands,
                    } = true;

                    state.looking_for.remove(&cap);
                }

                TwitchCap::NotAcknowledged(name) => {
                    return Err(Error::InvalidCap {
                        cap: name.to_string(),
                    })
                }
            }

            Ok(StepState::Continue)
        }

        // NOTE: This will only be sent when there's both Commands and atleast
        // one other CAP requested
        C::GlobalUserState(msg) => {
            // TODO: this is so shitty.
            let id = match msg.user_id {
                Some(id) => id.parse().unwrap(),
                // XXX: we can get this message without any tags
                None => {
                    return Ok(StepState::Identity(Identity::Basic {
                        name: state.our_name.take().unwrap(),
                        caps: std::mem::take(&mut state.caps),
                    }))
                }
            };

            Ok(StepState::Identity(Identity::Full {
                // these unwraps should be safe because we'll have all of the TAGs here
                name: state.our_name.take().unwrap(),
                user_id: id,
                display_name: msg.display_name.map(|s| s.to_string()),
                color: msg.color,
                caps: std::mem::take(&mut state.caps),
            }))
        }

        // Reply to any PINGs while waiting. Although Twitch doesn't
        // currently send a PING for spoof detection on initial
        // handshake, one day they may. Most IRC servers do this
        // already
        C::Ping(msg) => Ok(StepState::ShouldPong(msg.token().to_string())),

        // Skip this message because its a synthetic response
        C::Pong(_) => Ok(StepState::Skip),

        C::Reconnect(_) => Err(Error::ShouldReconnect),

        // we have our name, but we won't be getting GlobalUserState and we've
        // got all of our Caps
        _ if state.our_name.is_some()
            && !state.will_be_getting_global_user_state_hopefully
            && state.looking_for.is_empty() =>
        {
            Ok(StepState::Identity(Identity::Basic {
                name: state.our_name.take().unwrap(),
                caps: std::mem::take(&mut state.caps),
            }))
        }

        _ => Ok(StepState::Continue),
    }
}

enum StepState {
    Skip,
    Continue,
    ShouldPong(String),
    Identity(Identity),
}

struct ReadyState {
    looking_for: HashSet<Capability>,
    caps: Capabilities,
    our_name: Option<String>,
    is_anonymous: bool,
    will_be_getting_global_user_state_hopefully: bool,
}

impl ReadyState {
    fn new(user_config: &UserConfig) -> Self {
        let is_anonymous = user_config.is_anonymous();

        let will_be_getting_global_user_state_hopefully =
            user_config.capabilities.contains(&Capability::Tags)
                && user_config.capabilities.contains(&Capability::Commands);

        Self {
            looking_for: user_config.capabilities.iter().copied().collect(),
            caps: Capabilities::default(),
            our_name: None,
            is_anonymous,
            will_be_getting_global_user_state_hopefully,
        }
    }
}

// TODO record a bunch of twitch handshakes and test them here
