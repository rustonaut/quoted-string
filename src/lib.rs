//! This crate provides utilities to handle quoted strings as they appear in
//! multiple mail and web related RFCs. While it is mainly based on RFC5322
//! (Internet Message Format).  It also supports Utf8 based on RFC6532 (optional)
//! and is compatible with quoted-strings as they appear in mime types (including
//! in HTTP/1.1 context).
//!
//! What it currently does not support are soft-line breaks of RFC5322 and the
//! obsolete parts of the syntax.
//!
//! Grammar
//! -------
//! ```no-rust
//! quoted-string   = DQUOTE *( *WSP qcontent) *WSP DQUOTE
//! WSP = ' ' / '\t'
//! qcontent = qtext / quoted-pair
//! qtext = %d33 / %d35-91 / %d93-126 ; printable us-ascii chars not including '\\' and '"'
//! quoted-pair = ("\" (VCHAR / WSP)) ; VCHAR are printable us-ascii chars
//! ```
//!
//! The obsolete syntax is currently **not supported**. Differences would be:
//!
//! 1. it would allow CTL's in qtext
//! 2. it would allow quoted pairs to escape CTL's, `'\0'`, `'\n'`, `'\r'`
//!
//! Nevertheless this part of the syntax is obsolete and should not be generated at
//! all. Adding opt-in support for parts parsing quoted-string is in consideration.
#![warn(missing_docs)]
#![cfg_attr(test, deny(warnings))]

pub use utils::strip_quotes;
pub use spec::{QuotedStringSpec, UnquotedValidator};
pub use iter::{ContentChars, AsciiCaseInsensitiveEq};
pub use unquote::quoted_string_to_content;
pub use quote::{
    quote, quote_if_needed
};

#[macro_use]
mod utils;
mod spec;
mod iter;
mod unquote;
mod quote;
pub mod test_utils;
