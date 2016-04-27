use std::fmt::Display;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;

use message::Message;

/// A type representing an IRC connection, equivalent to TcpStream for TCP connections.
#[derive(Debug)]
pub struct IrcStream {
    tcp_stream: TcpStream,
}

/// A reader of IRC messages.
///
/// Generally R should implement BufRead. Because this type implments `Iterator` the messages in
/// the stream can be iterated, however errors will be skipped, and the iteration will stop on the
/// first error (which may not be fatal).
#[derive(Debug)]
pub struct IrcReader<R> {
    reader: R,
}

impl IrcStream {
    /// Connects to an IRC server as a client at address `server`.
    ///
    /// The server argument must implement `Display` since it is logged using the default format,
    /// it's not clear if this is overly limiting.
    pub fn connect<A: ToSocketAddrs + Display>(server: A) -> io::Result<Self> {
        info!("Connecting to server at {}", server);

        Ok(IrcStream { tcp_stream: try!(TcpStream::connect(server)) })
    }

    /// Create a new IrcStream wrapping a provided TcpStream.
    pub fn new(tcp_stream: TcpStream) -> Self {
        IrcStream { tcp_stream: tcp_stream }
    }

    /// Try to clone the IrcStream. Semantics w.r.t. closing and so on are the same as the
    /// TcpStream type being wrapped.
    pub fn try_clone(&self) -> io::Result<Self> {
        self.tcp_stream.try_clone().map(|cloned_stream| IrcStream { tcp_stream: cloned_stream })
    }

    /// Clone the stream and wrap it in an IrcReader. Because the IrcReader has buffering it's
    /// recommended not to have more than one reader for each stream.
    pub fn reader(&self) -> io::Result<IrcReader<BufReader<TcpStream>>> {
        let read_stream = try!(self.tcp_stream.try_clone());
        Ok(IrcReader { reader: BufReader::new(read_stream) })
    }

    /// Sends a message to the target of the stream.
    pub fn send(&mut self, message: &Message) -> io::Result<()> {
        debug!("SEND> {}", message);
        try!(write!(self.tcp_stream, "{}\r\n", message));
        self.tcp_stream.flush()
    }
}

impl<R> IrcReader<R> where R: BufRead
{
    /// Read the next message from this reader.
    pub fn next_message(&mut self) -> io::Result<Message> {
        // TODO: Is the buffer being in here really good? Moving it out leads to all manner of
        // annoying borrow errors.
        let mut buf = Vec::new();
        try!(self.reader.read_until(b'\n', &mut buf));
        match Message::parse(&buf[..]) {
            Ok((msg, remaining)) => {
                assert!(remaining.len() == 0);
                debug!("RECV> {}", msg);
                Ok(msg)
            }
            Err(parse_error) => Err(Error::new(ErrorKind::InvalidData, parse_error)),
        }
    }
}

impl<R> Iterator for IrcReader<R> where R: BufRead
{
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        self.next_message().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;
    use std::io::Cursor;
    use message::Message;
    use message::Prefix;
    use command::commands::PING;

    #[test]
    fn reader_read() {
        let input = b"PING 123\r\nPING 456\r\nPING 789\r\n".to_vec();

        let mut reader = IrcReader { reader: BufReader::new(Cursor::new(input)) };

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

        let reader = IrcReader { reader: BufReader::new(Cursor::new(input)) };

        let mut messages = 0;

        for message in reader {
            messages += 1;
            assert_eq!(message.command, PING());
        }

        assert_eq!(messages, 3);
    }
}
