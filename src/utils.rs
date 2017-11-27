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
pub fn strip_quotes(quoted_string: &str) -> Option<&str> {
    let len = quoted_string.len();
    let bytes = quoted_string.as_bytes();
    if bytes[0] == b'"' && bytes[len-1] == b'"' {
        Some(&quoted_string[1..len-1])
    } else {
        None
    }
}
