use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CountError {
    WrongInitialization(String),
    Message(String),
}

impl Error for CountError {}

impl Display for CountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CountError(")?;
        match self {
            CountError::WrongInitialization(msg) => {
                writeln!(f, "\tWrongInitializiation({msg})")?;
            }
            CountError::Message(msg) => {
                writeln!(f, "\tMessage({msg})")?;
            }
        }
        write!(f, ")")
    }
}

pub type CountResult<T> = Result<T, CountError>;
