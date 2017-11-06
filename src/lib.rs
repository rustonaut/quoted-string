#[macro_use]
extern crate quick_error;

pub use self::iter::{ContentChars, AsciiCaseInsensitiveEq};
pub use self::unquote::unquote_unchecked;
pub use self::quote::{
    QuotedStringType, ValidWithoutQuotationCheck,
    quote, quote_if_needed,
    CharType
};

#[macro_use]
mod utils;
pub mod error;
mod iter;
mod unquote;
mod quote;
