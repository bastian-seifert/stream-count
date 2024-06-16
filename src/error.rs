use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CountError {
    WrongInitialization(String),
    Message(String),
}

impl Error for CountError {}

impl Display for CountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CountError(\n")?;
        match self {
            CountError::WrongInitialization(msg) => {
                write!(f, "\tWrongInitializiation({msg})")?;
            }
            CountError::Message(msg) => {
                write!(f, "\tMessage({msg})")?;
            }
        }
        write!(f, "\n)")
    }
}

pub type CountResult<T> = Result<T, CountError>;
