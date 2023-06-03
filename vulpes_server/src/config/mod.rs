pub mod error;
pub mod http;
pub mod location;
pub mod server;

use error::ConfigError;
use vulpes_parser::ParsedConfig;

use self::http::HttpConfig;

#[derive(Debug, PartialEq, Default)]
pub struct Config {
    pub http: Vec<HttpConfig>,
}

impl TryFrom<Vec<ParsedConfig>> for Config {
    type Error = ConfigError;

    fn try_from(data: Vec<ParsedConfig>) -> Result<Config, ConfigError> {
        let mut c = Self::default();

        for v in data {
            log::debug!("parse value in config: {:?}", v);

            match v.label.as_ref() {
                "http" => c.http.push(HttpConfig::try_from(v.value)?),
                _ => {
                    log::warn!("unknown label: {:?}", v.label);
                }
            }
        }

        return Ok(c);
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{http::HttpConfig, location::LocationConfig, server::ServerConfig, Config};
    use vulpes_parser::{ParsedConfig, ParsedValue};

    #[test]
    fn test_try_from() {
        let data = vec![ParsedConfig {
            label: "http".to_owned(),
            value: ParsedValue::Block(vec![ParsedConfig {
                label: "server".to_owned(),
                value: ParsedValue::Block(vec![
                    ParsedConfig {
                        label: "listen".to_owned(),
                        value: ParsedValue::Value(vec![ParsedValue::String("80".to_owned())]),
                    },
                    ParsedConfig {
                        label: "server_name".to_owned(),
                        value: ParsedValue::Value(vec![ParsedValue::String(
                            "example.com".to_owned(),
                        )]),
                    },
                    ParsedConfig {
                        label: "index".to_owned(),
                        value: ParsedValue::Value(vec![
                            ParsedValue::String("index.html".to_owned()),
                            ParsedValue::String("index.htm".to_owned()),
                        ]),
                    },
                    ParsedConfig {
                        label: "location".to_owned(),
                        value: ParsedValue::Value(vec![
                            ParsedValue::String("/".to_owned()),
                            ParsedValue::Block(vec![ParsedConfig {
                                label: "alias".to_owned(),
                                value: ParsedValue::Value(vec![ParsedValue::String(
                                    "/var/www/html/".to_owned(),
                                )]),
                            }]),
                        ]),
                    },
                ]),
            }]),
        }];
        let result = Config::try_from(data).unwrap();
        assert_eq!(
            result,
            Config {
                http: vec![HttpConfig {
                    server: vec![ServerConfig {
                        listen: vec!["80".to_owned()],
                        server_name: vec!["example.com".to_owned()],
                        location: std::collections::HashMap::from([(
                            "/".to_owned(),
                            LocationConfig {
                                path: "/".to_owned(),
                                ret: http::StatusCode::OK,
                            }
                        ),]),
                        ret: http::StatusCode::NOT_FOUND,
                    }]
                },]
            }
        )
    }
}
