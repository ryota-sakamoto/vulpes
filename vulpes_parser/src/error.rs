use crate::parser::ParsedValue;

#[derive(Debug)]
pub struct ParserError {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedType { value: ParsedValue },
}
