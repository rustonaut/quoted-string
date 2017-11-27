use spec::{QuotedStringSpec, QuotedValidator};

/// validates if input is a valid quoted-string
///
/// in difference to parse it requires the whole input to be one quoted-string
///
/// # Example
///
/// ```
/// // use your own spec
/// use quoted_string::test_utils::TestSpec;
/// use quoted_string::validate;
///
/// assert!(validate::<TestSpec>("\"quoted string\""));
/// assert!(!validate::<TestSpec>("\"not right\"really not"));
/// ```
///
pub fn validate<Spec: QuotedStringSpec>(input: &str) -> bool {
    parse::<Spec>(input)
        .map(|res|res.tail.is_empty())
        .unwrap_or(false)
}

/// the result of successfully parsing a quoted string
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parsed<'a> {
    /// the parsed quoted string
    pub quoted_string: &'a str,
    /// the rest of the input string, not parsed
    pub tail: &'a str
}

/// parse a quoted string starting at the begin of `input` but possible ending earlier
///
/// To check if the whole string is a quoted-string (an nothing more) you have to
/// additional check if `parsed.tail` is empty.
///
/// # Error
///
/// a error and the char index where it was triggered is returned if the input does not start
/// with a valid quoted-string.
///
/// # Example
///
/// ```
/// // use your own Spec
/// use quoted_string::test_utils::TestSpec;
/// use quoted_string::{parse, Parsed};
///
/// let parsed = parse::<TestSpec>("\"list of\"; \"quoted strings\"").unwrap();
/// assert_eq!(parsed, Parsed {
///     quoted_string: "\"list of\"",
///     tail:  "; \"quoted strings\""
/// });
/// ```
///
pub fn parse<Spec: QuotedStringSpec>(input: &str) -> Result<Parsed, (usize, Spec::Err)> {
    use spec::ValidationResult::*;

    let mut q_validator = Spec::new_quoted_validator();

    if input.bytes().next() != Some(b'"') {
        return Err((0, Spec::quoted_string_missing_quotes()));
    }

    let mut last_was_escape = false;
    //SLICE_SAFE: returns before if input.len() < 1 || input[1] != '"' i.e. 1.. is valid
    for (idx, ch) in input.char_indices().skip(1) {
        if last_was_escape {
            last_was_escape = false;
            q_validator.validate_is_quotable(ch)
                .map_err(|err| (idx, err))?;
        } else if ch == '"' {
            let next_char_idx = idx + 1;
            //SLICE_SAFE: char.len_utf8() == 1, so the next char start at idx+1 if it was the last
            // char it's the len of the string so in both cases ..next_char_idx and next_char_idx..
            // are fine
            return Ok(Parsed {
                quoted_string: &input[0..next_char_idx],
                tail: &input[next_char_idx..]
            })
        } else {
            match q_validator.validate_next_char(ch) {
                QText |
                SemanticWs |
                NotSemanticWs => {},
                Escape => {
                    last_was_escape = true
                }
                Quotable => {
                    return Err((idx, Spec::unquoted_quotable_char(ch)))
                }
                Invalid(err) => {
                    return Err((idx, err))
                }
            }
        }

    }
    Err((input.len(), Spec::quoted_string_missing_quotes()))

}


#[cfg(test)]
mod test {

    mod parse {
        use test_utils::*;
        use super::super::parse;

        #[test]
        fn parse_simple() {
            let parsed = parse::<TestSpec>("\"simple\"").unwrap();
            assert_eq!(parsed.quoted_string, "\"simple\"");
            assert_eq!(parsed.tail, "");
        }

        #[test]
        fn parse_with_tail() {
            let parsed = parse::<TestSpec>("\"simple\"; abc").unwrap();
            assert_eq!(parsed.quoted_string, "\"simple\"");
            assert_eq!(parsed.tail, "; abc");
        }

        #[test]
        fn parse_with_quoted_pairs() {
            let parsed = parse::<TestSpec>("\"si\\\"m\\\\ple\"").unwrap();
            assert_eq!(parsed.quoted_string, "\"si\\\"m\\\\ple\"");
            assert_eq!(parsed.tail, "");
        }

        #[test]
        fn parse_with_unnecessary_quoted_pairs() {
            let parsed = parse::<TestSpec>("\"sim\\p\\le\"").unwrap();
            assert_eq!(parsed.quoted_string, "\"sim\\p\\le\"");
            assert_eq!(parsed.tail, "");
        }

        #[test]
        fn reject_missing_quoted() {
            let res = parse::<TestSpec>("simple");
            assert_eq!(res, Err((0, TestError::QuotesMissing)));
        }

        #[test]
        fn reject_tailing_escape() {
            let res = parse::<TestSpec>("\"simple\\\"");
            assert_eq!(res, Err((9, TestError::QuotesMissing)));
        }

        #[test]
        fn reject_unquoted_quotable() {
            let res = parse::<TestSpec>("\"simp\0le\"");
            assert_eq!(res, Err((5, TestError::EscapeMissing)));
        }

        #[test]
        fn reject_missing_closing_dquotes() {
            let res = parse::<TestSpec>("\"simple");
            assert_eq!(res, Err((7, TestError::QuotesMissing)));
        }

        #[test]
        fn empty_string_does_not_panic() {
            let res = parse::<TestSpec>("");
            assert_eq!(res, Err((0, TestError::QuotesMissing)));
        }

    }

    mod validate {
        use test_utils::*;
        use super::super::validate;

        #[test]
        fn accept_valid_quoted_string() {
            assert!(validate::<TestSpec>("\"that\\\"s strange\""));
        }

        #[test]
        fn reject_invalid_quoted_string() {
            assert!(!validate::<TestSpec>("ups"))
        }

        #[test]
        fn reject_quoted_string_shorter_than_input() {
            assert!(!validate::<TestSpec>("\"nice!\"ups whats here?\""))
        }

    }



}