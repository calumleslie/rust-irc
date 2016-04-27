
use command::Command;
use std;
use std::convert::Into;
use std::fmt::Display;
use std::fmt::Formatter;
use std::iter::Iterator;
use std::vec::Vec;

/// A single IRC message, as sent to and from server and client.
#[derive(Debug,Clone, PartialEq, Eq)]
pub struct Message {
    pub prefix: Prefix,
    pub command: Command,
    pub arguments: Vec<String>,
}

/// The prefix of an IRC message.
#[derive(Debug,Clone, PartialEq, Eq)]
pub enum Prefix {
    /// The message has no prefix.
    None,
    /// The prefix is a server hostname.
    Server(String),
    /// The prefix is information about a user.
    User(UserInfo),
}

/// Information about a user, as provided in the prefix of an IRC message.
/// Contains a nickname (`nickname`), and (optionally) information about the
/// host and username of the user (`host`)
#[derive(Debug,Clone, PartialEq, Eq)]
pub enum UserInfo {
    /// Nickname-only, as in prefix `:nickname`
    Nick(String),
    /// Nickname and host, as in prefix `:nickname@host`
    NickHost(String, String),
    /// Nickname, username, and host, as in prefix `:nickname!username@host`
    NickUserHost(String, String, String),
}

impl Message {
    /// Creates a new Message instance.
    pub fn new(prefix: Prefix, command: Command, arguments: Vec<String>) -> Self {
        Message {
            prefix: prefix,
            command: command,
            arguments: arguments,
        }
    }

    pub fn from_strs(prefix: Prefix, command: Command, arguments: Vec<&str>) -> Self {
        let cows: Vec<String> = arguments.iter().map(|arg| arg.to_string()).collect();

        Self::new(prefix, command, cows)
    }
}

impl UserInfo {
    pub fn of_nickname(nickname: &str) -> Self {
        UserInfo::Nick(nickname.into())
    }

    pub fn of_nickname_host(nickname: &str, host: &str) -> Self {
        UserInfo::NickHost(nickname.into(), host.into())
    }

    pub fn of_nickname_user_host(nickname: &str, user: &str, host: &str) -> Self {
        UserInfo::NickUserHost(nickname.into(), user.into(), host.into())
    }

    /// Convenience method to get a Prefix::User instance from this UserInfo.
    pub fn to_prefix<'a>(self) -> Prefix {
        Prefix::User(self)
    }

    pub fn nickname(&self) -> &str {
        match *self {
            UserInfo::Nick(ref nick) => nick,
            UserInfo::NickHost(ref nick, _) => nick,
            UserInfo::NickUserHost(ref nick, _, _) => nick,
        }
    }

    pub fn host(&self) -> Option<&str> {
        match *self {
            UserInfo::Nick(_) => None,
            UserInfo::NickHost(_, ref host) => Some(host),
            UserInfo::NickUserHost(_, _, ref host) => Some(host),
        }
    }

    pub fn username(&self) -> Option<&str> {
        match *self {
            UserInfo::Nick(_) => None,
            UserInfo::NickHost(_, _) => None,
            UserInfo::NickUserHost(_, ref user, _) => Some(user),
        }
    }
}

impl Display for UserInfo {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match *self {
            UserInfo::Nick(ref nick) => write!(fmt, "{}", nick),
            UserInfo::NickHost(ref nick, ref host) => write!(fmt, "{}@{}", nick, host),
            UserInfo::NickUserHost(ref nick, ref user, ref host) => {
                write!(fmt, "{}!{}@{}", nick, user, host)
            }
        }
    }
}

// Is using "Display" to format these for the wire a misuse?
// Should we be using a Write or soemthing instead?
impl Display for Message {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        try!(match self.prefix {
            Prefix::None => Ok(()),
            Prefix::Server(ref server) => write!(fmt, ":{} ", server),
            Prefix::User(ref user_info) => write!(fmt, ":{} ", user_info),
        });

        try!(write!(fmt, "{}", self.command));

        for (i, argument) in self.arguments.iter().enumerate() {
            try!(write!(fmt, " "));

            if i == self.arguments.len() - 1 && argument.contains(' ') {
                try!(write!(fmt, ":"));
            }

            try!(write!(fmt, "{}", argument));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use command::commands::{PING, PRIVMSG};
    use command::responses::RPL_WELCOME;

    #[test]
    fn command_only() {
        let line = Message::new(Prefix::None, PING(), vec![]);

        assert_eq!(format!("{}", line), "PING");
    }

    #[test]
    fn server_prefix() {
        let line = Message::new(Prefix::Server("somedude".into()), PING(), vec![]);

        assert_eq!(format!("{}", line), ":somedude PING");
    }

    #[test]
    fn response() {
        let line = Message::from_strs(Prefix::Server("some.server.here".into()),
                                      RPL_WELCOME(),
                                      vec!["Welcome to the server!"]);

        assert_eq!(format!("{}", line),
                   ":some.server.here 001 :Welcome to the server!");
    }

    #[test]
    fn user_prefix_nickname_only() {
        let line = Message::from_strs(UserInfo::of_nickname("nickname".into()).to_prefix(),
                                      PING(),
                                      vec![]);

        assert_eq!(format!("{}", line), ":nickname PING");
    }

    #[test]
    fn user_prefix_nickname_host() {
        let user_info = UserInfo::of_nickname_host("nickname".into(), "some.host.name".into());
        let line = Message::new(user_info.to_prefix(), PING(), vec![]);

        assert_eq!(format!("{}", line), ":nickname@some.host.name PING");
    }

    #[test]
    fn user_prefix_all_user_info() {
        let user_info = UserInfo::of_nickname_user_host("nickname".into(),
                                                        "realname".into(),
                                                        "some.host.name".into());
        let line = Message::new(user_info.to_prefix(), PING(), vec![]);

        assert_eq!(format!("{}", line),
                   ":nickname!realname@some.host.name PING");
    }

    #[test]
    fn command_args() {
        let line = Message::from_strs(Prefix::None, PRIVMSG(), vec!["someone", "something"]);

        assert_eq!(format!("{}", line), "PRIVMSG someone something");
    }

    #[test]
    fn command_args_with_long_final_argument() {
        let line = Message::from_strs(Prefix::None,
                                      PRIVMSG(),
                                      vec!["someone", "Hey I love being on IRC"]);

        assert_eq!(format!("{}", line),
                   "PRIVMSG someone :Hey I love being on IRC");
    }

    #[test]
    fn everything() {
        let line = Message::from_strs(Prefix::Server("information".into()),
                                      PRIVMSG(),
                                      vec!["someone", "something", "Hey I love being on IRC"]);

        assert_eq!(format!("{}", line),
                   ":information PRIVMSG someone something :Hey I love being on IRC");
    }
}
