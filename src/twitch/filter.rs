use super::Writer;
use super::{commands, Message};

use log::*;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;
#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

pub type FilterFn<W> = Box<dyn FnMut(Message, Writer<W>) + Send + Sync>;

/// A Token returned by the [`Client::on`](./struct.CLient.html#method.on) message filter
///
/// Keep this around if you want to remove the filter.
///
/// To remove one, use this with the [`Client::off`](./struct.CLient.html#method.off) method.
#[derive(Copy, Clone, PartialEq)]
pub struct Token(pub(super) usize);

struct TokenGen(AtomicUsize);

impl Default for TokenGen {
    fn default() -> Self {
        Self(AtomicUsize::new(0))
    }
}

impl TokenGen {
    fn next(&mut self) -> Token {
        Token(self.0.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct FilterId<W>(pub(super) FilterFn<W>, pub(super) Token);

pub struct FilterMap<W>(HashMap<Filter, Vec<FilterId<W>>>, TokenGen);

impl<W> Default for FilterMap<W> {
    fn default() -> Self {
        Self(HashMap::default(), TokenGen::default())
    }
}

impl<W: Write> FilterMap<W> {
    pub fn insert(&mut self, filter: Filter, f: FilterFn<W>) -> Token {
        let token = self.1.next();
        self.0.entry(filter).or_default().push(FilterId(f, token));
        trace!("added filter for {:?} (id: {})", filter, token.0);
        token
    }

    pub fn try_remove(&mut self, token: Token) -> bool {
        for (filter, vals) in self.0.iter_mut() {
            if let Some(pos) = vals.iter().position(|d| d.1 == token) {
                trace!("removed filter for {:?} (id: {})", filter, token.0);
                let _ = vals.remove(pos);
                return true;
            }
        }
        trace!("could not find a matching filter for id: {}", token.0);
        false
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self, filter: Filter) -> Option<&Vec<FilterId<W>>> {
        self.0.get(&filter)
    }

    pub(crate) fn get_mut(&mut self, filter: Filter) -> Option<&mut Vec<FilterId<W>>> {
        self.0.get_mut(&filter)
    }
}

macro_rules! filter_this {
    ($($t:ident),+ $(,)?) => {
        $(impl MessageFilter for commands::$t {
            fn to_filter() -> Filter {
                Filter::$t
            }
        }

        impl From<Message> for commands::$t {
            fn from(msg: Message) -> Self {
                match msg {
                    Message::$t(d @ commands::$t { .. }) => d,
                    _ => unreachable!(),
                }
            }
        })*
    };
}

filter_this!(
    Join,            //
    Part,            //
    PrivMsg,         //
    Mode,            //
    NamesStart,      //
    NamesEnd,        //
    ClearChat,       //
    ClearMsg,        //
    HostTargetStart, //
    HostTargetEnd,   //
    Notice,          //
    Reconnect,       //
    RoomState,       //
    UserNotice,      //
    UserState,       //
    GlobalUserState, //
);

#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum Filter {
    Irc,
    Join,
    Part,
    PrivMsg,
    Mode,
    NamesStart,
    NamesEnd,
    ClearChat,
    ClearMsg,
    HostTargetStart,
    HostTargetEnd,
    Notice,
    Reconnect,
    RoomState,
    UserNotice,
    UserState,
    GlobalUserState,
}

pub trait MessageFilter {
    fn to_filter() -> Filter;
}

impl Message {
    pub(crate) fn what_filter(&self) -> Filter {
        use Filter::*;
        match self {
            Message::Join { .. } => Join,
            Message::Part { .. } => Part,
            Message::PrivMsg { .. } => PrivMsg,
            Message::Mode { .. } => Mode,
            Message::NamesStart { .. } => NamesStart,
            Message::NamesEnd { .. } => NamesEnd,
            Message::ClearChat { .. } => ClearChat,
            Message::ClearMsg { .. } => ClearMsg,
            Message::HostTargetStart { .. } => HostTargetStart,
            Message::HostTargetEnd { .. } => HostTargetEnd,
            Message::Notice { .. } => Notice,
            Message::Reconnect { .. } => Reconnect,
            Message::RoomState { .. } => RoomState,
            Message::UserNotice { .. } => UserNotice,
            Message::UserState { .. } => UserState,
            Message::GlobalUserState { .. } => GlobalUserState,
            Message::Irc { .. } => Irc,
        }
    }
}
