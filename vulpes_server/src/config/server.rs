use crate::config::{
    error::{ConfigError, ErrorKind},
    location::LocationConfig,
};
use std::collections::HashMap;
use vulpes_parser::ParsedValue;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ServerConfig {
    pub listen: Vec<String>,
    pub server_name: Vec<String>,
    pub location: HashMap<String, LocationConfig>,
    pub ret: http::StatusCode,
}

impl TryFrom<ParsedValue> for ServerConfig {
    type Error = ConfigError;

    fn try_from(data: ParsedValue) -> Result<ServerConfig, ConfigError> {
        let mut c = Self::default();
        c.ret = http::StatusCode::NOT_FOUND;

        if let ParsedValue::Block(v) = data {
            log::debug!("parse value in server: {:?}", v);

            for v in v {
                match v.label.as_ref() {
                    "listen" => {
                        c.listen = v.value.try_into()?;
                    }
                    "server_name" => {
                        c.server_name = v.value.try_into()?;
                    }
                    "location" => {
                        let location: LocationConfig = v.value.try_into()?;
                        c.location.insert(location.path.clone(), location);
                    }
                    "return" => {
                        let code: u16 = v.value.try_into()?;
                        c.ret = http::StatusCode::from_u16(code)?;
                    }
                    _ => {
                        log::warn!("unknown config in server: {}", v);
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
