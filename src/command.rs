use std::fmt::Display;
use std::fmt::Formatter;
use std;

/// An IRC command. These can either be a sequence of letters
/// (which I'm calling "word") or a numeric value.
/// Note that creating one of these directly will
/// bypass validation and cause you to have a Bad Time.
#[derive(Debug,Clone, PartialEq, Eq)]
pub enum Command<'a> {
    Word(&'a str),
    Number(u16),
}

impl<'a> Command<'a> {
    /// Creates a Command::Word validated to ensure it is a valid IRC command.
    /// Only validates that the command is made up of valid characters, not that
    /// it's a command that appears in any RFC.
    ///
    /// # Panics
    ///
    /// Will panic if `word` has any characters outside of `[a-zA-Z]`.
    pub fn of_word(word: &'a str) -> Self {
        for (i, c) in word.chars().enumerate() {
            // As deep as my hatred for regexes goes this is getting a bit silly
            let codepoint = c as u32;
            let is_lowercase_letter = codepoint >= 0x41 && codepoint <= 0x5A;
            let is_uppercase_letter = codepoint >= 0x61 && codepoint <= 0x7A;
            assert!(is_lowercase_letter || is_uppercase_letter,
                    "Word IRC commands must contain only chars A-Za-z but [{}] index {} is [{}]",
                    word,
                    i,
                    c);
        }

        Command::Word(word)
    }

    /// Creates a Command::Number validated to ensure it is a valid IRC command.
    /// Only validates that the command is made up of a valid number, not that
    /// it's a command that appears in any RFC.
    ///
    /// # Panics
    ///
    /// Will panic if `number` cannot be represented as a 3-digit number (i.e. if it is
    /// greater than 999).
    pub fn of_number(number: u16) -> Self {
        assert!(number <= 999,
                "Numeric IRC commands must be representable as a 3-digit number but got {}",
                number);
        Command::Number(number)
    }
}

impl<'a> Display for Command<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match *self {
            Command::Word(word) => write!(fmt, "{}", word),
            Command::Number(number) => write!(fmt, "{:0>3}", number),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn of_number_valid() {
        Command::of_number(123);
    }

    #[test]
    #[should_panic]
    fn of_number_too_large() {
        Command::of_number(1000);
    }

    #[test]
    fn of_word_valid() {
        Command::of_word("PrIvMsG");
    }

    #[test]
    #[should_panic]
    fn of_word_non_alpha_chars() {
        Command::of_word("PR1VMSG");
    }

    #[test]
    fn fmt_number_unpadded() {
        assert_eq!(format!("{}", Command::of_number(123)), "123");
    }

    #[test]
    fn fmt_number_padded() {
        assert_eq!(format!("{}", Command::of_number(1)), "001");
    }

}
