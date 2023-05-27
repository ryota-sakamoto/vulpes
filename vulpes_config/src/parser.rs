use nom::{
    branch::{alt, permutation},
    bytes::complete::take_while,
    character::{
        complete::{anychar, char, multispace0, multispace1},
        is_space,
    },
    combinator::{map, peek},
    multi::{many0, separated_list1},
    sequence::{delimited, terminated},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedConfig {
    pub label: String,
    pub value: ParsedValue,
}

struct ParsedConfigWrapper<'a> {
    config: &'a ParsedConfig,
    nest: usize,
}

impl std::fmt::Display for ParsedConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            ParsedConfigWrapper {
                config: self,
                nest: 0,
            }
        )
    }
}

impl<'a> std::fmt::Display for ParsedConfigWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let prefix = "    ".repeat(self.nest);
        write!(f, "{}{} ", prefix, self.config.label).unwrap();
        write!(
            f,
            "{}",
            ParsedValueWrapper {
                value: &self.config.value,
                nest: self.nest,
            }
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParsedValue {
    Block(Vec<ParsedConfig>),
    Value(Vec<ParsedValue>),
    String(String),
}

struct ParsedValueWrapper<'a> {
    value: &'a ParsedValue,
    nest: usize,
}

impl TryInto<Vec<String>> for ParsedValue {
    type Error = ();

    fn try_into(self) -> Result<Vec<String>, ()> {
        match self {
            ParsedValue::Value(v) => {
                let mut result = Vec::with_capacity(v.len());
                for v in v {
                    if let ParsedValue::String(s) = &v {
                        result.push(s.clone());
                    } else {
                        return Err(());
                    }
                }

                return Ok(result);
            }
            _ => Err(()),
        }
    }
}

impl TryInto<u16> for ParsedValue {
    type Error = std::num::ParseIntError;

    fn try_into(self) -> Result<u16, std::num::ParseIntError> {
        match self {
            ParsedValue::Value(v) if v.len() == 1 => v[0].clone().try_into(),
            ParsedValue::String(v) => v.parse(),
            _ => "".parse(),
        }
    }
}

impl<'a> std::fmt::Display for ParsedValueWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let prefix = "    ".repeat(self.nest);
        match self.value {
            ParsedValue::Block(v) => {
                write!(f, "{{\n").unwrap();
                for v in v {
                    write!(
                        f,
                        "{}\n",
                        ParsedConfigWrapper {
                            config: v,
                            nest: self.nest + 1,
                        }
                    )
                    .unwrap();
                }
                write!(f, "{}}}", prefix).unwrap();

                Ok(())
            }
            ParsedValue::Value(v) => {
                write!(
                    f,
                    "{}",
                    v.iter()
                        .map(|v| ParsedValueWrapper {
                            value: v,
                            nest: self.nest,
                        }
                        .to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
                .unwrap();

                if let Some(ParsedValue::String(_)) = v.last() {
                    write!(f, ";").unwrap();
                }

                Ok(())
            }
            ParsedValue::String(v) => {
                write!(f, "{}", v)
            }
        }
    }
}

pub fn parse(data: &[u8]) -> IResult<&[u8], Vec<ParsedConfig>> {
    let (data, v) = many0(permutation((multispace0, parse_label, parse_value)))(data)?;

    return Ok((
        data,
        v.into_iter()
            .map(|v| ParsedConfig {
                label: String::from_utf8(v.1.to_vec()).unwrap(),
                value: v.2,
            })
            .collect(),
    ));
}

fn parse_label(data: &[u8]) -> IResult<&[u8], &[u8]> {
    let (_data, label) = take_while(is_allowed_string)(data)?;
    if label.len() == 0 {
        return Err(nom::Err::Error(nom::error::ParseError::from_error_kind(
            data,
            nom::error::ErrorKind::Eof,
        )));
    }

    return Ok((_data, label));
}

fn parse_value(data: &[u8]) -> IResult<&[u8], ParsedValue> {
    let (data, _) = multispace0(data)?;
    let (_, c) = peek(anychar)(data)?;
    match c {
        '{' => parse_block(data),
        _ => {
            let (data, result) =
                permutation((multispace0, parse_inline_multi_value, multispace0))(data)?;

            Ok((data, result.1))
        }
    }
}

fn parse_block(data: &[u8]) -> IResult<&[u8], ParsedValue> {
    map(
        delimited(
            permutation((multispace0, char('{'), multispace0)),
            parse,
            permutation((multispace0, char('}'), multispace0)),
        ),
        ParsedValue::Block,
    )(data)
}

fn parse_inline_multi_value(data: &[u8]) -> IResult<&[u8], ParsedValue> {
    map(
        alt((
            terminated(separated_list1(multispace1, parse_string), char(';')),
            separated_list1(multispace1, alt((parse_block, parse_string))),
        )),
        ParsedValue::Value,
    )(data)
}

fn parse_string(data: &[u8]) -> IResult<&[u8], ParsedValue> {
    map(take_while(is_allowed_string), |v: &[u8]| {
        ParsedValue::String(String::from_utf8(v.to_vec()).unwrap())
    })(data)
}

fn is_allowed_string(c: u8) -> bool {
    !is_space(c) && c != b';' && c != b'{' && c != b'}'
}

#[cfg(test)]
mod tests {
    use crate::parser::{parse, parse_block, parse_inline_multi_value, ParsedConfig, ParsedValue};

    fn test_config() -> ParsedConfig {
        ParsedConfig {
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
        }
    }

    #[test]
    fn test_parse() {
        let (_, result) = parse(
            "
            http {
                server {
                    listen 80;
                    server_name example.com;
                    index index.html index.htm;

                    location / {
                        alias /var/www/html/;
                    }
                }
            }"
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(result, vec![test_config()]);
    }

    #[test]
    fn test_parse_block() {
        let (data, result) = parse_block(
            "{
                listen 80;
            }"
            .as_bytes(),
        )
        .unwrap();

        assert_eq!(data, vec![]);
        assert_eq!(
            result,
            ParsedValue::Block(vec![ParsedConfig {
                label: "listen".to_owned(),
                value: ParsedValue::Value(vec![ParsedValue::String("80".to_owned())])
            }])
        );
    }

    #[test]
    fn test_parse_inline_single_value() {
        let (data, result) = parse_inline_multi_value("example.com;".as_bytes()).unwrap();

        assert_eq!(data, vec![]);
        assert_eq!(
            result,
            ParsedValue::Value(vec![ParsedValue::String("example.com".to_owned())])
        );
    }

    #[test]
    fn test_parse_inline_multi_value() {
        let (data, result) = parse_inline_multi_value("index.html index.htm;".as_bytes()).unwrap();

        assert_eq!(data, vec![]);
        assert_eq!(
            result,
            ParsedValue::Value(vec![
                ParsedValue::String("index.html".to_owned()),
                ParsedValue::String("index.htm".to_owned())
            ])
        );
    }

    #[test]
    fn test_parse_inline_value_block() {
        let (data, result) = parse_inline_multi_value(
            "/ {
                alias /var/www/html/;
            }"
            .as_bytes(),
        )
        .unwrap();

        assert_eq!(data, vec![]);
        assert_eq!(
            result,
            ParsedValue::Value(vec![
                ParsedValue::String("/".to_owned()),
                ParsedValue::Block(vec![ParsedConfig {
                    label: "alias".to_owned(),
                    value: ParsedValue::Value(vec![ParsedValue::String(
                        "/var/www/html/".to_owned()
                    )])
                },])
            ])
        );
    }

    #[test]
    fn test_try_into() {
        let data = ParsedConfig {
            label: "test".to_owned(),
            value: ParsedValue::Value(vec![
                ParsedValue::String("a".to_owned()),
                ParsedValue::String("b".to_owned()),
                ParsedValue::String("c".to_owned()),
            ]),
        };
        let result: Vec<String> = data.value.try_into().unwrap();
        assert_eq!(
            result,
            vec!["a".to_owned(), "b".to_owned(), "c".to_owned(),]
        );
    }

    #[test]
    fn test_config_to_string() {
        let data = test_config();
        assert_eq!(
            data.to_string(),
            "http {
    server {
        listen 80;
        server_name example.com;
        index index.html index.htm;
        location / {
            alias /var/www/html/;
        }
    }
}"
        );
    }
}
