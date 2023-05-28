use crate::parser::ParsedValue;

#[derive(Debug)]
pub struct ConfigError {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedType { value: ParsedValue },
    ParseIntError(std::num::ParseIntError),
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(value: std::num::ParseIntError) -> Self {
        ConfigError {
            kind: ErrorKind::ParseIntError(value),
        }
    }
}
