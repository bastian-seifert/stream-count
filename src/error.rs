#[derive(Debug)]
pub enum CountError {
    WrongInitialization(String),
}

pub type CountResult<T> = Result<T, CountError>;
