use std::fmt::Display;
use std::fmt::Formatter;
use std;

/// An IRC command. These can either be a sequence of letters
/// (which I'm calling "word") or a numeric value.
/// Note that creating one of these directly will
/// bypass validation and cause you to have a Bad Time.
#[derive(Debug,Clone, PartialEq, Eq)]
pub enum Command {
    Word(String),
    Number(u16),
}

impl Command {
    /// Creates a Command::Word validated to ensure it is a valid IRC command.
    /// Only validates that the command is made up of valid characters, not that
    /// it's a command that appears in any RFC.
    ///
    /// # Panics
    ///
    /// Will panic if `word` has any characters outside of `[a-zA-Z]`.
    pub fn of_word(word: &str) -> Self {
        for (i, c) in word.chars().enumerate() {
            // As deep as my hatred for regexes goes this is getting a bit silly
            let codepoint = c as u32;
            let is_lowercase_letter = codepoint >= 0x41 && codepoint <= 0x5A;
            let is_uppercase_letter = codepoint >= 0x61 && codepoint <= 0x7A;
            assert!(is_lowercase_letter || is_uppercase_letter,
                    "Word IRC commands must contain only chars A-Za-z but [{}] index {} is [{}]",
                    word,
                    i,
                    c);
        }

        Command::Word(word.into())
    }

    /// Creates a Command::Number validated to ensure it is a valid IRC command.
    /// Only validates that the command is made up of a valid number, not that
    /// it's a command that appears in any RFC.
    ///
    /// # Panics
    ///
    /// Will panic if `number` cannot be represented as a 3-digit number (i.e. if it is
    /// greater than 999).
    pub fn of_number(number: u16) -> Self {
        assert!(number <= 999,
                "Numeric IRC commands must be representable as a 3-digit number but got {}",
                number);
        Command::Number(number)
    }
}

/// Constants for the command types documented in RFC 8212
#[allow(non_snake_case)]
pub mod commands {
    use super::Command;

    macro_rules! commands {
        ( $( $x:ident ),* ) => {
            $(
                pub fn $x() -> Command {
                    Command::of_word(stringify!($x))
                }
            )*
        };
    }

    commands!(ADMIN,
              AWAY,
              CONNECT,
              DIE,
              ERROR,
              INFO,
              INVITE,
              ISON,
              JOIN,
              KICK,
              KILL,
              LINKS,
              LIST,
              LUSERS,
              MODE,
              MOTD,
              NAMES,
              NICK,
              NOTICE,
              OPER,
              PART,
              PASS,
              PING,
              PONG,
              PRIVMSG,
              QUIT,
              REHASH,
              RESTART,
              SERVICE,
              SERVLIST,
              SQUERY,
              SQUIT,
              STATS,
              SUMMON,
              TIME,
              TOPIC,
              TRACE,
              USER,
              USERHOST,
              USERS,
              VERSION,
              WALLOPS,
              WHO,
              WHOIS,
              WHOWAS);
}

/// Constants for all of the response types documented in RFC 8212
#[allow(non_snake_case)]
pub mod responses {
    use super::Command;

    macro_rules! response {
        ( $number:expr , $name:ident ) => {
            pub fn $name() -> Command {
                Command::Number($number)
            }
        };
    }

    response!(1, RPL_WELCOME);
    response!(2, RPL_YOURHOST);
    response!(3, RPL_CREATED);
    response!(4, RPL_MYINFO);
    response!(5, RPL_BOUNCE);
    response!(200, RPL_TRACELINK);
    response!(201, RPL_TRACECONNECTING);
    response!(202, RPL_TRACEHANDSHAKE);
    response!(203, RPL_TRACEUNKNOWN);
    response!(204, RPL_TRACEOPERATOR);
    response!(205, RPL_TRACEUSER);
    response!(206, RPL_TRACESERVER);
    response!(207, RPL_TRACESERVICE);
    response!(208, RPL_TRACENEWTYPE);
    response!(209, RPL_TRACECLASS);
    response!(210, RPL_TRACERECONNECT);
    response!(211, RPL_STATSLINKINFO);
    response!(212, RPL_STATSCOMMANDS);
    response!(219, RPL_ENDOFSTATS);
    response!(221, RPL_UMODEIS);
    response!(233, RPL_SERVICE);
    response!(234, RPL_SERVLIST);
    response!(235, RPL_SERVLISTEND);
    response!(242, RPL_STATSUPTIME);
    response!(243, RPL_STATSOLINE);
    response!(250, RPL_STATSDLINE);
    response!(251, RPL_LUSERCLIENT);
    response!(252, RPL_LUSEROP);
    response!(253, RPL_LUSERUNKNOWN);
    response!(254, RPL_LUSERCHANNELS);
    response!(255, RPL_LUSERME);
    response!(256, RPL_ADMINME);
    response!(259, RPL_ADMINEMAIL);
    response!(261, RPL_TRACELOG);
    response!(262, RPL_TRACEEND);
    response!(263, RPL_TRYAGAIN);
    response!(301, RPL_AWAY);
    response!(302, RPL_USERHOST);
    response!(303, RPL_ISON);
    response!(305, RPL_UNAWAY);
    response!(306, RPL_NOWAWAY);
    response!(311, RPL_WHOISUSER);
    response!(312, RPL_WHOISSERVER);
    response!(313, RPL_WHOISOPERATOR);
    response!(314, RPL_WHOWASUSER);
    response!(315, RPL_ENDOFWHO);
    response!(317, RPL_WHOISIDLE);
    response!(318, RPL_ENDOFWHOIS);
    response!(319, RPL_WHOISCHANNELS);
    response!(321, RPL_LISTSTART);
    response!(322, RPL_LIST);
    response!(323, RPL_LISTEND);
    response!(324, RPL_CHANNELMODEIS);
    response!(325, RPL_UNIQOPIS);
    response!(331, RPL_NOTOPIC);
    response!(332, RPL_TOPIC);
    response!(341, RPL_INVITING);
    response!(342, RPL_SUMMONING);
    response!(346, RPL_INVITELIST);
    response!(347, RPL_ENDOFINVITELIST);
    response!(348, RPL_EXCEPTLIST);
    response!(349, RPL_ENDOFEXCEPTLIST);
    response!(351, RPL_VERSION);
    response!(352, RPL_WHOREPLY);
    response!(353, RPL_NAMREPLY);
    response!(364, RPL_LINKS);
    response!(365, RPL_ENDOFLINKS);
    response!(366, RPL_ENDOFNAMES);
    response!(367, RPL_BANLIST);
    response!(368, RPL_ENDOFBANLIST);
    response!(369, RPL_ENDOFWHOWAS);
    response!(371, RPL_INFO);
    response!(372, RPL_MOTD);
    response!(374, RPL_ENDOFINFO);
    response!(375, RPL_MOTDSTART);
    response!(376, RPL_ENDOFMOTD);
    response!(381, RPL_YOUREOPER);
    response!(382, RPL_REHASHING);
    response!(383, RPL_YOURESERVICE);
    response!(384, RPL_MYPORTIS);
    response!(391, RPL_TIME);
    response!(392, RPL_USERSSTART);
    response!(393, RPL_USERS);
    response!(394, RPL_ENDOFUSERS);
    response!(395, RPL_NOUSERS);
    response!(401, ERR_NOSUCHNICK);
    response!(402, ERR_NOSUCHSERVER);
    response!(403, ERR_NOSUCHCHANNEL);
    response!(404, ERR_CANNOTSENDTOCHAN);
    response!(405, ERR_TOOMANYCHANNELS);
    response!(406, ERR_WASNOSUCHNICK);
    response!(407, ERR_TOOMANYTARGETS);
    response!(408, ERR_NOSUCHSERVICE);
    response!(409, ERR_NOORIGIN);
    response!(411, ERR_NORECIPIENT);
    response!(412, ERR_NOTEXTTOSEND);
    response!(413, ERR_NOTOPLEVEL);
    response!(414, ERR_WILDTOPLEVEL);
    response!(415, ERR_BADMASK);
    response!(421, ERR_UNKNOWNCOMMAND);
    response!(422, ERR_NOMOTD);
    response!(423, ERR_NOADMININFO);
    response!(424, ERR_FILEERROR);
    response!(431, ERR_NONICKNAMEGIVEN);
    response!(432, ERR_ERRONEUSNICKNAME);
    response!(433, ERR_NICKNAMEINUSE);
    response!(436, ERR_NICKCOLLISION);
    response!(437, ERR_UNAVAILRESOURCE);
    response!(441, ERR_USERNOTINCHANNEL);
    response!(442, ERR_NOTONCHANNEL);
    response!(443, ERR_USERONCHANNEL);
    response!(444, ERR_NOLOGIN);
    response!(445, ERR_SUMMONDISABLED);
    response!(446, ERR_USERSDISABLED);
    response!(451, ERR_NOTREGISTERED);
    response!(461, ERR_NEEDMOREPARAMS);
    response!(462, ERR_ALREADYREGISTRED);
    response!(463, ERR_NOPERMFORHOST);
    response!(464, ERR_PASSWDMISMATCH);
    response!(465, ERR_YOUREBANNEDCREEP);
    response!(466, ERR_YOUWILLBEBANNED);
    response!(467, ERR_KEYSET);
    response!(471, ERR_CHANNELISFULL);
    response!(472, ERR_UNKNOWNMODE);
    response!(473, ERR_INVITEONLYCHAN);
    response!(474, ERR_BANNEDFROMCHAN);
    response!(475, ERR_BADCHANNELKEY);
    response!(476, ERR_BADCHANMASK);
    response!(477, ERR_NOCHANMODES);
    response!(478, ERR_BANLISTFULL);
    response!(481, ERR_NOPRIVILEGES);
    response!(482, ERR_CHANOPRIVSNEEDED);
    response!(483, ERR_CANTKILLSERVER);
    response!(484, ERR_RESTRICTED);
    response!(485, ERR_UNIQOPPRIVSNEEDED);
    response!(491, ERR_NOOPERHOST);
    response!(492, ERR_NOSERVICEHOST);
    response!(501, ERR_UMODEUNKNOWNFLAG);
    response!(502, ERR_USERSDONTMATCH);
}

impl Display for Command {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match *self {
            Command::Word(ref word) => write!(fmt, "{}", word),
            Command::Number(number) => write!(fmt, "{:0>3}", number),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn of_number_valid() {
        Command::of_number(123);
    }

    #[test]
    #[should_panic]
    fn of_number_too_large() {
        Command::of_number(1000);
    }

    #[test]
    fn of_word_valid() {
        Command::of_word("PrIvMsG");
    }

    #[test]
    #[should_panic]
    fn of_word_non_alpha_chars() {
        Command::of_word("PR1VMSG");
    }

    #[test]
    fn fmt_number_unpadded() {
        assert_eq!(format!("{}", Command::of_number(123)), "123");
    }

    #[test]
    fn fmt_number_padded() {
        assert_eq!(format!("{}", Command::of_number(1)), "001");
    }

    #[test]
    fn commands() {
        assert_eq!(commands::PRIVMSG(), Command::of_word("PRIVMSG"));
    }

    #[test]
    fn replies() {
        assert_eq!(responses::RPL_BOUNCE(), Command::of_number(5));
    }

}
