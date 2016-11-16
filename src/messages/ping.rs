use command::commands;
use message::Message;
use message::Prefix;

/// Represents a received PING message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ping<'a> {
    pub arguments: &'a Vec<String>,
}

impl Message {
    pub fn as_ping(&self) -> Option<Ping> {
        if self.command != commands::PING() {
            return None;
        }

        Some(Ping { arguments: &self.arguments })
    }
}

impl<'a> Ping<'a> {
    /// Creates the PONG message corresponding to this PING message.
    pub fn pong(&self) -> Message {
        let arg_copy: Vec<&str> = self.arguments.iter().map(|s| s.as_str()).collect();
        Message::from_strs(Prefix::None, commands::PONG(), arg_copy)
    }
}
