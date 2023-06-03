use vulpes_parser::{ParsedValue, ParserError};

#[derive(Debug)]
pub struct ConfigError {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedType { value: ParsedValue },
    ParserError(ParserError),
    ParseIntError(std::num::ParseIntError),
    InvalidStatusCode(http::status::InvalidStatusCode),
}

impl From<ParserError> for ConfigError {
    fn from(value: ParserError) -> Self {
        ConfigError {
            kind: ErrorKind::ParserError(value),
        }
    }
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(value: std::num::ParseIntError) -> Self {
        ConfigError {
            kind: ErrorKind::ParseIntError(value),
        }
    }
}

impl From<http::status::InvalidStatusCode> for ConfigError {
    fn from(value: http::status::InvalidStatusCode) -> Self {
        ConfigError {
            kind: ErrorKind::InvalidStatusCode(value),
        }
    }
}
