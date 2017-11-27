//! This crate provides utilities to handle quoted strings like such appearing
//! in Media Types (both MIME (i.e. Mail) and HTTP). As there are many small but significant
//! differences in different specifications this crate does not provide
//! a specific implementation. Instead a `QuotedStringSpec` trait is
//! exposed. Implementing it (on zero-sized structs) should allow the
//! usage with any quoted-string specification.
//!
//!
//!
#![warn(missing_docs)]


pub use utils::strip_quotes;
pub use spec::{QuotedStringSpec, UnquotedValidator, QuotedValidator};
pub use iter::{ContentChars, AsciiCaseInsensitiveEq};
pub use unquote::to_content;
pub use quote::{
    quote, quote_if_needed
};
pub use parse::{validate, parse, Parsed};

#[macro_use]
mod utils;
mod spec;
mod iter;
mod unquote;
mod quote;
mod parse;
pub mod test_utils;
