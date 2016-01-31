
use command::Command;
use std;
use std::fmt::Display;
use std::fmt::Formatter;
use std::vec::Vec;

/// A single IRC message, as sent to and from server and client.
#[derive(Debug,Clone, PartialEq, Eq)]
pub struct Message<'a> {
    prefix: Prefix<'a>,
    command: Command<'a>,
    arguments: Vec<&'a str>,
}

/// The prefix of an IRC message.
#[derive(Debug,Clone, PartialEq, Eq)]
pub enum Prefix<'a> {
    /// The message has no prefix.
    None,
    /// The prefix is a server hostname.
    Server(&'a str),
    /// The prefix is information about a user.
    User(UserInfo<'a>),
}

/// Information about a user, as provided in the prefix of an IRC message.
/// Contains a nickname (`nickname`), and (optionally) information about the
/// host and username of the user (`host`)
#[derive(Debug,Clone, PartialEq, Eq)]
pub enum UserInfo<'a> {
    /// Nickname-only, as in prefix `:nickname`
    Nick(&'a str),
    /// Nickname and host, as in prefix `:nickname@host`
    NickHost(&'a str, &'a str),
    /// Nickname, username, and host, as in prefix `:nickname!username@host`
    NickUserHost(&'a str, &'a str, &'a str),
}

impl<'a> Message<'a> {
    /// Creates a new Message instance.
    pub fn new(prefix: Prefix<'a>, command: Command<'a>, arguments: Vec<&'a str>) -> Self {
        Message {
            prefix: prefix,
            command: command,
            arguments: arguments,
        }
    }
}

impl<'a> UserInfo<'a> {
    pub fn of_nickname(nickname: &'a str) -> Self {
        UserInfo::Nick(nickname)
    }

    pub fn of_nickname_host(nickname: &'a str, host: &'a str) -> Self {
        UserInfo::NickHost(nickname, host)
    }

    pub fn of_nickname_user_host(nickname: &'a str, user: &'a str, host: &'a str) -> Self {
        UserInfo::NickUserHost(nickname, user, host)
    }

    /// Convenience method to get a Prefix::User instance from this UserInfo.
    pub fn to_prefix(self) -> Prefix<'a> {
        Prefix::User(self)
    }

    pub fn nickname(&self) -> &'a str {
        match *self {
            UserInfo::Nick(nick) => nick,
            UserInfo::NickHost(nick, _) => nick,
            UserInfo::NickUserHost(nick, _, _) => nick,
        }
    }

    pub fn host(&self) -> Option<&'a str> {
        match *self {
            UserInfo::Nick(_) => None,
            UserInfo::NickHost(_, host) => Some(host),
            UserInfo::NickUserHost(_, _, host) => Some(host),
        }
    }

    pub fn username(&self) -> Option<&'a str> {
        match *self {
            UserInfo::Nick(_) => None,
            UserInfo::NickHost(_, _) => None,
            UserInfo::NickUserHost(_, user, _) => Some(user),
        }
    }
}

impl<'a> Display for UserInfo<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match *self {
            UserInfo::Nick(nick) => write!(fmt, "{}", nick),
            UserInfo::NickHost(nick, host) => write!(fmt, "{}@{}", nick, host),
            UserInfo::NickUserHost(nick, user, host) => write!(fmt, "{}!{}@{}", nick, user, host),
        }
    }
}

// Is using "Display" to format these for the wire a misuse?
// Should we be using a Write or soemthing instead?
impl<'a> Display for Message<'a> {
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
        let line = Message::new(Prefix::None, PING, vec![]);

        assert_eq!(format!("{}", line), "PING");
    }

    #[test]
    fn server_prefix() {
        let line = Message::new(Prefix::Server("somedude"), PING, vec![]);

        assert_eq!(format!("{}", line), ":somedude PING");
    }

    #[test]
    fn response() {
        let line = Message::new(Prefix::Server("some.server.here"),
                                RPL_WELCOME,
                                vec!["Welcome to the server!"]);

        assert_eq!(format!("{}", line),
                   ":some.server.here 001 :Welcome to the server!");
    }

    #[test]
    fn user_prefix_nickname_only() {
        let line = Message::new(UserInfo::of_nickname("nickname").to_prefix(), PING, vec![]);

        assert_eq!(format!("{}", line), ":nickname PING");
    }

    #[test]
    fn user_prefix_nickname_host() {
        let user_info = UserInfo::of_nickname_host("nickname", "some.host.name");
        let line = Message::new(user_info.to_prefix(), PING, vec![]);

        assert_eq!(format!("{}", line), ":nickname@some.host.name PING");
    }

    #[test]
    fn user_prefix_all_user_info() {
        let user_info = UserInfo::of_nickname_user_host("nickname", "realname", "some.host.name");
        let line = Message::new(user_info.to_prefix(), PING, vec![]);

        assert_eq!(format!("{}", line),
                   ":nickname!realname@some.host.name PING");
    }

    #[test]
    fn command_args() {
        let line = Message::new(Prefix::None, PRIVMSG, vec!["someone", "something"]);

        assert_eq!(format!("{}", line), "PRIVMSG someone something");
    }

    #[test]
    fn command_args_with_long_final_argument() {
        let line = Message::new(Prefix::None,
                                PRIVMSG,
                                vec!["someone", "Hey I love being on IRC"]);

        assert_eq!(format!("{}", line),
                   "PRIVMSG someone :Hey I love being on IRC");
    }

    #[test]
    fn everything() {
        let line = Message::new(Prefix::Server("information"),
                                PRIVMSG,
                                vec!["someone", "something", "Hey I love being on IRC"]);

        assert_eq!(format!("{}", line),
                   ":information PRIVMSG someone something :Hey I love being on IRC");
    }
}