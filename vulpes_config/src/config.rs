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
    listen: Vec<String>,
    server_name: Vec<String>,
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
                        c.listen = v.value.try_into()?;
                    }
                    "server_name" => {
                        c.server_name = v.value.try_into()?;
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
        Config,
    };

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
                    }]
                },]
            }
        )
    }
}
