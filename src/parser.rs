use std;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::string::String;
use std::str;
use std::str::FromStr;
use std::str::Utf8Error;
use std::vec::Vec;
use nom::IResult;
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

#[derive(Debug)]
pub struct ParseError {
    input: Vec<u8>,
}

impl Error for ParseError {
    fn description(&self) -> &str {
        "failed to parse IRC message from line"
    }
}

impl Display for ParseError {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        let as_text = str::from_utf8(&self.input);

        if as_text.is_ok() {
            write!(fmt, "Failed to parse line: [{}]", as_text.unwrap())
        } else {
            write!(fmt,
                   "Failed to parse line and could not interpret as UTF-8, raw bytes: [{:?}]",
                   self.input)
        }
    }
}

pub fn parse_message(input: &[u8]) -> Result<(Message, &[u8]), ParseError> {
    match message(input) {
        IResult::Done(remaining, message) => Ok((message, remaining)),
        _ => Err(ParseError { input: input.to_vec() }),
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

named!(params<Vec<String> >, many0!( preceded!( tag!(" "), alt!( final_param | param ) ) ) );
named!(param<String>, map!( take_while1!(not_space), copy_to_string ) );
named!(final_param<String>, preceded!( tag!(":"), trailing ) );
named!(trailing<String>, map!( take_while!(trailing_char), copy_to_string ) );

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
named!(server_prefix<Prefix>, dbg!( map!( host, |host: &str| { Prefix::Server(host.to_string()) } ) ) );

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

fn copy_to_string(input: &[u8]) -> String {
    String::from_utf8_lossy(input).into_owned()
}

fn to_cow_str(input: &[u8]) -> Result<Cow<str>, Utf8Error> {
    str::from_utf8(input).map(|string| string.into())
}

// This is a horrible hack; just over-match and allow anything
// that can be in an IPv4 address, IPv6 address, or the RFC's
// definition of "hostname".
// TODO: What about internationalized hostnames?
fn is_host_char(c: u8) -> bool {
    is_alphabetic(c) || is_digit(c) || c == b'.' || c == b':' || c == b'-'
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

fn make_word(input: &[u8]) -> Result<Command, str::Utf8Error> {
    str::from_utf8(input).map(|w| Command::of_word(w))
}

fn make_number(input: &[u8]) -> Result<Command, str::Utf8Error> {
    to_cow_str(input).map(|text| u16::from_str(&*text).unwrap_or(123)).map(Command::Number)
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
    match host("hello-world.com".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, "hello-world.com"),
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
        IResult::Done(_, out) => assert_eq!(out, commands::PING()),
        other => panic!("{:?}", other),
    }
}

#[test]
fn command_numeric() {
    match command("004".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, responses::RPL_MYINFO()),
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
                       Message::from_strs(Prefix::None,
                                          commands::PRIVMSG(),
                                          vec!["someone", "Hey what is up"]))
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn message_invalid_utf8() {
    match message(b"PRIVMSG someone :Hey there \xc3\r\n") {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       Message::from_strs(Prefix::None,
                                          commands::PRIVMSG(),
                                          vec!["someone", "Hey there \u{fffd}"]))
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn message_user_prefix() {
    match message(":x!y@z PRIVMSG someone :Hey what is up\r\n".as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       Message::from_strs(Prefix::User(UserInfo::of_nickname_user_host("x".into(), "y".into(), "z".into())),
                                    commands::PRIVMSG(),
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
                       Message::from_strs(Prefix::Server("some.where".into()),
                                          commands::PRIVMSG(),
                                          vec!["someone", "Hey what is up"]))
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn prefix_server() {
    match prefix(":some.where.com ".as_bytes()) {
        IResult::Done(_, out) => assert_eq!(out, Prefix::Server("some.where.com".into())),
        other => panic!("{:?}", other),
    }
}

#[test]
fn prefix_user_prefix_full() {
    match prefix(":x!y@z ".as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out,
                       Prefix::User(UserInfo::of_nickname_user_host("x".into(),
                                                                    "y".into(),
                                                                    "z".into())))
        }
        other => panic!("{:?}", other),
    }
}

#[test]
fn prefix_user_prefix_nickname_only() {
    match prefix(":aperson ".as_bytes()) {
        IResult::Done(_, out) => {
            assert_eq!(out, Prefix::User(UserInfo::of_nickname("aperson".into())))
        }
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
                       Message::from_strs(Prefix::Server("leguin.freenode.net".into()),
                                          responses::RPL_BOUNCE(),
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
