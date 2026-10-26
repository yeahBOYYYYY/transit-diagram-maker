use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum EditError {
    NotFound(u64),
    Locked(u64),
    Invalid(String),
    Parse(String),
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::NotFound(id) => write!(f, "no layer with id {id}"),
            EditError::Locked(id) => write!(f, "layer {id} is locked"),
            EditError::Invalid(m) => write!(f, "{m}"),
            EditError::Parse(m) => write!(f, "could not read file: {m}"),
        }
    }
}

impl std::error::Error for EditError {}

pub type Result<T> = std::result::Result<T, EditError>;
