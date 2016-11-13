use command::commands;
use message::Message;
use message::Prefix;
use message::UserInfo;

/// Simple accessor for a received PRIVMSG message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Privmsg<'a> {
    pub from: &'a UserInfo,
    pub to: &'a str,
    pub text: &'a str,
}

impl Message {
    pub fn as_privmsg(&self) -> Option<Privmsg> {
        if self.command != commands::PRIVMSG() {
            return None;
        }
        if self.arguments.len() != 2 {
            warn!("Not parsing message as Privmsg because we expect 2 arguments: {}", self);
            return None;
        }
        let user = match self.prefix {
            Prefix::User(ref u) => u,
            _ => {
                warn!("Not parsing user as Privmsg because we expect prefix of user: {}", self);
                return None;
            }
        };

        Some(Privmsg {
            from: user,
            to: self.arguments.get(0).unwrap(),
            text: self.arguments.get(1).unwrap(),
        })
    }

    pub fn privmsg(to: &str, text: &str) -> Message {
        Message::from_strs(Prefix::None, commands::PRIVMSG(), vec![to, text])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use message::Message;
    use message::UserInfo;

    #[test]
    fn successful() {
        let message = message(":nick!someone@somewhere PRIVMSG #channel :Hey everyone!\r\n");
        let privmsg = message.as_privmsg();

        assert_eq!( privmsg, Some( Privmsg {
            from: &UserInfo::of_nickname_user_host( "nick", "someone", "somewhere" ),
            to: "#channel",
            text: "Hey everyone!",
        } ) );
    }

    #[test]
    fn bad_no_message() {
        let message = message(":nick!someone@somewhere PRIVMSG #channel\r\n");
        assert_eq!( message.as_privmsg(), None);
    }

    #[test]
    fn bad_too_many_arguments() {
        let message = message(":nick!someone@somewhere PRIVMSG #channel #anotherchannel \
                               :message\r\n");
        assert_eq!( message.as_privmsg(), None);
    }

    #[test]
    fn bad_server_prefix() {
        let message = message(":test.irc.com PRIVMSG #channel :message\r\n");
        assert_eq!( message.as_privmsg(), None);
    }

    #[test]
    fn bad_no_prefix() {
        let message = message("PRIVMSG #channel :message\r\n");
        assert_eq!( message.as_privmsg(), None);
    }

    #[test]
    fn bad_not_privmsg() {
        let message = message(":nick!someone@somewhere PING #channel\r\n");
        assert_eq!( message.as_privmsg(), None);
    }

    fn message(message: &str) -> Message {
        let parsed = Message::parse(message.as_bytes());
        match parsed {
            Ok((msg, _)) => msg,
            other => panic!( "Could not parse {}, got result {:?}", message, other ),
        }
    }
}
