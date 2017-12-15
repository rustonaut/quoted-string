use std::error::{Error as StdError};
use std::fmt::{self, Display};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum CoreError {
    AdvancedFailedAutomaton,
    QuotedStringAlreadyEnded,
    UnquoteableCharQuoted,
    DoesNotStartWithDQuotes,
    DoesNotEndWithDQuotes,
    InvalidChar
}

impl Display for CoreError {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        fter.write_str(self.description())
    }
}

impl StdError for CoreError {
    fn description(&self) -> &'static str {
        use self::CoreError::*;
        match *self {
            AdvancedFailedAutomaton =>
                "advanced automaton after it entered the failed state",
            QuotedStringAlreadyEnded =>
                "continued to use the automaton after the end of the quoted string was found",
            UnquoteableCharQuoted =>
                "a char was escaped with a quoted-pair which can not be represented with a quoted-pair",
            DoesNotStartWithDQuotes =>
                "quoted string did not start with \"",
            DoesNotEndWithDQuotes =>
                "quoted string did not end with \"",
            InvalidChar =>
                "char can not be represented in a quoted string (without encoding)"
        }
    }
}
