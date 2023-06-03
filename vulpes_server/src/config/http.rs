use crate::config::{
    error::{ConfigError, ErrorKind},
    server::ServerConfig,
};
use vulpes_parser::ParsedValue;

#[derive(Debug, PartialEq, Default)]
pub struct HttpConfig {
    pub server: Vec<ServerConfig>,
}

impl TryFrom<ParsedValue> for HttpConfig {
    type Error = ConfigError;

    fn try_from(data: ParsedValue) -> Result<HttpConfig, ConfigError> {
        let mut c = Self::default();

        if let ParsedValue::Block(v) = data {
            for v in v {
                log::debug!("parse value in http: {:?}", v);

                match v.label.as_ref() {
                    "server" => {
                        c.server.push(ServerConfig::try_from(v.value)?);
                    }
                    _ => {
                        log::warn!("unknown label in http: {:?}", v.label);
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
