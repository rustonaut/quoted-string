use std::str::Chars;
use std::iter::Iterator;
use std::cmp::{ PartialEq, Eq, max };
use std::ascii::AsciiExt;

pub trait AsciiCaseInsensitiveEq<Rhs: ?Sized> {
    fn eq_ignore_ascii_case(&self, &Rhs) -> bool;
}

#[derive(Debug, Clone)]
pub struct ContentChars<'a> {
    inner: Chars<'a>
}

impl<'s> ContentChars<'s> {

    /// Crates a `ContentChars` iterator from a &str slice,
    /// assuming it represents a valid quoted-string
    ///
    /// It can be used on both with a complete quoted
    /// string and one from which the surrounding double
    /// quotes where stripped.
    ///
    /// This function relies on quoted to be valid, if it isn't
    /// it will not panic, but might not yield the expected result
    /// as there is no clear way how to handle invalid input.
    pub fn from_string_unchecked(quoted: &'s str) -> Self {
        let inner =
            if quoted.chars().next().map(|ch| ch == '"').unwrap_or(false) {
                // do not panic on invalid input "\"" is seen as "\"\""
                quoted[1..max(1, quoted.len()-1)].chars()
            } else {
                quoted.chars()
            };

        ContentChars{ inner }
    }
}


impl<'a> Iterator for ContentChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
            .map(|ch| {
                if ch == '\\' {
                    // do not panic on invalid input "\"\\\"" is seen as "\"\\\\\""
                    self.inner.next().unwrap_or('\\')
                } else {
                    ch
                }
            })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}


impl<'a> PartialEq<str> for ContentChars<'a> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        iter_eq(self.clone(), other.chars())
    }
}

impl<'a, 's> PartialEq<&'a str> for ContentChars<'s> {
    #[inline]
    fn eq(&self, other: &&'a str) -> bool {
        iter_eq(self.clone(), other.chars())
    }
}

impl<'a, 'b> PartialEq<ContentChars<'a>> for ContentChars<'b> {
    #[inline]
    fn eq(&self, other: &ContentChars<'a>) -> bool {
        iter_eq(self.clone(), other.clone())
    }
}

impl<'a> Eq for ContentChars<'a> {}

impl<'a> AsciiCaseInsensitiveEq<str> for ContentChars<'a> {
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &str) -> bool {
        iter_eq_ascii_case_insensitive(self.clone(), other.chars())
    }
}

impl<'a, 's> AsciiCaseInsensitiveEq<&'a str> for ContentChars<'s> {
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &&'a str) -> bool {
        iter_eq_ascii_case_insensitive(self.clone(), other.chars())
    }
}

impl<'a, 'b> AsciiCaseInsensitiveEq<ContentChars<'a>> for ContentChars<'b> {
    #[inline]
    fn eq_ignore_ascii_case(&self, other: &ContentChars<'a>) -> bool {
        iter_eq_ascii_case_insensitive(self.clone(), other.clone())
    }
}

fn iter_eq<I1, I2>(mut left: I1, mut right: I2) -> bool
    where I1: Iterator<Item=char>,
          I2: Iterator<Item=char>
{
    loop {
        match (left.next(), right.next()) {
            (None, None) => break,
            (Some(x), Some(y)) if x == y => (),
            _ => return false
        }
    }
    true
}

fn iter_eq_ascii_case_insensitive<I1, I2>(mut left: I1, mut right: I2) -> bool
    where I1: Iterator<Item=char>,
          I2: Iterator<Item=char>
{
    loop {
        match (left.next(), right.next()) {
            (None, None) => break,
            (Some(x), Some(y)) if x.eq_ignore_ascii_case(&y) => (),
            _ => return false
        }
    }
    true
}

#[cfg(test)]
mod test {
    use super::{ContentChars, AsciiCaseInsensitiveEq};

    #[test]
    fn no_quotation() {
        let res = ContentChars::from_string_unchecked("abcdef");
        assert_eq!(res.collect::<Vec<_>>().as_slice(), &[
            'a', 'b', 'c' ,'d', 'e', 'f'
        ])
    }

    #[test]
    fn unnecessary_quoted() {
        let res = ContentChars::from_string_unchecked("\"abcdef\"");
        assert_eq!(res.collect::<Vec<_>>().as_slice(), &[
            'a', 'b', 'c' ,'d', 'e', 'f'
        ])
    }

    #[test]
    fn quoted() {
        let res = ContentChars::from_string_unchecked("\"abc def\"");
        assert_eq!(res.collect::<Vec<_>>().as_slice(), &[
            'a', 'b', 'c', ' ', 'd', 'e', 'f'
        ])
    }

    #[test]
    fn with_quoted_pair() {
        let res = ContentChars::from_string_unchecked(r#""abc\" \def""#);
        assert_eq!(res.collect::<Vec<_>>().as_slice(), &[
            'a', 'b', 'c', '"', ' ', 'd', 'e', 'f'
        ])
    }

    #[test]
    fn missing_double_quoted() {
        let res = ContentChars::from_string_unchecked(r#"abc\" \def"#);
        assert_eq!(res.collect::<Vec<_>>().as_slice(), &[
            'a', 'b', 'c', '"', ' ', 'd', 'e', 'f'
        ])
    }

    #[test]
    fn ascii_case_insensitive_eq() {
        let left = ContentChars::from_string_unchecked(r#""abc""#);
        let right = ContentChars::from_string_unchecked(r#""aBc""#);
        assert!(left.eq_ignore_ascii_case(&right))
    }

}