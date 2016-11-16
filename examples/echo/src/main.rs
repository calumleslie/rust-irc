extern crate irc;
#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::LogLevelFilter;
use simplelog::TermLogger;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::env;
use std::str::FromStr;
use irc::IrcStream;
use irc::Message;

fn main() {
    TermLogger::init(LogLevelFilter::Trace).unwrap();

    let args: Vec<String> = env::args().skip(1).collect();
    assert_eq!(args.len(),
               5,
               "Provide 5 arguments: [server] [port] [plain|ssl] [nick] [channel]");

    let server = args.get(0).unwrap();
    let port = u16::from_str(args.get(1).unwrap()).unwrap();
    let protocol = args.get(2).unwrap();
    let nick = args.get(3).unwrap();
    let channel = args.get(4).unwrap();

    match protocol.as_str() {
        "ssl" => {
            let irc = IrcStream::connect_ssl(server.as_str(), port).unwrap();
            echobot(irc, nick, channel).unwrap();
        }
        "plain" => {
            let irc = IrcStream::connect(server.as_str(), port).unwrap();
            echobot(irc, nick, channel).unwrap();
        }
        _ => panic!("Unrecognised protocol: {}", protocol),
    }
}

fn echobot<S: Read + Write>(mut irc: IrcStream<S>, nick: &str, channel: &str) -> io::Result<()> {
    info!("Connecting with nick {} and joining channel {}",
          nick,
          channel);

    irc.send(&Message::nick(nick))?;
    irc.send(&Message::user(nick, "Echo Bot"))?;
    irc.send(&Message::join(channel))?;

    loop {
        let message = irc.next_message()?;
        if let Some(ping) = message.as_ping() {
            info!("Responding to a PING message");
            irc.send(&ping.pong()).unwrap();
        } else if let Some(privmsg) = message.as_privmsg() {
            if privmsg.text.starts_with("!echo ") {
                info!("Responding to an !echo request");
                irc.send(&Message::privmsg(privmsg.to, &privmsg.text[5..]))?
            }
        }
    }
}
