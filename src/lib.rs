#[macro_use]
extern crate nom;

mod command;
mod message;
mod parser;

pub use command::Command;
pub use command::responses;
pub use command::commands;
pub use message::Message;
pub use message::Prefix;
pub use message::UserInfo;

use parser::parse_message;

impl<'a> Message<'a> {
    pub fn parse(input: &'a [u8]) -> Result<(Message, &'a [u8]), ()> {
        parse_message(input)
    }
}

#[test]
fn simple_parse() {
    match Message::parse("PING 12345\r\nsome other content".as_bytes()) {
        Ok((msg, remaining)) => {
            assert_eq!(msg,
                       Message::new(Prefix::None, commands::PING, vec!["12345"]));
            assert_eq!(remaining, "some other content".as_bytes());
        }
        other => panic!("{:?}", other),
    }
}
