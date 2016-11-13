use command::commands;
use message::Message;
use message::Prefix;

impl Message {
    pub fn user(user: &str, realname: &str) -> Message {
        Message::from_strs(Prefix::None,
                           commands::USER(),
                           vec![user, "0", "*", realname])
    }
}
