use spec::{QuotedStringSpec, QuotedValidator};
use std::borrow::Cow;
use super::iter::ContentChars;

use utils::strip_quotes;

/// converts a quoted string into it's content
///
/// This methods retrieves the content of a quoted-string, which means it strips the
/// surrounding `'"'`-quoted, converts quoted-pairs into the values they represent and
/// strips not-semantic character.
///
/// # Example
/// ```
/// # use std::borrow::Cow;
/// //use your own Spec in practise
/// use quoted_string::test_utils::TestSpec;
/// use quoted_string::to_content;
///
/// let content = to_content::<TestSpec>("\"ab\\\"c\nde\"")
///     .expect("only fails if the input is not a quoted string");
/// assert_eq!(&*content, "ab\"cde");
///
/// let content = to_content::<TestSpec>("\"simple\"").unwrap();
/// // to content will just use slicing to strip `'"'`-quotes if possible
/// assert_eq!(content, Cow::Borrowed("simple"));
/// ```
///
pub fn to_content<'a, Spec:QuotedStringSpec>(
    quoted_string: &'a str
) -> Result<Cow<'a, str>, Spec::Err>
{
    let quoted_string_content =
        if let Some(content) = strip_quotes(quoted_string) {
            content
        } else {
            return Err(Spec::quoted_string_missing_quotes())
        };

    let mut q_validator = Spec::new_quoted_validator();
    let unchanged = scan_unchanged::<Spec>(quoted_string_content, &mut q_validator)?;
    let (split_idx, last_ch, last_was) =
        match unchanged {
            ScanResult::ValidUnchanged => return Ok(Cow::Borrowed(quoted_string_content)),
            ScanResult::ValidUpTo { split_idx, last_ch, last_was } => (split_idx, last_ch, last_was)
        };

    let (unquoted, tail) = quoted_string_content.split_at(split_idx);
    let mut buffer = String::from(unquoted);
    // we have to handle the last char ourself
    let tail_offset;
    match last_was {
        LastWas::Escape => {
            debug_assert_eq!(last_ch, '\\');
            if let Some(ch) = tail[1..].chars().next() {
                buffer.push(ch);
                tail_offset = 1 + ch.len_utf8();
            } else {
                Spec::error_for_tailing_escape()?;
                buffer.push('\\');
                tail_offset = 1;
            }
        }
        LastWas::NotSemanticWs => {
            tail_offset = last_ch.len_utf8();
        }
    }

    let iter = ContentChars::<Spec>::from_parts_unchecked(&tail[tail_offset..], q_validator);
    for ch in iter {
        buffer.push(ch?)
    }
    Ok(Cow::Owned(buffer))
}

#[repr(u8)] enum LastWas { Escape, NotSemanticWs }
enum ScanResult {
    ValidUnchanged,
    ValidUpTo {
        split_idx: usize,
        last_was: LastWas,
        last_ch: char
    }
}

fn scan_unchanged<Spec: QuotedStringSpec>(
    input: &str,
    q_validator: &mut Spec::QuotedValidator
) -> Result<ScanResult, Spec::Err>
{
    use spec::ValidationResult::*;
    for (idx, ch) in input.char_indices() {
        match q_validator.validate_next_char(ch) {
            QText | SemanticWs => {},
            NeedsQuotedPair => {
                if ch == '\\' {
                    return Ok(ScanResult::ValidUpTo {
                        split_idx: idx,
                        last_ch: ch,
                        last_was: LastWas::Escape
                    })
                }
                return Err(Spec::unquoted_quotable_char(ch));
            }
            NotSemantic => {
                return Ok(ScanResult::ValidUpTo {
                    split_idx: idx,
                    last_ch: ch,
                    last_was: LastWas::NotSemanticWs
                })
            }
            Invalid(err) => {
                return Err(err);
            }

        }
    }
    Ok(ScanResult::ValidUnchanged)
}


#[cfg(test)]
mod test {
    use test_utils::*;
    use std::borrow::Cow;
    use super::to_content;

    #[test]
    fn no_quotes() {
        let res = to_content::<TestSpec>("noquotes");
        assert_eq!(res, Err(TestError::QuotesMissing));
    }

    #[test]
    fn unnecessary_quoted() {
        let res = to_content::<TestSpec>(r#""simple""#).unwrap();
        assert_eq!(res, Cow::Borrowed("simple"))
    }

    #[test]
    fn quoted_but_no_quoted_pair() {
        let res = to_content::<TestSpec>(r#""abc def""#).unwrap();
        assert_eq!(res, Cow::Borrowed("abc def"))
    }

    #[test]
    fn with_quoted_pair() {
        let res = to_content::<TestSpec>(r#""a\"b""#).unwrap();
        let expected: Cow<'static, str> = Cow::Owned(r#"a"b"#.into());
        assert_eq!(res, expected);
    }

    #[test]
    fn with_multiple_quoted_pairs() {
        let res = to_content::<TestSpec>(r#""a\"\bc\ d""#).unwrap();
        let expected: Cow<'static, str> = Cow::Owned(r#"a"bc d"#.into());
        assert_eq!(res, expected);
    }

    #[test]
    fn empty() {
        let res = to_content::<TestSpec>(r#""""#).unwrap();
        assert_eq!(res, Cow::Borrowed(""))
    }

    #[test]
    fn strip_non_semantic_ws() {
        let res = to_content::<TestSpec>("\"hy \nthere\"").unwrap();
        let expected: Cow<'static, str> = Cow::Owned("hy there".into());
        assert_eq!(res, expected);
    }

    #[test]
    fn tailing_escape() {
        let res = to_content::<TestSpec>(r#""ab\""#);
        assert_eq!(res, Err(TestError::TailingEscape));
    }

    #[test]
    fn missing_escape() {
        let res = to_content::<TestSpec>("\"a\"\"");
        assert_eq!(res, Err(TestError::EscapeMissing));
    }

}
