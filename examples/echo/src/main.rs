extern crate irc;
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate openssl;

use simplelog::LogLevelFilter;
use simplelog::TermLogger;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::env;
use std::str::FromStr;
use irc::IrcStream;
use irc::Message;
use openssl::ssl::SslConnectorBuilder;
use openssl::ssl::SslMethod;

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
            info!("Connecting to {}:{} over SSL", server, port);
            let ssl_connector = SslConnectorBuilder::new(SslMethod::tls()).unwrap().build();
            let raw_connection = TcpStream::connect((server.as_str(), port)).unwrap();
            let connection = ssl_connector.connect(server.as_str(), raw_connection).unwrap();
            echobot(IrcStream::new(connection), nick, channel);
        }
        "plain" => {
            info!("Connecting to {}:{} over plain IRC", server, port);
            let connection = TcpStream::connect((server.as_str(), port)).unwrap();
            echobot(IrcStream::new(connection), nick, channel);
        }
        _ => panic!("Unrecognised protocol: {}", protocol),
    }
}

fn echobot<S: Read + Write>(mut irc: IrcStream<S>, nick: &str, channel: &str) {
    info!("Connecting with nick {} and joining channel {}",
          nick,
          channel);

    irc.send(&Message::nick(nick)).unwrap();
    irc.send(&Message::user(nick, "Echo Bot")).unwrap();
    irc.send(&Message::join(channel)).unwrap();

    loop {
        let message = irc.next_message().unwrap();
        if let Some(ping) = message.as_ping() {
            info!("Responding to a PING message");
            irc.send(&ping.pong()).unwrap();
        } else if let Some(privmsg) = message.as_privmsg() {
            if privmsg.text.starts_with("!echo ") {
                info!("Responding to an !echo request");
                irc.send(&Message::privmsg(privmsg.to, &privmsg.text[5..])).unwrap()
            }
        }
    }
}
