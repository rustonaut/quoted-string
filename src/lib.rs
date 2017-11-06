//! This crate provides utilities to handle quoted strings as they appear in multiple
//! mail and web related RFCs. While it is mainly based on rfc5322 (Internet Message Format).
//! It also supports Utf8 based on rfc6532 (optional) and is compatible with quoted-strings
//! as they appear in mime types (including in HTTP/1.1 context).
//!
//! What it currently does not support are soft-line breaks of rfc5322,
//! neither is the obsolete part of the syntax supported (for now).
//!
//! The supported grammar is:
//!
//! ```no-rust
//! quoted-string   = DQUOTE *( *WSP qcontent) *WSP DQUOTE
//! qcontent = qtext / quoted-pair
//! qtext = %d33 / %d35-91 / %d93-126 ; printable us-ascii chars not including '\\' and '"'
//! quoted-pair = ("\" (VCHAR / WSP)) ; VCHAR are printable us-ascii chars
//! DQUPTE = '"'
//! WSP = ' ' / '\t'
//! ```
//!
//! Note that the contained `CharType` enum has a more fine grained differentiation
//! then aboves grammar as it can differ between chars which are unquotable, which need
//! to be represented as a quoted-pair (`'\\'` and `'"'`) which are token chars (token of rfc2045)
//! or tspecial chars (rfc2045) or non us-ascii chars or allowed white space chars (`'\t'` and `' '`).
//!
//! The obsolete syntax is is currently _not supported_. Differences would be:
//!
//! 1. it would allow CTL's in qtext
//! 2. it would allow quoted pairs to escape CTL's, `'\0'`, `'\n'`, `'\r'`
//!
//! Nevertheless this part of the syntax is obsolete and should not be generated
//! at all. Adding opt-in support for parts parsing quoted-string is in consideration.
#![warn(missing_docs)]

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

/// quoted-string errors
pub mod error;
mod iter;
mod unquote;
mod quote;
