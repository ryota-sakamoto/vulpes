use crate::parser::{ParsedConfig, ParsedValue};

#[derive(Debug, PartialEq, Default)]
pub struct Config {
    http: Vec<HttpConfig>,
}

#[derive(Debug, PartialEq, Default)]
pub struct HttpConfig {
    server: Vec<ServerConfig>,
}

#[derive(Debug, PartialEq, Default)]
pub struct ServerConfig {
    listen: u16,
    server_name: String,
}

impl TryFrom<Vec<ParsedConfig>> for Config {
    type Error = ();

    fn try_from(data: Vec<ParsedConfig>) -> Result<Config, ()> {
        let mut c = Self::default();

        for v in data {
            match v.label.as_ref() {
                "http" => c.http.push(HttpConfig::try_from(v.value)?),
                _ => {
                    println!("unknown label: {:?}", v.label);
                }
            }
        }

        return Ok(c);
    }
}

impl TryFrom<ParsedValue> for HttpConfig {
    type Error = ();

    fn try_from(data: ParsedValue) -> Result<HttpConfig, ()> {
        let mut c = Self::default();

        if let ParsedValue::Block(v) = data {
            for v in v {
                match v.label.as_ref() {
                    "server" => {
                        c.server.push(ServerConfig::try_from(v.value)?);
                    }
                    _ => {
                        println!("unknown label in http: {:?}", v.label);
                    }
                }
            }
        } else {
            return Err(());
        }

        return Ok(c);
    }
}

impl TryFrom<ParsedValue> for ServerConfig {
    type Error = ();

    fn try_from(data: ParsedValue) -> Result<ServerConfig, ()> {
        let mut c = Self::default();

        if let ParsedValue::Block(v) = data {
            for v in v {
                match v.label.as_ref() {
                    "listen" => {
                        let s = v.value.get_string()?;
                        c.listen = s.parse().unwrap();
                    }
                    "server_name" => {
                        c.server_name = v.value.get_string()?.to_string();
                    }
                    _ => {
                        println!("unknown label in server: {:?}", v.label);
                    }
                }
            }
        } else {
            return Err(());
        }

        return Ok(c);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{HttpConfig, ServerConfig},
        parser::{ParsedConfig, ParsedValue},
    };

    #[test]
    fn test_http_config_try_from() {
        let data = ParsedValue::Block(vec![ParsedConfig {
            label: "server".to_owned(),
            value: ParsedValue::Block(vec![
                ParsedConfig {
                    label: "listen".to_owned(),
                    value: ParsedValue::String("80".to_owned()),
                },
                ParsedConfig {
                    label: "server_name".to_owned(),
                    value: ParsedValue::String("example.com".to_owned()),
                },
            ]),
        }]);
        let result = HttpConfig::try_from(data).unwrap();
        assert_eq!(
            result,
            HttpConfig {
                server: vec![ServerConfig {
                    listen: 80,
                    server_name: "example.com".to_owned(),
                }]
            }
        )
    }

    #[test]
    fn test_server_config_try_from() {
        let data = ParsedValue::Block(vec![
            ParsedConfig {
                label: "listen".to_owned(),
                value: ParsedValue::String("80".to_owned()),
            },
            ParsedConfig {
                label: "server_name".to_owned(),
                value: ParsedValue::String("example.com".to_owned()),
            },
        ]);
        let result = ServerConfig::try_from(data).unwrap();
        assert_eq!(
            result,
            ServerConfig {
                listen: 80,
                server_name: "example.com".to_owned(),
            }
        )
    }
}
