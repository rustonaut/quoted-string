use std::error::Error;

/// result of validating a quoted char representing what kind of char it is
///
/// Not that this the variant of this enum differ to there
/// grammar counterparts as they are split in distinct groups,
/// e.g. for most grammar a quotable char is a char which is not
/// `Invalid` (i.e. `QText`, `SemanticWs`, `NotSemantic`,  `NeedsQuotedPair`) and
/// `NeedsQuotedPair` only represents any chare which can be quoted but is not in `QText`,
/// `SemanticWs` or `NotSemantic`)
#[derive(Clone, PartialOrd, PartialEq, Hash, Debug)]
pub enum ValidationResult<Err> {
    /// a character in qtext
    QText,
    /// a normal whitespace (normally `' '`,`'\t'`)
    SemanticWs,
    /// a not-semantic character (i.e. a ws which can/should be stripped depending on context)
    /// or a comment
    NotSemantic,
    /// quotable characters including `'\\'`, but excluding or any char belonging to another
    /// variant of this enum
    NeedsQuotedPair,
    /// invalid character e.g. non-us-ascii in us-ascii only context
    Invalid(Err)
}


/// A Trait used for providing the spec of the quoted string
///
/// Quoted-strings are very similar in most places but sadly often differ in detail,
/// the simplest most occurring difference is which values are valid without using a
/// quoted string, but other include what kind of white space and character can be contained.
///
/// E.g. a quoted string in a Media Type can not contain any newling characters but in MIME (i.e.
/// in Mails) they can contain forward-white-spaces which can contain soft newline character.
///
///
/// # Implementation
///
/// This trait is meant to be implemented on zero-sized types, and passed around as a generic
/// parameter to functions, it does not contain any methods using `self`.
///
///
pub trait QuotedStringSpec: Clone {
    /// Error type which can be returned if thinks can not be quoted/parsed/etc.
    type Err: Error;
    /// Validator to find out if a value can be represented without quotation
    type UnquotedValidator: UnquotedValidator<Err=Self::Err>;
    /// Validator to find out if a value can be represented in a quoted string
    type QuotedValidator: QuotedValidator<Err=Self::Err>;

    /// Returns a validator which can be used to check if a value is valid without quoting
    ///
    /// quoted strings are normally a alternate form for normally more restriced syntax parts
    /// e.g. in media type parameter values can be either a token or a quoted string
    fn new_unquoted_validator() -> Self::UnquotedValidator;

    /// returns a validator which can be used to check if a quoted string is valid
    fn new_quoted_validator() -> Self::QuotedValidator;


    /// used to return an error if a unquoteable char is found
    fn unquoteable_char(ch: char) -> Self::Err;

    /// is called if a non a char appears with is validated as quotable but was not preceeded by an escape
    ///
    /// While all chars like QText and SemanticWs can also be quoted they are not part of the
    /// `ValidatorResult::NeedsQuotedPair` category as they are covered with other enum variants.
    fn unquoted_quotable_char(ch: char) -> Self::Err;

    /// is called if a tailing escape is detected
    ///
    /// if ok is returned the tailing slash is interpreted as a slash.
    ///
    /// A tailing escape is a escape at the end of a quoted string as it can not quote anything
    /// it is not valid. Nevertheless some specifications have a less strict grammar in which a
    /// tailing `r#"\"#` could be interpreted as semantically representing a "\".
    /// (E.g. for a tailing escape:  `r#"some quoted string\"#`)
    fn error_for_tailing_escape() -> Result<(), Self::Err>;

    /// returns a error indicating that the quoted string missed at last one of the surrounding quotes
    fn quoted_string_missing_quotes() -> Self::Err;
}

/// Used to "validate" a quoted string by calling `validate_next_char` on every char in order
///
/// This method is used by more or less any utility method, e.g. when used with `to_content`
/// it is used to determine which white spaces are not semantic and should not appear in the
/// content. When used with quoting it's used do determine if a character can be represented
/// in a quoted-string and if it is needed to represent it through a quoted-pair, etc.
///
/// While this trait can be implemented on a zero sized struct for most specifications,
/// some need some context to validate the next char, e.g. in a MIME header quoted strings
/// can contain (not semantic) `"\r\n "` sequences while any other appearence of "\r" and "\n"
/// is not allowed (except in the obsolete grammar in quoted pair).
///
/// Note that while the trait has to keep track of the anny additional context, the context
/// wrt. quoted-pairs is done for it. As it is also used to quote chars it's possible that
/// `validate_next_char` is called with a character only allowed in quoted pairs without beeing
/// called befor with `'\\'` (which is then added implicity by the quoting function). Any
/// implementation has to be able to handle this.
///
/// Additionally to validation instances of this type can be used to "on the fly" trace
/// additional information when a string is iterated when, e.g. parsing or quoting it.
/// A simple example for this is to keep track if a input is us-ascii only.
///
/// # Usage
///
/// This trait meant to be implemented in context of an `QuotedStringSpec` implementation
/// and is used by function of this crate for their specification specific behaviour.
pub trait QuotedValidator: Clone {
    /// carry over of the Error type of `QuotedStringSpec`(
    type Err;

    /// validate if the char is valid given the context stored in this instance
    ///
    /// (there might be no context, i.e. if this is implented on a zero-sized type, which
    ///  is fine for most specifications)
    ///
    /// # Usage
    ///
    /// Consumers of this trait have to make sure to:
    ///
    /// 1. call `validate_next_char` in a sequence, char by char not skipping any char
    ///    (except '\\' of a quoted pair if used to quote a string)
    /// 2. call `end_validation` after they are done
    /// 3. do nothing more with it afterwards (i.e. they need a new instance for a new validation)
    ///
    /// # Implementation
    ///
    /// Be aware that this method is not only used for parsing & validation but also
    /// for e.g. quoting string, in which case a this method can be called with a
    /// quotable char (e.g. `'"'`) _without_ a preceeding call with `'\\'` in valid manner.
    ///
    fn validate_next_char(&mut self, ch: char) -> ValidationResult<Self::Err>;


    /// ends the validation, use to check if e.g. it ends with `"\r\n"` missing a following `" "` or
    /// `"\t"`
    fn end_validation(&mut self) -> Result<(), Self::Err>;

    /// checks if the char is valid in a quoted-pair
    ///
    /// by default it call `validate_next_char` and checks if the char
    /// is not invalid (i.e. any qtext, ws, quotable or `'\\'`)
    ///
    /// This can be overridden to only allow quoted-pair for e.g. `'"'` and `'\\'`,
    /// but forbid any "unnecessary" quoted-pairs like e.g. `"\\a"`.
    #[inline]
    fn validate_is_quotable(&mut self, ch: char) -> Result<(), Self::Err> {
        match self.validate_next_char(ch) {
            ValidationResult::Invalid(err) => Err(err),
            _ => Ok(())
        }
    }
}


/// Used to "validate" a string by calling `validate_next_char` on every char in order
///
/// This is used to determine if a string needs to be represented as a quoted-string, or
/// if it can be represented directly. E.g. a Media Type parameter value of `abc` can be
/// represented directly without needing a quoted string
///
/// # Usage
///
/// This trait meant to be implemented in context of an `QuotedStringSpec` implementation
/// and is used by function of this crate for their specification specific behaviour.
pub trait UnquotedValidator: Clone {
    /// carry over of the Error type of `QuotedStringSpec`
    type Err;

    /// returns true if the next char in context the previoud chars is valid without quoting
    ///
    /// This function is called element by element on a sequence of chars.
    ///
    /// # Usage
    ///
    /// Consumers of this trait have to make sure to:
    ///
    /// 1. call `validate_next_char` in a sequence, char by char not skipping any char
    /// 2. call `end_validation` after they are done
    /// 3. do nothing more with it afterwards (i.e. they need a new instance for a new validation)
    ///
    /// # Implementation
    ///
    /// Implementors have to assure that even if they are used differently by consumers
    /// they neither panic, nor violate rust safety.
    ///
    /// if any context is needed to validate the char in context of the previous chars
    /// (e.g. a sequence of `"a..a"` is not allowed but `"a.a"` is) it can be passed on as
    /// the state of the used `UnquotedValidator` implementation.
    fn validate_next_char(&mut self, ch: char) -> bool;

    /// called after validating a input, returns true if the input is valid
    ///
    /// Some thinks can not be validated with only `validate_next_char` i.e. if a input can contain
    /// `"."` but it is not allowed to end with `"."`
    ///
    /// Note that it returns true if the input is valid assuming that every char of the input
    /// char sequence was passed to `validate_next_char` in order before and `validate_next_char`
    /// always returned `true` if this is not the case the return value can be unreliable.
    ///
    fn end_validation(&mut self) -> bool;
}
