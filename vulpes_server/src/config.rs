use crate::error::{ConfigError, ErrorKind};
use std::collections::HashMap;
use vulpes_parser::{ParsedConfig, ParsedValue};

#[derive(Debug, PartialEq, Default)]
pub struct Config {
    pub http: Vec<HttpConfig>,
}

#[derive(Debug, PartialEq, Default)]
pub struct HttpConfig {
    pub server: Vec<ServerConfig>,
}

#[derive(Debug, PartialEq, Default)]
pub struct ServerConfig {
    pub listen: Vec<String>,
    pub server_name: Vec<String>,
    pub location: HashMap<String, LocationConfig>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct LocationConfig {
    pub path: String,
    pub ret: http::StatusCode,
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

impl TryFrom<ParsedValue> for ServerConfig {
    type Error = ConfigError;

    fn try_from(data: ParsedValue) -> Result<ServerConfig, ConfigError> {
        let mut c = Self::default();

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

#[cfg(test)]
mod tests {
    use crate::config::{Config, HttpConfig, LocationConfig, ServerConfig};
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
                    }]
                },]
            }
        )
    }
}
