//! provides an example implementation of QuotedStringSpec
use std::fmt::{self, Display};
use std::error::Error;

use spec::{
    QuotedStringSpec,
    QuotedValidator, UnquotedValidator,
    ValidationResult
};

/// Error used by TestQuotedStringSpec
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TestError {
    /// character was not quotable
    Unquoteable,
    /// the `'"'` normally surrounding a quoted string are missing
    QuotesMissing,
    /// a character which needs to be escaped with a quoted-pair wasn't
    EscapeMissing,
    /// a quoted-string ended in a `'\\'` without a character it escapes
    TailingEscape
}

/// a QuotedString spec example implementation
///
/// It's used for testing inkl. doc tests (which is mainly why it is public).
#[derive(Copy, Clone, Debug)]
pub struct TestSpec;


/// The type used by `<TestSpec as QuotedStringSpec>::QuotedValidator`
///
/// - it treats '\0' and '"' as non-qtext, non-ws quotable
/// - qtext are all printable us-ascii chars ('!'...'~')
/// - semantic ws are ' ' and '\t'
/// - '\n' is a non-semantic ws
#[derive(Copy, Clone, Debug)]
pub struct TestQuotedValidator;

/// The type used by `<TestSpec as QuotedStringSpec>::UnquotedValidator`
///
/// it exptects a 6 character sequence of upper and lower case us-ascii leters
/// which can also contain dots but not multiple dots in a row. If it is constructed
/// with `last_was_dot==true` then it also can not start with a dot.
#[derive(Clone, Debug)]
pub struct TestUnquotedValidator {
    /// len of the validated unquoted string (== count of validate_next_char calls)
    pub len: usize,
    /// if the last char was `'.'`
    pub last_was_dot: bool
}

impl Display for TestError {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, fter)
    }
}

impl Error for TestError {
    fn description(&self) -> &str {
        "test error triggered"
    }
}



impl QuotedStringSpec for TestSpec {
    type Err = TestError;
    type QuotedValidator = TestQuotedValidator;
    type UnquotedValidator = TestUnquotedValidator;

    #[inline]
    fn new_unquoted_validator() -> Self::UnquotedValidator {
        TestUnquotedValidator {
            len: 0,
            last_was_dot: true
        }
    }

    #[inline]
    fn new_quoted_validator() -> Self::QuotedValidator {
        TestQuotedValidator
    }

    #[inline]
    fn unquoteable_char(_ch: char) -> Self::Err {
        TestError::Unquoteable
    }

    #[inline]
    fn unquoted_quotable_char(_ch: char) -> Self::Err {
        TestError::EscapeMissing
    }

    #[inline]
    fn error_for_tailing_escape() -> Result<(), Self::Err> {
        Err(TestError::TailingEscape)
    }

    #[inline]
    fn quoted_string_missing_quotes() -> Self::Err {
        TestError::QuotesMissing
    }


}

impl QuotedValidator for TestQuotedValidator {

    type Err = TestError;

    #[inline]
    fn validate_next_char(&mut self, ch: char) -> ValidationResult<Self::Err> {
        match ch {
            '\\' | '"' | '\0' => ValidationResult::Quotable,
            '!'...'~' => ValidationResult::QText,
            ' ' | '\t' => ValidationResult::SemanticWs,
            '\n' => ValidationResult::NotSemantic,
            _ => ValidationResult::Invalid(TestError::Unquoteable)
        }
    }

    #[inline]
    fn end_validation(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }
}

impl UnquotedValidator for TestUnquotedValidator {

    type Err = TestError;

    #[inline]
    fn validate_next_char(&mut self, ch: char) -> bool {
        self.len += 1;
        if self.last_was_dot {
            if ch == '.' {
                return false
            }
            self.last_was_dot = false;
        }
        match ch {
            '.' => {self.last_was_dot = true; true},
            'a'...'z' |
            'A'...'Z' => true,
            _ => false
        }
    }


    #[inline]
    fn end_validation(&mut self) -> bool {
        self.len == 6
    }
}