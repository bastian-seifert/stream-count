use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CountError {
    WrongInitialization(String),
}

impl Error for CountError {}

impl Display for CountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CountError::WrongInitialization(msg) => {
                write!(f, "CountError(WrongInitializiation({msg}))")
            }
        }
    }
}

pub type CountResult<T> = Result<T, CountError>;
