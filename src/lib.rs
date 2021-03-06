//! Experimental library for working with the IRC protocol.
//!
//! _Very_ work-in-progress so if you do decide to use this, please expect breakages. In particular
//! I suspect that the IRC message parsing is not complete.
//!
//! See `examples/echo` for a simple bot which sits on a channel and responds to `!echo` commands.

// I'd happily have Clippy on all the time but it's nightly-only so it's hidden behind a feature
// flag.
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate log;

#[macro_use]
extern crate nom;

extern crate openssl;

mod command;
mod irc_stream;
mod message;
mod parser;

pub mod messages;
pub use command::Command;
pub use command::responses;
pub use command::commands;
pub use message::Message;
pub use message::Prefix;
pub use message::UserInfo;
pub use irc_stream::IrcStream;
pub use parser::ParseError;

use parser::parse_message;

impl Message {
    pub fn parse(input: &[u8]) -> Result<(Message, &[u8]), ParseError> {
        parse_message(input)
    }
}

#[test]
fn simple_parse() {
    match Message::parse("PING 12345\r\nsome other content".as_bytes()) {
        Ok((msg, remaining)) => {
            assert_eq!(msg,
                       Message::from_strs(Prefix::None, commands::PING(), vec!["12345"]));
            assert_eq!(remaining, "some other content".as_bytes());
        }
        other => panic!("{:?}", other),
    }
}
