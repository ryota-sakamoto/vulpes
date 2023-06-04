use super::error::ConfigError;
use std::str::FromStr;
use vulpes_parser::ParsedValue;

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub code: http::StatusCode,
    pub text: Option<String>,
}

impl Default for Return {
    fn default() -> Self {
        Return {
            code: http::StatusCode::NOT_FOUND,
            text: None,
        }
    }
}

impl TryFrom<ParsedValue> for Return {
    type Error = ConfigError;

    fn try_from(data: ParsedValue) -> Result<Return, ConfigError> {
        let mut c = Self::default();

        let mut ret: Vec<String> = data.try_into()?;
        ret.reverse();

        if let Some(code) = ret.pop() {
            c.code = http::StatusCode::from_str(&code)?;
        }

        if let Some(text) = ret.pop() {
            c.text = Some(text);
        }

        return Ok(c);
    }
}
