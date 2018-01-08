use std::fmt::Debug;
use error::CoreError;

pub trait GeneralQSSpec: Clone+Debug {
    type Quoting: QuotingClassifier;
    type Parsing: ParsingImpl;
}

pub trait QuotingClassifier {
    fn classify_for_quoting(pcp: PartialCodePoint) -> QuotingClass;
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum QuotingClass {
    QText,
    NeedsQuoting,
    Invalid
}

pub trait WithoutQuotingValidator {
    /// if next returns false, it's state should NOT be modified i.e. calling
    /// .end() after next(..) returned false corresponds to the input sequence _until_ next(..) was false
    fn next(&mut self, pcp: PartialCodePoint) -> bool;

    /// this is called once the validation through next ended
    ///
    /// - the validation might end because there is no more input
    /// - but it also might end because next returned false, due to the
    ///   definition of next to not change the state if it returns false
    ///   this can and is done
    /// - it _does not_ need to validate that the length is at last 1, this
    ///   is done by the algorithm using it
    /// - so for many cases this is just true (the default impl)
    fn end(&self) -> bool { true }
}


#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum State<T: Copy+Eq+Debug> {
    Start,
    Normal,
    Failed,
    QPStart,
    Custom(T),
    End
}


pub trait ParsingImpl: Copy+Eq+Debug {
    fn can_be_quoted(bch: PartialCodePoint) -> bool;
    fn handle_normal_state(bch: PartialCodePoint) -> Result<(State<Self>, bool), CoreError>;
    fn advance(&self, _pcp: PartialCodePoint) -> Result<(State<Self>, bool), CoreError> {
        unreachable!("[BUG] custom state is not used, so advance is unreachable")
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ScanAutomaton<T: ParsingImpl> {
    state: State<T>,
    last_was_emit: bool
}

impl<Impl> ScanAutomaton<Impl>
    where Impl: ParsingImpl
{

    pub fn new() -> Self {
        ScanAutomaton { state: State::Start, last_was_emit: false }
    }

    pub fn did_end(&self) -> bool {
        self.state == State::End
    }

    pub fn end(&mut self) -> Result<(), CoreError> {
        if self.did_end() {
            Ok(())
        } else {
            Err(CoreError::DoesNotEndWithDQuotes.into())
        }
    }

    pub fn advance(&mut self, pcp: PartialCodePoint) -> Result<bool, CoreError> {
        match _advance_scan_automaton(self.state, pcp) {
            Ok((state, emit)) => {
                self.state = state;
                self.last_was_emit = emit;
                Ok(emit)
            },
            Err(err) => {
                self.state = State::Failed;
                Err(err)
            }
        }
    }
}

fn _advance_scan_automaton<Impl: ParsingImpl>(state: State<Impl>, pcp: PartialCodePoint)
    -> Result<(State<Impl>, bool), CoreError>
{
    use self::State::*;
    let pcp_val = pcp.as_u8();
    match state {
        Start => {
            if pcp_val == b'"' {
                Ok((Normal, false))
            } else {
                Err(CoreError::DoesNotStartWithDQuotes)
            }
        }
        Normal => {
            match pcp_val {
                b'"' => Ok((End, false)),
                b'\\' => Ok((QPStart, false)),
                _ => Impl::handle_normal_state(pcp)
            }
        }
        QPStart => {
            if Impl::can_be_quoted(pcp) {
                Ok((Normal, true))
            } else {
                Err(CoreError::UnquoteableCharQuoted.into())
            }
        }
        Custom(inner) => {
            inner.advance(pcp)
        }
        End => {
            Err(CoreError::QuotedStringAlreadyEnded.into())
        },
        Failed => Err(CoreError::AdvancedFailedAutomaton.into())
    }
}


#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct PartialCodePoint(u8);
impl PartialCodePoint {
    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self.0
    }
    #[inline(always)]
    pub fn from_utf8_byte(u8b: u8) -> PartialCodePoint {
        debug_assert!(u8b != 0xFF, "utf8 bytes can not be 0xFF");
        PartialCodePoint(u8b)
    }
    #[inline]
    pub fn from_code_point(code_point: u32) -> PartialCodePoint {
        if code_point > 0x7f {
            PartialCodePoint(0xFF)
        } else {
            PartialCodePoint(code_point as u8)
        }
    }
}


