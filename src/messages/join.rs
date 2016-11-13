use command::commands;
use message::Message;
use message::Prefix;

impl Message {
    pub fn join(channel: &str) -> Message {
        Message::from_strs(Prefix::None, commands::JOIN(), vec![channel])
    }
}
