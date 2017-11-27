use std::str::Chars;
use std::iter::Iterator;
use std::cmp::{ PartialEq, Eq };
use std::marker::PhantomData;

use spec::{QuotedStringSpec, QuotedValidator, };
use utils::strip_quotes;
// this import will become unused in future rust versions
// but won't be removed for now for supporting current
// rust versions
#[allow(warnings)]
use std::ascii::AsciiExt;

/// Analogous to PartialEq, but with _ascii_ case insensitive equality
pub trait AsciiCaseInsensitiveEq<Rhs: ?Sized> {

    /// compares this instance with other with a ascii case insensitive comparsion algorithm
    ///
    /// Note that this is _ascii_ case insensitivity. Which means this will not work
    /// well/as expected if the content contain non ascii characters.
    /// E.g. the upercase of `"ß"` was `"SS"` but by know there is also a
    /// "ẞ" used in all caps writing.
    fn eq_ignore_ascii_case(&self, other: &Rhs) -> bool;
}

/// A iterator over chars of the content represented by the quoted strings (PartialEq<&str>)
///
/// It will on the fly (without extra allocation)
/// remove the surrounding quotes and unquote quoted-pairs
///
/// It implements Eq, and PartialEq with str, &str and itself. This
/// enables to compare a quoted string with a string representing the
/// content of a quoted string.
///
/// # Example
///
/// ```
/// # use quoted_string::ContentChars;
/// use quoted_string::AsciiCaseInsensitiveEq;
/// use quoted_string::test_utils::TestSpec;
///
/// let quoted_string = r#""ab\"\ c""#;
/// let cc = ContentChars::<TestSpec>::from_str_unchecked(quoted_string).unwrap();
/// assert_eq!(cc, "ab\" c");
/// assert!(cc.eq_ignore_ascii_case("AB\" c"));
/// assert_eq!(cc.collect::<Result<Vec<_>,_>>().unwrap().as_slice(), &[ 'a', 'b', '"', ' ', 'c' ] );
///
/// ```
#[derive(Debug, Clone)]
pub struct ContentChars<'a, Spec: QuotedStringSpec> {
    inner: Chars<'a>,
    q_validator: Spec::QuotedValidator,
    marker: PhantomData<Spec>
}

impl<'s, Spec> ContentChars<'s, Spec>
    where Spec: QuotedStringSpec
{

    /// creates a char iterator over the content of a quoted string
    ///
    /// the quoted string is _assumed_ to be valid and not explicitely checked for validity
    /// but because of the way unquoting works a number of error can be detected
    ///
    /// # Error
    /// if the string does not start and end with `'"'` a error is returned as
    /// the surrounding `'"'` are stripped in the constructor
    pub fn from_str_unchecked(quoted: &'s str) -> Result<Self, Spec::Err> {
        let content =
            strip_quotes(quoted)
            .ok_or_else(Spec::quoted_string_missing_quotes)?;

        let q_validator = Spec::new_quoted_validator();
        let inner = content.chars();

        Ok(ContentChars{ inner, q_validator, marker: PhantomData })
    }

    /// creates a ContentChars iterator from a str and a QuotedValidator
    ///
    /// The `partial_quoted_content` is assumed to be a valid quoted string
    /// without the surrounding `'"'`. It might not be a the complete content
    /// of a quoted string but if it isn't the q_validator is expected to have
    /// been used on a chars stripped on the left side (and no more than that).
    /// Note that it can work with using it on less then all chars but this depends
    /// on the Spec used. E.g. if any decison of the spec only depends on the current char
    /// (QuotedValidator is zero-sized) then no char had to be used with it.
    pub fn from_parts_unchecked(
        partial_quoted_content: &'s str,
        q_validator: Spec::QuotedValidator
    ) -> Self
    {
        let inner = partial_quoted_content.chars();
        ContentChars{ inner, q_validator, marker: PhantomData }
    }
}


impl<'a, Spec> Iterator for ContentChars<'a, Spec>
    where Spec: QuotedStringSpec
{
    type Item = Result<char, Spec::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        use spec::ValidationResult::*;
        loop {
            if let Some(ch) = self.inner.next() {
                match self.q_validator.validate_next_char(ch) {
                    QText | SemanticWs => return Some(Ok(ch)),
                    Escape => {
                        if let Some(ch) = self.inner.next() {
                            return Some(Ok(ch));
                        } else {
                            return Some(Spec::error_for_tailing_escape().map(|_|'\\'));
                        }
                    }
                    Quotable =>  return Some(Err(Spec::unquoted_quotable_char(ch))),
                    Invalid(err) => return Some(Err(err)),
                    NotSemanticWs => continue,
                }
            } else {
                return None;
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}


impl<'a, Spec> PartialEq<str> for ContentChars<'a, Spec>
    where Spec: QuotedStringSpec
{

    #[inline]
    fn eq(&self, other: &str) -> bool {
        iter_eq(self.clone(), other.chars().map(|ch|Ok(ch)), |l,r|l==r)
    }
}

impl<'a, 'b, Spec> PartialEq<ContentChars<'b, Spec>> for &'a str
    where Spec: QuotedStringSpec
{
    #[inline]
    fn eq(&self, other: &ContentChars<'b, Spec>) -> bool {
        *other == **self
    }
}

impl<'a, 'b, Spec> PartialEq<&'b str> for ContentChars<'a, Spec>
    where Spec: QuotedStringSpec
{
    #[inline]
    fn eq(&self, other: &&'b str) -> bool {
        self == *other
    }
}

impl<'a, 'b, Spec> PartialEq<ContentChars<'b, Spec>> for ContentChars<'a, Spec>
    where Spec: QuotedStringSpec
{
    #[inline]
    fn eq(&self, other: &ContentChars<'b, Spec>) -> bool {
        iter_eq(self.clone(), other.clone(), |l,r|l==r)
    }
}

impl<'a, Spec> Eq for ContentChars<'a, Spec> where Spec: QuotedStringSpec {}



impl<'a, Spec> AsciiCaseInsensitiveEq<str> for ContentChars<'a, Spec>
    where Spec: QuotedStringSpec
{
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &str) -> bool {
        iter_eq(self.clone(), other.chars().map(|ch|Ok(ch)), |l,r| l.eq_ignore_ascii_case(&r))
    }
}

impl<'a, 'b, Spec> AsciiCaseInsensitiveEq<ContentChars<'b, Spec>> for ContentChars<'a, Spec>
    where Spec: QuotedStringSpec
{
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &ContentChars<'b, Spec>) -> bool {
        iter_eq(self.clone(), other.clone(), |l,r|l.eq_ignore_ascii_case(&r))
    }
}

impl<'a, 'b, Spec> AsciiCaseInsensitiveEq<ContentChars<'b, Spec>> for &'a str
    where Spec: QuotedStringSpec
{
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &ContentChars<'b, Spec>) -> bool {
        other == *self
    }
}



impl<'a, 'b, Spec> AsciiCaseInsensitiveEq<&'b str> for ContentChars<'a, Spec>
    where Spec: QuotedStringSpec
{
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &&'b str) -> bool {
        self == *other
    }
}

fn iter_eq<I1, I2, E, FN>(mut left: I1, mut right: I2, cmp: FN) -> bool
    where I1: Iterator<Item=Result<char, E>>,
          I2: Iterator<Item=Result<char, E>>, FN: Fn(char, char) -> bool
{
    loop {
        match (left.next(), right.next()) {
            (None, None) => return true,
            (Some(Ok(x)), Some(Ok(y))) if cmp(x, y) => (),
            _ => return false
        }
    }
}



#[cfg(test)]
mod test {
    use test_utils::*;
    use super::{ContentChars, AsciiCaseInsensitiveEq};

    #[test]
    fn missing_double_quoted() {
        let res = ContentChars::<TestSpec>::from_str_unchecked("abcdef");
        assert_eq!(res, Err(TestError::QuotesMissing));
    }

    #[test]
    fn unnecessary_quoted() {
        let res = ContentChars::<TestSpec>::from_str_unchecked("\"abcdef\"").unwrap();
        assert_eq!(res.collect::<Result<Vec<_>, _>>().unwrap().as_slice(), &[
            'a', 'b', 'c' ,'d', 'e', 'f'
        ])
    }

    #[test]
    fn quoted() {
        let res = ContentChars::<TestSpec>::from_str_unchecked("\"abc def\"").unwrap();
        assert_eq!(res.collect::<Result<Vec<_>, _>>().unwrap().as_slice(), &[
            'a', 'b', 'c', ' ', 'd', 'e', 'f'
        ])
    }

    #[test]
    fn with_quoted_pair() {
        let res = ContentChars::<TestSpec>::from_str_unchecked(r#""abc\" \def""#).unwrap();
        assert_eq!(res.collect::<Result<Vec<_>, _>>().unwrap().as_slice(), &[
            'a', 'b', 'c', '"', ' ', 'd', 'e', 'f'
        ])
    }

    #[test]
    fn strip_non_semantic_ws() {
        let res = ContentChars::<TestSpec>::from_str_unchecked("\"abc\ndef\"").unwrap();
        assert_eq!(res.collect::<Result<Vec<_>, _>>().unwrap().as_slice(), &[
            'a', 'b', 'c', 'd', 'e', 'f'
        ])
    }

    #[test]
    fn ascii_case_insensitive_eq() {
        let left = ContentChars::<TestSpec>::from_str_unchecked(r#""abc""#).unwrap();
        let right = ContentChars::<TestSpec>::from_str_unchecked(r#""aBc""#).unwrap();
        assert!(left.eq_ignore_ascii_case(&right))
    }
}
