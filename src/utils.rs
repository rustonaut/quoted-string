#[doc(hidden)]
#[macro_export]
macro_rules! assert_ok {
    ($val:expr) => ({
        match $val {
            Ok( res ) => res,
            Err( err ) => panic!( "expected Ok(..) got Err({:?})", err)
        }
    });
    ($val:expr, $ctx:expr) => ({
        match $val {
            Ok( res ) => res,
            Err( err ) => panic!( "expected Ok(..) got Err({:?}) [ctx: {:?}]", err, $ctx)
        }
    });
}

#[doc(hidden)]
#[macro_export]
macro_rules! assert_err {
    ($val:expr) => ({
        match $val {
            Ok( val ) => panic!( "expected Err(..) got Ok({:?})", val),
            Err( err ) => err,
        }
    });
    ($val:expr, $ctx:expr) => ({
        match $val {
            Ok( val ) => panic!( "expected Err(..) got Ok({:?}) [ctx: {:?}]", val, $ctx),
            Err( err ) => err,
        }
    });
}


/// strips quotes if they exists
///
/// returns None if the input does not start with `"` and ends with `"`
///
/// # Example
/// ```
/// use quoted_string::strip_quotes;
/// assert_eq!(strip_quotes("\"a b\""), Some("a b"));
/// assert_eq!(strip_quotes("a b"), None);
/// assert_eq!(strip_quotes("\"a b"), None);
/// assert_eq!(strip_quotes("a b\""), None);
/// ```
pub fn strip_quotes(quoted_string: &str) -> Option<&str> {
    let len = quoted_string.len();
    let bytes = quoted_string.as_bytes();
    //SLICE_SAFE: && shor circuites if len < 1 and by using bytes there is no problem with utf8
    // char boundaries
    if bytes.iter().next() == Some(&b'"') && bytes[len-1] == b'"' {
        //SLICE_SAFE: [0] and [len-1] are checked to be '"'
        Some(&quoted_string[1..len-1])
    } else {
        None
    }
}

#[cfg(test)]
mod test {


    mod strip_quotes {
        use super::super::strip_quotes;

        #[test]
        fn empty_string() {
            assert!(strip_quotes("").is_none());
        }

        #[test]
        fn empty_quoted_string() {
            assert_eq!(strip_quotes("\"\""), Some(""));
        }

        #[test]
        fn missing_quotes() {
            assert_eq!(strip_quotes("\"abc"), None);
            assert_eq!(strip_quotes("abc\""), None);
        }

        #[test]
        fn simple_string() {
            assert_eq!(strip_quotes("\"simple\""), Some("simple"));
        }
    }
}
