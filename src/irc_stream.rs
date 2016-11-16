use std::io;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::io::ErrorKind;
use std::io::Write;
use std::net::TcpStream;

use message::Message;

use openssl::ssl::SslConnectorBuilder;
use openssl::ssl::SslMethod;
use openssl::ssl::SslStream;

/// A type representing an IRC connection, equivalent to `TcpStream` for TCP connections.
#[derive(Debug)]
pub struct IrcStream<S: Read + Write> {
    reader: BufReader<S>,
}

impl IrcStream<SslStream<TcpStream>> {
    pub fn connect_ssl(server: &str, port: u16) -> io::Result<Self> {
        debug!("Connecting to ircs://{}:{}", server, port);
        let ssl_connector = SslConnectorBuilder::new(SslMethod::tls())?.build();
        let raw_connection = TcpStream::connect((server, port))?;
        let connection = ssl_connector.connect(server, raw_connection)
            .map_err(|ssl_err| io::Error::new(ErrorKind::Other, ssl_err))?;
        Ok(IrcStream::new(connection))
    }
}

impl IrcStream<TcpStream> {
    pub fn connect(server: &str, port: u16) -> io::Result<Self> {
        debug!("Connecting to irc://{}:{}", server, port);
        let connection = TcpStream::connect((server, port))?;
        Ok(IrcStream::new(connection))
    }
}

impl<S: Read + Write> IrcStream<S> {
    /// Create a new `IrcStream` wrapping a provided `TcpStream`.
    pub fn new(stream: S) -> Self {
        IrcStream { reader: BufReader::new(stream) }
    }

    /// Sends a message to the target of the stream.
    pub fn send(&mut self, message: &Message) -> io::Result<()> {
        debug!("SEND> {}", message);
        write!(self.stream(), "{}\r\n", message)?;
        self.stream().flush()
    }

    /// Read the next message from this reader.
    pub fn next_message(&mut self) -> io::Result<Message> {
        // TODO: Is the buffer being in here really good? Moving it out leads to all manner of
        // annoying borrow errors.
        let mut buf = Vec::new();
        self.reader.read_until(b'\n', &mut buf)?;
        match Message::parse(&buf[..]) {
            Ok((msg, remaining)) => {
                assert!(remaining.len() == 0);
                debug!("RECV> {}", msg);
                Ok(msg)
            }
            Err(parse_error) => Err(io::Error::new(ErrorKind::InvalidData, parse_error)),
        }
    }

    fn stream(&mut self) -> &mut S {
        self.reader.get_mut()
    }
}

impl<S: Read + Write> Iterator for IrcStream<S> {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        self.next_message().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use message::Message;
    use message::Prefix;
    use command::commands::PING;

    #[test]
    fn reader_read() {
        let input = b"PING 123\r\nPING 456\r\nPING 789\r\n".to_vec();

        let mut reader = IrcStream::new(Cursor::new(input));

        assert_eq!(reader.next_message().unwrap(),
                   Message::from_strs(Prefix::None, PING(), vec!["123"]));
        assert_eq!(reader.next_message().unwrap(),
                   Message::from_strs(Prefix::None, PING(), vec!["456"]));
        assert_eq!(reader.next_message().unwrap(),
                   Message::from_strs(Prefix::None, PING(), vec!["789"]));

        assert!(reader.next_message().is_err());
    }

    #[test]
    fn reader_as_iterator() {
        let input = b"PING 123\r\nPING 456\r\nPING 789\r\n".to_vec();

        let reader = IrcStream::new(Cursor::new(input));

        let mut messages = 0;

        for message in reader {
            messages += 1;
            assert_eq!(message.command, PING());
        }

        assert_eq!(messages, 3);
    }
}
