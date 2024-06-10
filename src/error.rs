use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CountError {
    WrongInitialization(String),
    Message(String),
}

impl Error for CountError {}

impl Display for CountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CountError::WrongInitialization(msg) => {
                write!(f, "CountError(WrongInitializiation({msg}))")
            }
            CountError::Message(msg) => {
                write!(f, "CountError(Message({msg}))")
            }
        }
    }
}

pub type CountResult<T> = Result<T, CountError>;
