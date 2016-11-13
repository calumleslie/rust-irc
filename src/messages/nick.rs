use command::commands;
use message::Message;
use message::Prefix;

impl Message {
    pub fn nick(nick: &str) -> Message {
        Message::from_strs(Prefix::None, commands::NICK(), vec![nick])
    }
}
