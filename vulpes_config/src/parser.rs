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

#[derive(Debug, PartialEq)]
pub struct ParsedConfig {
    pub label: String,
    pub value: ParsedValue,
}

#[derive(Debug, PartialEq)]
pub enum ParsedValue {
    Block(Vec<ParsedConfig>),
    Value(Vec<ParsedValue>),
    String(String),
}

impl ParsedValue {
    pub fn get_string(&self) -> Result<&str, ()> {
        match self {
            ParsedValue::String(v) => Ok(v),
            _ => Err(()),
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
        assert_eq!(
            result,
            vec![ParsedConfig {
                label: "http".to_owned(),
                value: ParsedValue::Block(vec![ParsedConfig {
                    label: "server".to_owned(),
                    value: ParsedValue::Block(vec![
                        ParsedConfig {
                            label: "listen".to_owned(),
                            value: ParsedValue::Value(vec![ParsedValue::String("80".to_owned())])
                        },
                        ParsedConfig {
                            label: "server_name".to_owned(),
                            value: ParsedValue::Value(vec![ParsedValue::String(
                                "example.com".to_owned()
                            )])
                        },
                        ParsedConfig {
                            label: "index".to_owned(),
                            value: ParsedValue::Value(vec![
                                ParsedValue::String("index.html".to_owned()),
                                ParsedValue::String("index.htm".to_owned()),
                            ])
                        },
                        ParsedConfig {
                            label: "location".to_owned(),
                            value: ParsedValue::Value(vec![
                                ParsedValue::String("/".to_owned()),
                                ParsedValue::Block(vec![ParsedConfig {
                                    label: "alias".to_owned(),
                                    value: ParsedValue::Value(vec![ParsedValue::String(
                                        "/var/www/html/".to_owned()
                                    )])
                                },])
                            ])
                        },
                    ])
                }]),
            }]
        );
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
}
