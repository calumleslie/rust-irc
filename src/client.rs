use std::fmt::Display;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use std::io;
use std::str;
use std::time::Duration;
use std::thread;
use log::LogLevel::Warn;
use command::Command;
use command::commands;
use message::Message;
use message::Prefix;
use message::UserInfo;

pub fn connect<A: ToSocketAddrs + Display>(server: A) -> io::Result<()> {
    info!("Connecting to server at {}", server);

    let read_side = try!(TcpStream::connect(server));
    let write_side = try!(read_side.try_clone());

    thread::spawn(move || {
        let mut writer = write_side;

        thread::sleep(Duration::from_secs(5));

        write_message(&mut writer,
                      &Message::new(Prefix::None, commands::NICK, vec!["zootmbot"]));
        writer.flush();
        write_message(&mut writer,
                      &Message::new(Prefix::None,
                                    commands::USER,
                                    vec!["zootmbot",
                                         "0",
                                         "*",
                                         "This is pretty sweet assuming it works"]));
        writer.flush();
        write_message(&mut writer,
                      &Message::new(Prefix::None, commands::JOIN, vec!["#superhugs"]));
        writer.flush();

        // info!( "Bailing writer thread" );
    });

    thread::spawn(move || {
        // TODO: Feels suboptimal.
        let mut reader = BufReader::new(read_side);
        let mut buf: Vec<u8> = Vec::new();
        // fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize>

        let mut read_result = reader.read_until(b'\n', &mut buf);

        while read_result.is_ok() {
            let mut remaining = handle_line(&buf);

            buf.clear();
            buf.append(&mut remaining);

            read_result = reader.read_until(b'\n', &mut buf);
        }

        info!("Bailing reader thread with error.");
    });

    Ok(())
}

fn write_message(writer: &mut TcpStream, message: &Message) -> io::Result<()> {
    debug!("SEND> {}", message);
    write!(writer, "{}\r\n", message)
}

fn handle_line(buf: &Vec<u8>) -> Vec<u8> {
    match Message::parse(&buf[..]) {
        Ok((msg, remaining)) => {
            debug!("RECV> {}", msg);
            remaining.to_vec()
        }
        Err(_) => {
            if log_enabled!(Warn) {
                let as_text = str::from_utf8(&buf[..]);

                if as_text.is_ok() {
                    warn!("Failed to parse line: [{}]", as_text.unwrap());
                } else {
                    warn!("Failed to parse line and could not interpret as UTF-8, raw bytes: \
                           [{:?}]",
                          buf);
                }
            }
            Vec::new()
        }
    }
}
