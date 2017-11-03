use std::result::{Result as StdResult};

quick_error! {

    #[derive(Debug)]
    pub enum Error {
        UnquotableCharacter(ch: char) {
            description("tried to quote a character which can not be quoted")
            display("the character {:?} can not be quoted", ch)
        }

        NonUsAsciiInput {
            description("text is expected to be us-ascii only but wasn't")
        }
    }
}

pub type Result<T> = StdResult<T, Error>;