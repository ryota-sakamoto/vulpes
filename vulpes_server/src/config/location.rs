use crate::config::error::{ConfigError, ErrorKind};
use vulpes_parser::ParsedValue;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct LocationConfig {
    pub path: String,
    pub ret: http::StatusCode,
}

impl TryFrom<ParsedValue> for LocationConfig {
    type Error = ConfigError;

    fn try_from(data: ParsedValue) -> Result<LocationConfig, ConfigError> {
        let mut c = Self::default();

        if let ParsedValue::Value(mut v) = data {
            log::debug!("parse value in location: {:?}", v);

            let mut path = None;

            v.reverse();

            if let Some(ParsedValue::String(label)) = v.pop() {
                path = Some(label.to_owned());
            }

            if let Some(ParsedValue::Block(v)) = v.pop() {
                for v in v {
                    match v.label.as_ref() {
                        "return" => {
                            let code: u16 = v.value.try_into()?;
                            c.ret = http::StatusCode::from_u16(code)?;
                        }
                        _ => {
                            log::warn!("unknown config in location: {}", v);
                        }
                    }
                }
            }

            if let Some(p) = path {
                c.path = p;
            }
        } else {
            return Err(ConfigError {
                kind: ErrorKind::UnexpectedType { value: data },
            });
        }

        return Ok(c);
    }
}
