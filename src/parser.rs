
use std::str;
use std::str::FromStr;
use std::vec::Vec;
use nom::IResult;
use nom::Err;
use nom::is_digit;
use nom::is_alphabetic;
use command::Command;
use message::Message;
use message::Prefix;
use message::UserInfo;

#[cfg(test)]
use nom::GetInput;

#[cfg(test)]
use command::commands;

#[cfg(test)]
use command::responses;

pub fn parse_message(input: &[u8]) -> Result<(Message, &[u8]), ()> {
    match message(input) {
        IResult::Done(remaining, message) => return Ok((message, remaining)),
        _ => return Err(()),
    }
}

named!(message<Message>, chain!(
  prefix: prefix? ~
  command: command ~
  params: params ~
  tag!("\r\n"), ||{
    Message::new( prefix.unwrap_or( Prefix::None ), command, params )
  }
)) ;

named!(params<Vec<&str> >, many0!( preceded!( tag!(" "), alt!( final_param | param ) ) ) );
named!(param<&str>, map_res!( take_while1!(not_space), str::from_utf8 ) );
named!(final_param<&str>, preceded!( tag!(":"), trailing ) );
named!(trailing<&str>, map_res!( take_while!(trailing_char), str::from_utf8 ) );

named!(command<Command>, alt!( word_command | numeric_command ) );
named!(word_command<Command>, map_res!( take_while1!(is_alphabetic), make_word) );
// TODO: This does not limit values to 3 digits, and no validation in make_number.
named!(numeric_command<Command>, map_res!( take_while1!(is_digit), make_number ) );

// This consumes the final space too, a simple way of testing we eat everything
// up to the delimiter.
named!(prefix<Prefix>, preceded!( tag!( ":" ), alt!(
  complete!( terminated!( user_prefix, tag!( " " ) ) )
| complete!( terminated!( server_prefix, tag!( " " ) ) ) ) ) );

named!(user_prefix<Prefix>, map!(user_info, Prefix::User ) );
named!(server_prefix<Prefix>, dbg!( map!( host, Prefix::Server ) ) );

// Use of complete! here stops the earlier patterns returning Incomplete.
named!(user_info<UserInfo>, alt!(
  complete!( chain!( n: nickname ~ tag!("!") ~ u: username ~ tag!("@") ~ h: host, ||{
    UserInfo::of_nickname_user_host( n, u, h )
  } ) )
| complete!( chain!( n: nickname ~ tag!("@") ~ h: host, ||{ UserInfo::of_nickname_host( n, h ) } ) )
| map!( nickname, |value|{ UserInfo::of_nickname( value ) } )
));

// Note: This allows nicknames with invalid first characters
named!(nickname<&str>, map_res!( take_while1!(is_nickname_char), str::from_utf8));
named!(username<&str>, map_res!( take_while1!(is_username_char), str::from_utf8));
named!(host<&str>, map_res!( take_while1!(is_host_char), str::from_utf8));

// This is a horrible hack; just over-match and allow anything
// that can be in an IPv4 address, IPv6 address, or the RFC's
// definition of "hostname".
// TODO: What about internationalized hostnames?
fn is_host_char(c: u8) -> bool {
    is_alphabetic(c) || is_digit(c) || c == b'.' || c == b':'
}

// Everything except NUL, CR, LF, and " "
fn not_space(c: u8) -> bool {
    (c != 0) && (c != b'\r') && (c != b'\n') && (c != b' ')
}

// "[", "]", "\", "`", "_", "^", "{", "|", "}"
fn is_special(c: u8) -> bool {
    (c == b'[') || (c == b']') || (c == b'\\') || (c == b'`') || (c == b'_') || (c == b'^') ||
    (c == b'{') || (c == b'|') || (c == b'}')
}

fn trailing_char(c: u8) -> bool {
    (c == b' ') || not_space(c)
}

fn make_word<'a>(input: &'a [u8]) -> Result<Command<'a>, str::Utf8Error> {
    str::from_utf8(input).map(|w| Command::Word(w))
}

fn make_number<'a>(input: &'a [u8]) -> Result<Command<'a>, str::Utf8Error> {
    str::from_utf8(input).map(|text| u16::from_str(text).unwrap_or(123)).map(|n| Command::Number(n))
}

fn is_nickname_char(c: u8) -> bool {
    is_alphabetic(c) || is_special(c) || is_digit(c) || c == b'-'
}

// Not NUL, CR, LF, " " and "@"
fn is_username_char(c: u8) -> bool {
    (c != 0) && (c != b'\r') && (c != b'\n') && (c != b' ') && (c != b'@')
}

#[test]
fn host_hostname() {
    match host("hello.world.com".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, "hello.world.com"),
        other => panic!("{:?}", other),
    }
}

#[test]
fn host_ipv4() {
    match host("192.168.0.1".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, "192.168.0.1"),
        other => panic!("{:?}", other),
    }
}

#[test]
fn host_ipv6() {
    match host("2001:db8:85a3::8a2e:370:7334".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, "2001:db8:85a3::8a2e:370:7334"),
        other => panic!("{:?}", other),
    }
}

#[test]
fn host_user_info_does_not_match() {
    let result = host("hello!user@place".as_bytes());

    assert!(result.remaining_input().unwrap().len() > 0,
            "Expected unfinished matching but got {:?}",
            result);
}

#[test]
fn command_word() {
    match command("PING".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, commands::PING),
        other => panic!("{:?}", other),
    }
}

#[test]
fn command_numeric() {
    match command("004".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, responses::RPL_MYINFO),
        other => panic!("{:?}", other),
    }
}

#[test]
fn final_param_with_content() {
    match final_param(":content can contain spaces and ':'".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, "content can contain spaces and ':'"),
        other => panic!("{:?}", other),
    }
}

#[test]
fn params_multiple() {
    match params(" here are some :parameters including a long final one".as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       vec!["here", "are", "some", "parameters including a long final one"])
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn params_no_trailing() {
    match params(" here are some parameters".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, vec!["here", "are", "some", "parameters"]),
        other => panic!("{:?}", other),
    }
}

#[test]
fn message_no_prefix() {
    match message("PRIVMSG someone :Hey what is up\r\n".as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       Message::new(Prefix::None,
                                    commands::PRIVMSG,
                                    vec!["someone", "Hey what is up"]))
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn message_user_prefix() {
    match message(":x!y@z PRIVMSG someone :Hey what is up\r\n".as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       Message::new(Prefix::User(UserInfo::of_nickname_user_host("x", "y", "z")),
                                    commands::PRIVMSG,
                                    vec!["someone", "Hey what is up"]))
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn message_server_prefix() {
    match message(":some.where PRIVMSG someone :Hey what is up\r\n".as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       Message::new(Prefix::Server("some.where"),
                                    commands::PRIVMSG,
                                    vec!["someone", "Hey what is up"]))
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn prefix_server() {
    match prefix(":some.where.com ".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, Prefix::Server("some.where.com")),
        other => panic!("{:?}", other),
    }
}

#[test]
fn prefix_user_prefix_full() {
    match prefix(":x!y@z ".as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       Prefix::User(UserInfo::of_nickname_user_host("x", "y", "z")))
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn prefix_user_prefix_nickname_only() {
    match prefix(":aperson ".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, Prefix::User(UserInfo::of_nickname("aperson"))),
        other => panic!("{:?}", other),
    }
}

#[test]
fn real_message_complex() {
    match message(":leguin.freenode.net 005 zootmbot CHANTYPES=# EXCEPTS INVEX \
                   CHANMODES=eIbq,k,flj,CFLMPQScgimnprstz CHANLIMIT=#:120 PREFIX=(ov)@+ \
                   MAXLIST=bqeI:100 MODES=4 NETWORK=freenode KNOCK STATUSMSG=@+ CALLERID=g :are \
                   supported by this server\r\n"
                      .as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       Message::new(Prefix::Server("leguin.freenode.net"),
                                    responses::RPL_BOUNCE,
                                    vec!["zootmbot",
                                         "CHANTYPES=#",
                                         "EXCEPTS",
                                         "INVEX",
                                         "CHANMODES=eIbq,k,flj,CFLMPQScgimnprstz",
                                         "CHANLIMIT=#:120",
                                         "PREFIX=(ov)@+",
                                         "MAXLIST=bqeI:100",
                                         "MODES=4",
                                         "NETWORK=freenode",
                                         "KNOCK",
                                         "STATUSMSG=@+",
                                         "CALLERID=g",
                                         "are supported by this server"]))
        }
        other => panic!("{:?}", other),
    }
}
