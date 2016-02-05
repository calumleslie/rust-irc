
use std::str;
use std::str::FromStr;
use nom::IResult;
use nom::GetInput;
use command::Command;
use command::commands;
use command::responses;
// message    =  [ ":" prefix SPACE ] command [ params ] crlf
//     prefix     =  servername / ( nickname [ [ "!" user ] "@" host ] )
//     command    =  1*letter / 3digit
//     params     =  *14( SPACE middle ) [ SPACE ":" trailing ]
//                =/ 14( SPACE middle ) [ SPACE [ ":" ] trailing ]

//     nospcrlfcl =  %x01-09 / %x0B-0C / %x0E-1F / %x21-39 / %x3B-FF
//                     ; any octet except NUL, CR, LF, " " and ":"
//     middle     =  nospcrlfcl *( ":" / nospcrlfcl )
//     trailing   =  *( ":" / " " / nospcrlfcl )

//     SPACE      =  %x20        ; space character
//     crlf       =  %x0D %x0A   ; "carriage return" "linefeed"




// Kalt                         Informational                      [Page 6]


// RFC 2812          Internet Relay Chat: Client Protocol        April 2000


//    NOTES:
//       1) After extracting the parameter list, all parameters are equal
//          whether matched by <middle> or <trailing>. <trailing> is just a
//          syntactic trick to allow SPACE within the parameter.

//       2) The NUL (%x00) character is not special in message framing, and
//          basically could end up inside a parameter, but it would cause
//          extra complexities in normal C string handling. Therefore, NUL
//          is not allowed within messages.

//    Most protocol messages specify additional semantics and syntax for
//    the extracted parameter strings dictated by their position in the
//    list.  For example, many server commands will assume that the first
//    parameter after the command is the list of targets, which can be
//    described with:

//   target     =  nickname / server
//   msgtarget  =  msgto *( "," msgto )
//   msgto      =  channel / ( user [ "%" host ] "@" servername )
//   msgto      =/ ( user "%" host ) / targetmask
//   msgto      =/ nickname / ( nickname "!" user "@" host )
//   channel    =  ( "#" / "+" / ( "!" channelid ) / "&" ) chanstring
//                 [ ":" chanstring ]
//   servername =  hostname
fn is_letter(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z')
}

fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn make_word<'a>(input: &'a [u8]) -> Result<Command<'a>, str::Utf8Error> {
    str::from_utf8(input).map(|w| Command::Word(w))
}

fn make_number<'a>(input: &'a [u8]) -> Result<Command<'a>, str::Utf8Error> {
    str::from_utf8(input).map(|text| u16::from_str(text).unwrap()).map(|n| Command::Number(n))
}

named!(command<Command>, alt!( word_command | numeric_command ) );
named!(word_command<Command>, map_res!( take_while1!(is_letter), make_word) );
// TODO: This does not limit values to 3 digits, and no validation in make_number.
named!(numeric_command<Command>, map_res!( take_while!(is_digit), make_number ) );

// This is a horrible hack; just over-match and allow anything
// that can be in an IPv4 address, IPv6 address, or the RFC's
// definition of "hostname".
// TODO: What about internationalized hostnames?
fn is_host_char(c: u8) -> bool {
    is_letter(c) || is_digit(c) || c == b'.' || c == b':'
}

named!(host<&str>, map_res!( take_while1!(is_host_char), str::from_utf8));


//   nickname   =  ( letter / special ) *8( letter / digit / special / "-" )
//   targetmask =  ( "$" / "#" ) mask
//                   ; see details on allowed masks in section 3.3.1
//   chanstring =  %x01-07 / %x08-09 / %x0B-0C / %x0E-1F / %x21-2B
//   chanstring =/ %x2D-39 / %x3B-FF
//                   ; any octet except NUL, BELL, CR, LF, " ", "," and ":"
//   channelid  = 5( %x41-5A / digit )   ; 5( A-Z / 0-9 )


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

// Kalt                         Informational                      [Page 7]


// RFC 2812          Internet Relay Chat: Client Protocol        April 2000


// Other parameter syntaxes are:

//   user       =  1*( %x01-09 / %x0B-0C / %x0E-1F / %x21-3F / %x41-FF )
//                   ; any octet except NUL, CR, LF, " " and "@"
//   key        =  1*23( %x01-05 / %x07-08 / %x0C / %x0E-1F / %x21-7F )
//                   ; any 7-bit US_ASCII character,
//                   ; except NUL, CR, LF, FF, h/v TABs, and " "
//   letter     =  %x41-5A / %x61-7A       ; A-Z / a-z
//   digit      =  %x30-39                 ; 0-9
//   hexdigit   =  digit / "A" / "B" / "C" / "D" / "E" / "F"
//   special    =  %x5B-60 / %x7B-7D
//                    ; "[", "]", "\", "`", "_", "^", "{", "|", "}"
