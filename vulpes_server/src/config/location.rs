use crate::config::{
    error::{ConfigError, ErrorKind},
    types::Return,
};
use vulpes_parser::ParsedValue;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct LocationConfig {
    pub path: String,
    pub exp: LocationExp,
    pub ret: Return,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub enum LocationExp {
    #[default]
    Empty,
    Exact,
}

impl TryFrom<String> for LocationExp {
    type Error = ConfigError;

    fn try_from(value: String) -> Result<LocationExp, ConfigError> {
        match value.as_str() {
            "" => Ok(LocationExp::Empty),
            "=" => Ok(LocationExp::Exact),
            _ => Err(ConfigError {
                kind: ErrorKind::UnexpectedValue { value: value },
            }),
        }
    }
}

impl TryFrom<ParsedValue> for LocationConfig {
    type Error = ConfigError;

    fn try_from(data: ParsedValue) -> Result<LocationConfig, ConfigError> {
        let mut c = Self::default();

        if let ParsedValue::Value(mut v) = data {
            log::debug!("parse value in location: {:?}", v);

            let mut exp: Option<String> = None;
            let mut path: Option<String> = None;

            v.reverse();

            if let Some(ParsedValue::String(label)) = v.pop() {
                exp = Some(label);
            }

            if let Some(ParsedValue::String(_)) = v.last() {
                if let Some(ParsedValue::String(label)) = v.pop() {
                    path = Some(label);
                }
            }

            match (exp, path) {
                (Some(e), Some(p)) => {
                    c.exp = e.try_into()?;
                    c.path = p;
                }
                (Some(p), None) => {
                    c.path = p;
                }
                _ => {}
            }

            if let Some(ParsedValue::Block(v)) = v.pop() {
                for v in v {
                    match v.label.as_ref() {
                        "return" => {
                            c.ret = v.value.try_into()?;
                        }
                        _ => {
                            log::warn!("unknown config in location: {}", v);
                        }
                    }
                }
            }
        } else {
            return Err(ConfigError {
                kind: ErrorKind::UnexpectedType { value: data },
            });
        }

        return Ok(c);
    }
}
