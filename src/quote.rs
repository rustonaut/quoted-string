use std::borrow::Cow;
// this import will become unused in future rust versions
// but won't be removed for now for supporting current
// rust versions
#[allow(unused_imports)]
use std::ascii::AsciiExt;

use spec::{QuotedStringSpec, QuotedValidator, UnquotedValidator, ValidationResult};



/// quotes the input string returning the quoted string and the used validator.
///
/// The validator is returned so that consumers can collect additional information if needed
/// without iterating over the input again, e.g. if the input was pure us-ascii.
///
/// # Example
///
/// ```
/// // use your own Spec instead
/// use quoted_string::test_utils::TestSpec;
/// use quoted_string::quote;
///
/// let (qs, _) = quote::<TestSpec>("some\"text").unwrap();
/// assert_eq!(qs, "\"some\\\"text\"");
/// ```
///
#[inline]
pub fn quote<Spec: QuotedStringSpec>(
    input: &str
) -> Result<(String, Spec::QuotedValidator), Spec::Err>
{
    let mut out = String::with_capacity(input.len()+2);
    out.push('"');
    let mut validator = Spec::new_quoted_validator();
    quote_inner::<Spec>(input, &mut out, &mut validator)?;
    out.push('"');

    Ok((out, validator))
}

/// quotes a input writing it into the output buffer, does not add surrounding '"'
///
/// if ascii_only is true and non asii chars a fround an error is returned.
///
/// If no error is returned a boolean indecating if the whole input was ascii is
/// returned.
fn quote_inner<Spec: QuotedStringSpec>(
    input: &str,
    out: &mut String,
    validator: &mut Spec::QuotedValidator
) -> Result<(), Spec::Err>
{
    for ch in input.chars() {
        quote_char_into::<Spec>(ch, validator.validate_next_char(ch), out)?;
    }
    validator.end_validation()
}

#[inline]
fn quote_char_into<Spec: QuotedStringSpec>(
    ch: char,
    validator_result: ValidationResult<Spec::Err>,
    out: &mut String
) -> Result<(), Spec::Err>
{
    use spec::ValidationResult::*;
    match validator_result {
        QText | SemanticWs | NotSemantic => {
            out.push(ch)
        }
        Quotable => {
            out.push('\\');
            out.push(ch);
        }
        Invalid(err) => {
            return Err(err);
        }
    }
    Ok(())
}

/// quotes the input string if needed
///
/// The unquoted validator (and the quoted one if it was used) are returned
/// to allow the collection and extraction of additional information if needed.
/// I.e. if a string was ascii.
///
/// # Example
///
/// ```
/// # use std::borrow::Cow;
/// // use your own Spec
/// use quoted_string::test_utils::TestSpec;
/// use quoted_string::quote_if_needed;
///
/// let (quoted, _meta) = quote_if_needed::<TestSpec>("simple")
///     .expect("only fails if input can not be represented as quoted string with used Spec");
///
/// // The used spec states a 6 character us-ascii word does not need to be represented as
/// // quoted string
/// assert_eq!(quoted, Cow::Borrowed("simple"));
///
/// let (quoted2, _meta) = quote_if_needed::<TestSpec>("more complex").unwrap();
/// let expected: Cow<'static, str> = Cow::Owned("\"more complex\"".into());
/// assert_eq!(quoted2, expected);
/// ```
///
pub fn quote_if_needed<'a, Spec: QuotedStringSpec>(
    input: &'a str,
) -> Result<(Cow<'a, str>, (Spec::UnquotedValidator, Spec::QuotedValidator)), Spec::Err>
{
    let (offset, mut meta) = scan_ahead::<Spec>(input)?;
    if offset == input.len() && meta.unq_validator.end_validation() {
        return Ok((Cow::Borrowed(input), (meta.unq_validator, meta.q_validator)))
    }
    let (ok, rest) = input.split_at(offset);
    //just guess 1/8 of the remaining chars needs escaping
    let mut out = String::with_capacity(input.len() + (rest.len() >> 3));
    out.push('"');
    out.push_str(ok);
    // we have to handle one char ourself, as it was already used with q_validator
    let one_char_offset;
    if let Some((ch, res)) = meta.last_q_validator_res {
        quote_char_into::<Spec>(ch, res, &mut out)?;
        one_char_offset = ch.len_utf8();
    } else {
        one_char_offset = 0;
    }
    quote_inner::<Spec>(&rest[one_char_offset..], &mut out, &mut meta.q_validator)?;
    out.push('"');
    Ok((Cow::Owned(out), (meta.unq_validator, meta.q_validator)))
}


struct ScanMeta<Spec: QuotedStringSpec> {
    unq_validator: Spec::UnquotedValidator,
    q_validator: Spec::QuotedValidator,
    last_q_validator_res: Option<(char, ValidationResult<Spec::Err>)>
}

fn scan_ahead<Spec: QuotedStringSpec>(
    inp: &str
) -> Result<(usize, ScanMeta<Spec>), Spec::Err>
{
    use spec::ValidationResult::*;
    let mut unq_validator = Spec::new_unquoted_validator();
    let mut q_validator = Spec::new_quoted_validator();


    for (offset, ch) in inp.char_indices() {
        //I have to drive the state of both validators:
        // though the overhead should be compiled out by the optimizer
        // except in cases where it is needed
        let q_valid = q_validator.validate_next_char(ch);
        if unq_validator.validate_next_char(ch) {
            #[cfg(debug_assertions)]
            {
                //QuotedStringSpec is expected to be implemented so that anything valid without quotations
                //is also a valid quoted string if dquotes are added, validate this with debug assertions
                match q_validator.validate_next_char(ch) {
                    QText | SemanticWs | NotSemantic |
                    Quotable => {},
                    Invalid(_err) => {
                        panic!(concat!("[BUG/QuotedStringSpec/custom_impl]",
                            " valid outside of quoting but not inside of quoting)"));
                    }
                };
            }
        } else {
            let meta = ScanMeta {
                unq_validator, q_validator,
                last_q_validator_res: Some((ch, q_valid))
            };
            return Ok((offset, meta));
        }
    }
    let meta = ScanMeta { unq_validator, q_validator, last_q_validator_res: None };
    Ok((inp.len(), meta))
}


#[cfg(test)]
mod test {
    // this import will become unused in future rust versions
    // but won't be removed for now for supporting current
    // rust versions
    #[allow(warnings)]
    use std::ascii::AsciiExt;
    use test_utils::*;
    use super::*;

    #[test]
    fn quote_simple() {
        let data = &[
            ("this is simple", "\"this is simple\""),
            ("also\tsimple", "\"also\tsimple\""),
            ("with quotes\"<-", "\"with quotes\\\"<-\""),
            ("with slash\\<-", "\"with slash\\\\<-\"")
        ];
        for &(unquoted, quoted) in data.iter() {
            let (got_quoted, _qvalidator) = quote::<TestSpec>(unquoted).unwrap();
            assert_eq!(got_quoted, quoted);
        }
    }

    #[test]
    fn quote_unquotable() {
        let res = quote::<TestSpec>("→");
        assert_eq!(res.unwrap_err(), TestError::Unquoteable);
    }

    #[test]
    fn quote_if_needed_unneded() {
        let (out, _) = quote_if_needed::<TestSpec>("abcdef").unwrap();
        assert_eq!(out, Cow::Borrowed("abcdef"));
    }

    #[test]
    fn quote_if_needed_state() {
        let (out, (uqv,_qv)) = quote_if_needed::<TestSpec>("abcde.").unwrap();
        assert_eq!(out, Cow::Borrowed("abcde."));
        assert_eq!(uqv.len, 6);
        assert_eq!(uqv.last_was_dot, true)
    }

    #[test]
    fn quote_if_needed_needed_because_char() {
        let (out, (uqv, _qv)) = quote_if_needed::<TestSpec>("ab def").unwrap();
        let expected: Cow<'static, str> = Cow::Owned("\"ab def\"".into());
        assert_eq!(out, expected);
        assert!(uqv.len >= 3);
    }

    #[test]
    fn quote_if_needed_needed_because_state() {
        let (out, (uqv, _qv)) = quote_if_needed::<TestSpec>("abc..f").unwrap();
        let expected: Cow<'static, str> = Cow::Owned("\"abc..f\"".into());
        assert_eq!(out, expected);
        assert!(uqv.len >= 5);
    }

    #[test]
    fn quote_if_needed_needed_because_end() {
        let (out, (uqv, _qv)) = quote_if_needed::<TestSpec>("a").unwrap();
        let expected: Cow<'static, str> = Cow::Owned("\"a\"".into());
        assert_eq!(out, expected);
        assert!(uqv.len >= 1);
    }

}
