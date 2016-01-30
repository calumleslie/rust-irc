#[macro_use]
extern crate nom;

use std::fmt::Display;
use std::fmt::Formatter;
use std::vec::Vec;

// Maybe consider storing the whole line and references?
#[derive(Debug,Clone)]
pub struct IrcLine<'a> {
    prefix: Option<&'a str>,
    command: &'a str,
    arguments: Vec<&'a str>,
}

impl<'a> Display for IrcLine<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
    	try!(match self.prefix {
    		None => Ok( () ),
    		Some(value) => write!(fmt, ":{} ", value )
    	});

        try!( write!(fmt, "{}", self.command) );

        for (i, argument) in self.arguments.iter().enumerate() {
        	try!( write!( fmt, " " ) );

        	if i == self.arguments.len() - 1 && argument.contains( ' ' ) {
        		try!( write!( fmt, ":" ) );
        	}

        	try!( write!( fmt, "{}", argument ) );
        }

        Ok( () )
    }
}

#[cfg(test)]
mod tests {
	use super::*;
    #[test]
    fn command_only() {
        let line = IrcLine {
            prefix: None,
            command: "PING",
            arguments: vec![],
        };

        assert_eq!(format!("{}", line), "PING");
    }

    #[test]
    fn prefix_command() {
    	let line = IrcLine {
    		prefix: Some( "somedude" ),
    		command: "PING",
    		arguments: vec![],
    	};

    	assert_eq!(format!("{}", line), ":somedude PING" );
    }

    #[test]
    fn command_args() {
    	let line = IrcLine {
    		prefix: None,
    		command: "PRIVMSG",
    		arguments: vec![ "someone", "something" ],
    	};

    	assert_eq!(format!("{}", line), "PRIVMSG someone something" );
    }

    #[test]
    fn command_args_with_long_final_argument() {
    	let line = IrcLine {
    		prefix: None,
    		command: "PRIVMSG",
    		arguments: vec![ "someone", "Hey I love being on IRC" ],
    	};

    	assert_eq!(format!("{}", line), "PRIVMSG someone :Hey I love being on IRC" );
    }

    #[test]
    fn everything() {
    	let line = IrcLine {
    		prefix: Some("information"),
    		command: "PRIVMSG",
    		arguments: vec![ "someone", "something", "Hey I love being on IRC" ],
    	};

    	assert_eq!(format!("{}", line), ":information PRIVMSG someone something :Hey I love being on IRC" );
    }
}
