#[macro_use]
extern crate log;

#[macro_use]
extern crate nom;

mod client;
mod command;
mod message;
mod parser;

pub use client::connect;

pub use command::Command;
pub use command::responses;
pub use command::commands;
pub use message::Message;
pub use message::Prefix;
pub use message::UserInfo;

use parser::parse_message;

impl Message {
    pub fn parse(input: &[u8]) -> Result<(Message, &[u8]), ()> {
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
