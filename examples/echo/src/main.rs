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
use std::net::ToSocketAddrs;
use std::env;
use std::str::FromStr;
use irc::IrcStream;
use irc::Message;
use irc::commands;
use irc::Prefix;
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
        },
        "plain" => {
            info!("Connecting to {}:{} over plain IRC", server, port);
            let connection = TcpStream::connect((server.as_str(), port)).unwrap();
            echobot(IrcStream::new(connection), nick, channel);
        },
        _ => panic!("Unrecognised protocol: {}", protocol)
    }
}

fn echobot<S: Read + Write>(mut irc: IrcStream<S>, nick: &str, channel: &str) {
    info!("Connecting with nick {} and joining channel {}", nick, channel);

    irc.send(&Message::from_strs(Prefix::None, commands::NICK(), vec![nick])).unwrap();
    irc.send(&Message::from_strs(Prefix::None,
                                  commands::USER(),
                                  vec![nick, "0", "*", "This is pretty sweet assuming it works"]))
        .unwrap();

    irc.send(&Message::from_strs(Prefix::None, commands::JOIN(), vec![channel.into()]))
        .unwrap();

    loop {
        let message = irc.next_message().unwrap();
        if message.command == commands::PING() {
            info!("Responding to a PING message");
            irc.send(&Message::new(Prefix::None, commands::PONG(), message.arguments)).unwrap();
        } else if message.command == commands::PRIVMSG() {
            if message.arguments.len() != 2 {
                continue;
            }

            let sender = message.arguments.get(0).unwrap().as_str();
            let line = message.arguments.get(1).unwrap().as_str();

            if line.starts_with("!echo ") {
                info!("Responding to an !echo request");
                irc.send(&Message::from_strs(Prefix::None,
                                              commands::PRIVMSG(),
                                              vec![sender, &line[5..]]))
                    .unwrap();
            }

        }
    }
}
