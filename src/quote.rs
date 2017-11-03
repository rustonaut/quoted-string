use std::borrow::Cow;
use std::ascii::AsciiExt;

use error::{Error, Result};

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
#[repr(u8)]
pub enum CharType {
    Unquotable = 0,
    NeedsQuoting = 1,
    Normal = 2,
    Ws = 3
}

impl PartialEq<u8> for CharType {
    fn eq(&self, other: &u8) -> bool {
        *self as u8 == *other
    }
}
impl PartialEq<CharType> for u8 {
    fn eq(&self, other: &CharType) -> bool {
        *self == *other as u8
    }
}

/// A lookup table for chars < 0x80
///
/// Following results can be optained:
///  - `0` (`CharType::Unquotable`): character can not appear in a quoted string at all
///  - `1` (`CharType::NeedsQuoting`): character can appear in a quoted string
///    is quoted (`'\\'` and `'"'`)
///  - `2` (`CharType::Normal`): character can appear without quoting in a quoted string
///  - `3` (`CharType::Ws`): character can appear without quoting in a quoted string and
///    is a whitespace character
///
/// # Example
/// ```
/// use quoted_string::QTEXT_INFO;
/// fn write_to_quoted_string_lossy(ch: char, out: &mut String) {
///     if ch as u8 <= 0x7f {
///         match QTEXT_INFO[ch as usize] {
///             0 => (),
///             1 => { out.push('\\'); out.push(ch); },
///             2 | 3 => { out.push(ch); }
///             _ => unreachable!()
///         }
///     }
/// }
/// ```
pub static QTEXT_INFO: &[u8] = &[
    //0 1 2  3  4  5  6  7  8  9  A  B  C  D  E  F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, //0x0_
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //0x1_
    3, 2, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, //0x2_
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, //0x3_
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, //0x4_
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, //0x5_
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, //0x6_
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0  //0x7_
];




/// Indicates what kind of quoted-strings are used
///
/// As parameter this normally means if Utf8 is allowed
/// or only Ascii.
///
/// In return position it is sometimes used to indicate
/// whether or not a validated string contains non-us-ascci
/// utf8 characters.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum QuotedStringType {
    Utf8,
    AsciiOnly
}

impl QuotedStringType {

    #[inline]
    pub fn from_is_ascii(is_ascii: bool) -> Self {
        if is_ascii {
            QuotedStringType::AsciiOnly
        } else {
            QuotedStringType::Utf8
        }
    }
}


/// Used to determine
/// 1. if the string needs quoting
/// 2. where the first char which could require quoting appears
///
/// Note that a string consisiting only of chars which do not need quoting by them self
/// could still need quoting. For example the string `"a."` requires quoting if it appears
/// in a position where only a `dot-atom` or `quoted-string` is allowed.
pub trait ValidWithoutQuotationCheck {

    /// should return true if the next char is valid without quotation
    fn next_char(&mut self, ch: char) -> bool;

    /// Called after the last char was passed to `next_char`.
    /// It should return true if the whole string is valid without
    /// quotation _assuming_ that before all chars where passed in
    /// order to `next_char` and all calls to `next_char` returned
    /// true.
    ///
    /// This can be used to checks not possible with on a char by
    /// char basis e.g. if it does not end in a `.`.
    ///
    /// Note that because it is only called after one iteration,
    /// validation should be done, if possible, in the `next_char`
    /// method.
    #[inline]
    fn end(&mut self, _all: &str) -> bool { true }
}

impl<T> ValidWithoutQuotationCheck for T
    where T: FnMut(char) -> bool
{
    fn next_char(&mut self, ch: char) -> bool {
        (self)(ch)
    }
}


/// quotes the input string
///
/// basically calls `quote_if_needed(input, |_|false, QuotedStringType::Utf8)`
#[inline]
pub fn quote(input: &str) -> Result<(QuotedStringType, String)> {
    let (mt, res) = quote_if_needed(input, |_|false, QuotedStringType::Utf8)?;
    Ok( match res {
        Cow::Owned(owned) =>  (mt, owned),
        _ => unreachable!("[BUG] the string should have been quoted but wasn't")
    } )
}

/// quotes the input string if needed(RFC 5322/6532/822)
///
/// The `valid_without_quoting` parameter accepts a function,
/// which should _only_ return true if the char is valid
/// without quoting. So this function should never return true
/// for e.g. `\0`. Use this function if some characters are
/// only valid in a quoted-string context.
///
/// If the `allowed_mail_type` parameter is set to `Ascii`
/// the algorithm will return a error if it stumbles over
/// a non-ascii character, else it will just indicate the
/// appearance of one through the returned quoted string type. Note
/// that if you set `quoted_string_type` to `Utf8`
/// the function still can returns a `QuotedStringType::AsciiOnly` which
/// means only ascii characters where contained, even through non ascii
/// character where allowed.
///
/// The quoting process can fail if characters are contained,
/// which can not appear in a quoted string independent of
/// quoted string type. This are chars which are neither
/// `qtext`,`vchar` nor WS (`' '` and `'\t'`).
/// Which are basically only 0x7F (DEL) and the
/// characters < 0x20 (`' '`) except 0x09 (`'\t'`).
///
/// Note that if the `valid_without_quoting` function states a CTL
/// char is valid without quoting then the algorithm will see it
/// as such even through there shouldn't be any context in which a
/// CTL char is valid without quoting.
///
pub fn quote_if_needed<'a, FN>(
    input: &'a str,
    mut valid_without_quoting: FN,
    quoted_string_type: QuotedStringType
) -> Result<(QuotedStringType, Cow<'a, str>)>
    where FN: ValidWithoutQuotationCheck
{
    let (ascii, offset) = scan_ahead(input, &mut valid_without_quoting, quoted_string_type)?;
    if offset == input.len() && valid_without_quoting.end(input) {
        //NOTE: no need to check ascii scan_ahead errors if !ascii && allowed_mail_type == Ascii
        return Ok((QuotedStringType::from_is_ascii(ascii), Cow::Borrowed(input)))
    }

    let (ascii, out) = _quote(input, ascii, quoted_string_type, offset)?;

    Ok( (QuotedStringType::from_is_ascii(ascii), Cow::Owned(out)) )
}

fn _quote(
    input: &str,
    was_ascii: bool,
    quoted_string_type: QuotedStringType,
    start_escape_check_from: usize
) -> Result<(bool, String)>
{
    let ascii_only = quoted_string_type == QuotedStringType::AsciiOnly;
    debug_assert!(!(ascii_only && !was_ascii));

    let (ok, rest) = input.split_at(start_escape_check_from);
    //just guess half of the remaining chars needs escaping
    let mut out = String::with_capacity((rest.len() as f64 * 1.5) as usize);
    out.push('\"');
    out.push_str(ok);

    let mut ascii = was_ascii;
    for ch in rest.chars() {
        if ch.is_ascii() {
            //in this branch char is_ascii is true so char is us-ascci
            //so char is <=128 so the lookup can not panic
            let res = QTEXT_INFO[ch as usize];
            if res == CharType::Unquotable {
                return Err(Error::UnquotableCharacter(ch));
            }
            if res == CharType::NeedsQuoting {
                out.push('\\');
            }
            out.push(ch);
        } else {
            if ascii_only {
                return Err(Error::NonUsAsciiInput);
            }
            ascii = false;

            //any on us-ascii char is valid qtext
            out.push(ch);
        }
    }
    out.push( '"' );
    Ok((ascii, out))

}



fn scan_ahead<FN>(inp: &str, valid_without_quoting: &mut FN, tp: QuotedStringType) -> Result<(bool, usize)>
    where FN: ValidWithoutQuotationCheck
{
    let ascii_only = tp == QuotedStringType::AsciiOnly;
    let mut ascii = true;
    for (offset, ch) in inp.char_indices() {
        if ascii && !ch.is_ascii() {
            if ascii_only {
                return Err(Error::NonUsAsciiInput);
            } else {
                ascii = false;
            }
        }
        if !valid_without_quoting.next_char(ch) {
            return Ok((ascii, offset))
        }
    }
    Ok((ascii, inp.len()))
}


#[cfg(test)]
mod test {
    use std::ascii::AsciiExt;
    use super::*;
    fn is_qtext(ch: char) -> bool {
        match ch {
            //not ' ' [d:32]
            '!' |
            //not '"' [d:34]
            '#'...'[' |
            //not '\\' [d:92]
            ']'...'~' => true,
            _ => false
        }
    }

    struct TokenCheck;
    impl ValidWithoutQuotationCheck for TokenCheck {
        fn next_char(&mut self, ch: char) -> bool {
            match ch {
                'a'...'z' |
                'A'...'Z' |
                '-' => true,
                _ => false
            }
        }

        fn end(&mut self, all: &str) -> bool {
            all.len() > 0
        }

    }

    #[test]
    fn quote_ascii() {
        let mti = QuotedStringType::Utf8;
        let data = &[
            ("this is simple", "\"this is simple\""),
            ("also\tsimple", "\"also\tsimple\""),
            ("with quotes\"<-", "\"with quotes\\\"<-\""),
            ("with slash\\<-", "\"with slash\\\\<-\"")
        ];
        for &(unquoted, quoted) in data.iter() {
            let (mail_type, got_quoted) = assert_ok!(
                quote_if_needed( unquoted, |ch| ch > ' ' && ch <= '~', mti));
            assert_eq!(QuotedStringType::AsciiOnly, mail_type);
            assert_eq!(quoted, &*got_quoted);
        }
    }

    #[test]
    fn quote_utf8() {
        let data = &[
            ("has → uft8", "\"has → uft8\""),
            ("also\t→\tsimple", "\"also\t→\tsimple\""),
            ("with→quotes\"<-", "\"with→quotes\\\"<-\""),
            ("with→slash\\<-", "\"with→slash\\\\<-\"")
        ];
        for &(unquoted, quoted) in data.iter() {
            let res = quote_if_needed( unquoted, |_|false, QuotedStringType::Utf8 );
            let (mail_type, got_quoted) = assert_ok!(res);
            assert_eq!(QuotedStringType::Utf8, mail_type);
            assert_eq!(quoted, &*got_quoted);
        }
    }


    #[test]
    fn no_quotation_needed_ascii() {
        let res = quote_if_needed("simple", |ch| ch >= 'a' && ch <= 'z', QuotedStringType::AsciiOnly);
        let (qst, res) = assert_ok!(res);
        assert_eq!(QuotedStringType::AsciiOnly, qst);
        assert_eq!("simple", &*res);
        let is_borrowed = if let Cow::Borrowed(_) = res { true } else { false };
        assert_eq!(true, is_borrowed);
    }

    #[test]
    fn no_quotation_needed_utf8() {
        let mt = QuotedStringType::Utf8;
        let (mt, res) = assert_ok!(
            quote_if_needed("simp↓e", |ch: char| !ch.is_ascii() || is_qtext(ch), mt));
        assert_eq!(QuotedStringType::Utf8, mt);
        assert_eq!("simp↓e", &*res);
        let is_borrowed = if let Cow::Borrowed(_) = res { true } else { false };
        assert_eq!(true, is_borrowed);
    }

    #[test]
    fn no_del() {
        assert_err!(quote_if_needed("\x7F", |_|false, QuotedStringType::AsciiOnly));
    }

    #[test]
    fn no_ctl() {
        let mut text = String::with_capacity(1);
        let bad_chars = (b'\0'..b' ').filter(|&b| b != b'\t' ).map(|byte| byte as char);
        for char in bad_chars {
            text.clear();
            text.insert(0, char);
            assert_err!(quote_if_needed(&*text, |_|false, QuotedStringType::AsciiOnly));
        }
    }

    #[test]
    fn quote_always_quotes() {
        assert_eq!(
            (QuotedStringType::AsciiOnly, "\"simple\"".to_owned()),
            assert_ok!(quote("simple"))
        );
    }

    #[test]
    fn using_valid_without_quoting() {
        let data = &[
            ("not@a-token", "\"not@a-token\"", true),
            ("not a-token", "\"not a-token\"", true),
            ("a-token-it-is", "a-token-it-is", false)
        ];
        for &(unquoted, exp_res, quoted) in data.iter() {
            let res = quote_if_needed(unquoted, TokenCheck, QuotedStringType::AsciiOnly);
            let (mt, res) = assert_ok!(res);
            assert_eq!(QuotedStringType::AsciiOnly, mt);
            if quoted {
                let owned: Cow<str> = Cow::Owned(exp_res.to_owned());
                assert_eq!(owned, res);
            } else {
                assert_eq!(Cow::Borrowed(exp_res), res);
            }
        }
    }

    #[test]
    fn quotes_utf8() {
        let res = assert_ok!(quote_if_needed("l↓r", TokenCheck, QuotedStringType::Utf8));
        let was_quoted = if let &Cow::Owned(..) = &res.1 { true } else { false };
        assert_eq!( true, was_quoted );
    }

    #[test]
    fn error_with_quotable_utf8_but_ascii_only() {
        assert_err!(quote_if_needed("l→r", TokenCheck, QuotedStringType::AsciiOnly));
    }

    #[test]
    fn check_end_is_used() {
        let qt = QuotedStringType::AsciiOnly;
        let res = quote_if_needed("", TokenCheck, qt);
        let (got_mt, quoted) = assert_ok!(res);
        assert_eq!(QuotedStringType::AsciiOnly, got_mt);
        assert_eq!("\"\"", quoted);
    }

    #[test]
    fn qtext_info_check() {
        for bch in 0u8..0x80 {
            let ch = bch as char;
            let res = QTEXT_INFO[bch as usize];
            match res {
                0 => assert!(!is_qtext(ch)),
                1 => assert!(ch == '\\' || ch == '"' ),
                2 => assert!(is_qtext(ch)),
                3 => assert!(ch == ' ' || ch == '\t'),
                _ => panic!("unexpected value in lookup table")
            }
        }
    }

}