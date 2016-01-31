#[macro_use]
extern crate nom;

mod command;

pub use command::Command;
pub use command::responses;
pub use command::commands;
use std::fmt::Display;
use std::fmt::Formatter;
use std::vec::Vec;

#[derive(Debug,Clone, PartialEq, Eq)]
pub struct Message<'a> {
    prefix: Prefix<'a>,
    command: Command<'a>,
    arguments: Vec<&'a str>,
}

#[derive(Debug,Clone, PartialEq, Eq)]
pub enum Prefix<'a> {
    None,
    Server(&'a str),
    User(UserInfo<'a>),
}

#[derive(Debug,Clone, PartialEq, Eq)]
pub struct UserInfo<'a> {
    nickname: &'a str,
    host: Option<HostInfo<'a>>,
}

#[derive(Debug,Clone, PartialEq, Eq)]
pub struct HostInfo<'a> {
    user: Option<&'a str>,
    host: &'a str,
}

impl<'a> Message<'a> {
    pub fn new(prefix: Prefix<'a>, command: &'a str, arguments: Vec<&'a str>) -> Self {
        Message {
            prefix: prefix,
            command: Command::of_word(command),
            arguments: arguments,
        }
    }
}

impl<'a> UserInfo<'a> {
    pub fn of_nickname(nickname: &'a str) -> Self {
        UserInfo {
            nickname: nickname,
            host: None,
        }
    }

    pub fn of_nickname_host(nickname: &'a str, host: &'a str) -> Self {
        UserInfo {
            nickname: nickname,
            host: Some(HostInfo {
                host: host,
                user: None,
            }),
        }
    }

    pub fn of_nickname_user_host(nickname: &'a str, user: &'a str, host: &'a str) -> Self {
        UserInfo {
            nickname: nickname,
            host: Some(HostInfo {
                host: host,
                user: Some(user),
            }),
        }
    }
}

impl<'a> Display for UserInfo<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        try!(write!(fmt, "{}", self.nickname));

        match self.host {
            None => Ok(()),
            Some(ref host_info) => {
                match host_info.user {
                    Some(user) => write!(fmt, "!{}@{}", user, host_info.host),
                    None => write!(fmt, "@{}", host_info.host),
                }
            }
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


    #[test]
    fn command_only() {
        let line = Message {
            prefix: Prefix::None,
            command: Command::of_word("PING"),
            arguments: vec![],
        };

        assert_eq!(format!("{}", line), "PING");
    }

    #[test]
    fn server_prefix() {
        let line = Message {
            prefix: Prefix::Server("somedude"),
            command: Command::of_word("PING"),
            arguments: vec![],
        };

        assert_eq!(format!("{}", line), ":somedude PING");
    }

    #[test]
    fn response() {
        let line = Message {
            prefix: Prefix::Server("some.server.here"),
            command: responses::RPL_WELCOME,
            arguments: vec!["Welcome to the server!"],
        };

        assert_eq!(format!("{}", line),
                   ":some.server.here 001 :Welcome to the server!");
    }

    #[test]
    fn user_prefix_nickname_only() {
        let line = Message {
            prefix: Prefix::User(UserInfo::of_nickname("nickname")),
            command: Command::of_word("PING"),
            arguments: vec![],
        };

        assert_eq!(format!("{}", line), ":nickname PING");
    }

    #[test]
    fn user_prefix_nickname_host() {
        let line = Message {
            prefix: Prefix::User(UserInfo::of_nickname_host("nickname", "some.host.name")),
            command: Command::of_word("PING"),
            arguments: vec![],
        };

        assert_eq!(format!("{}", line), ":nickname@some.host.name PING");
    }

    #[test]
    fn user_prefix_all_user_info() {
        let line = Message {
            prefix: Prefix::User(UserInfo::of_nickname_user_host("nickname",
                                                                 "realname",
                                                                 "some.host.name")),
            command: Command::of_word("PING"),
            arguments: vec![],
        };

        assert_eq!(format!("{}", line),
                   ":nickname!realname@some.host.name PING");
    }

    #[test]
    fn command_args() {
        let line = Message {
            prefix: Prefix::None,
            command: commands::PRIVMSG,
            arguments: vec!["someone", "something"],
        };

        assert_eq!(format!("{}", line), "PRIVMSG someone something");
    }

    #[test]
    fn command_args_with_long_final_argument() {
        let line = Message {
            prefix: Prefix::None,
            command: Command::of_word("PRIVMSG"),
            arguments: vec!["someone", "Hey I love being on IRC"],
        };

        assert_eq!(format!("{}", line),
                   "PRIVMSG someone :Hey I love being on IRC");
    }

    #[test]
    fn everything() {
        let line = Message {
            prefix: Prefix::Server("information"),
            command: Command::of_word("PRIVMSG"),
            arguments: vec!["someone", "something", "Hey I love being on IRC"],
        };

        assert_eq!(format!("{}", line),
                   ":information PRIVMSG someone something :Hey I love being on IRC");
    }
}
