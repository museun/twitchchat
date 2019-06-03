use super::Writer;
use super::{commands, Message, Token, TokenGen};

use hashbrown::HashMap;

pub type FilterFn = Box<dyn FnMut(Message, Writer) + Send + Sync>;
pub struct FilterId(pub(super) FilterFn, pub(super) Token);
pub struct FilterMap(HashMap<Filter, Vec<FilterId>>, TokenGen);

impl Default for FilterMap {
    fn default() -> Self {
        Self(HashMap::default(), TokenGen::default())
    }
}

impl FilterMap {
    pub fn insert(&mut self, filter: Filter, f: FilterFn) -> Token {
        let token = self.1.next();
        self.0.entry(filter).or_default().push(FilterId(f, token));
        log::trace!("added filter for {:?} (id: {})", filter, token.0);
        token
    }

    pub fn try_remove(&mut self, token: Token) -> bool {
        for (filter, vals) in self.0.iter_mut() {
            if let Some(pos) = vals.iter().position(|d| d.1 == token) {
                log::trace!("removed filter for {:?} (id: {})", filter, token.0);
                let _ = vals.remove(pos);
                return true;
            }
        }
        log::trace!("could not find a matching filter for id: {}", token.0);
        false
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self, filter: Filter) -> Option<&Vec<FilterId>> {
        self.0.get(&filter)
    }

    pub(crate) fn get_mut(&mut self, filter: Filter) -> Option<&mut Vec<FilterId>> {
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
    // Reserve the right to add more fields to this enum
    #[doc(hidden)]
    __Nonexhaustive,
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
            Message::__Nonexhaustive => unreachable!(),
        }
    }
}
