mod error;
mod parser;

pub use error::ParserError;
pub use parser::{parse, ParsedConfig, ParsedValue};
