use std::error::Error;

//TODO doc call at last one validate_* method per char
#[derive(Clone, PartialOrd, PartialEq, Hash, Debug)]
pub enum ValidationResult<Err> {
    /// a character in qtext
    QText,
    /// a normal Ws (normally `' '`,`'\t'`)
    SemanticWs,
    /// a not-semantic ws (i.e. a ws which can/should be stripped depending on context)
    NotSemanticWs,
    /// escape character `'\\'`
    Escape,
    /// quotable characters _without_ esacape (`'\\'`)
    Quotable,
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
    /// ValidatorResult::Quotable category as they are covered with other enum variants.
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

//TODO
///
pub trait QuotedValidator: Clone {
    /// carry over of the Error type of `QuotedStringSpec`(
    type Err;

    fn validate_next_char(&mut self, ch: char) -> ValidationResult<Self::Err>;


    /// ends the validation, use to check if e.g. it ends with "\r\n" missing a following " " or
    /// "\t"
    fn end_validation(&mut self) -> Result<(), Self::Err>;
}

//TODO
///
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

    /// called after validating a input, returns if the input is valid
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
