use nom::{
    branch::permutation,
    bytes::complete::take_while,
    character::{
        complete::{anychar, char, multispace0, multispace1},
        is_alphanumeric,
    },
    combinator::peek,
    multi::{many0, separated_list1},
    sequence::{delimited, terminated},
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct ParsedConfig {
    label: String,
    value: ParsedValue,
}

#[derive(Debug, PartialEq)]
enum ParsedValue {
    Block(Vec<ParsedConfig>),
    Value(Vec<String>),
}

pub fn parse(data: &[u8]) -> IResult<&[u8], Vec<ParsedConfig>> {
    let (data, v) = many0(permutation((
        multispace0,
        take_while(is_allowed_string),
        parse_value,
    )))(data)?;

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

fn parse_value(data: &[u8]) -> IResult<&[u8], ParsedValue> {
    let (data, _) = multispace0(data)?;
    let (_, c) = peek(anychar)(data)?;
    match c {
        '{' => {
            let (data, result) = parse_block(data)?;
            Ok((data, ParsedValue::Block(result)))
        }
        _ => {
            let (data, c) =
                permutation((multispace0, parse_inline_multi_value, multispace0))(data)?;

            Ok((
                data,
                ParsedValue::Value(
                    c.1.into_iter()
                        .map(|v| String::from_utf8(v.to_vec()).unwrap())
                        .collect(),
                ),
            ))
        }
    }
}

fn parse_block(data: &[u8]) -> IResult<&[u8], Vec<ParsedConfig>> {
    delimited(
        permutation((multispace0, char('{'), multispace0)),
        parse,
        permutation((multispace0, char('}'), multispace0)),
    )(data)
}

fn parse_inline_multi_value(data: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    terminated(
        separated_list1(multispace1, take_while(is_allowed_string)),
        char(';'),
    )(data)
}

fn is_allowed_string(c: u8) -> bool {
    is_alphanumeric(c) || c == b'.' || c == b'_'
}

#[cfg(test)]
mod tests {
    use crate::parser::{parse, parse_block, parse_inline_multi_value, ParsedConfig, ParsedValue};

    #[test]
    fn test_parse() {
        let (_, result) = parse(
            "
            http {
                listen 80;
                server_name example.com;
                index index.html index.htm;
            }"
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(
            result,
            vec![ParsedConfig {
                label: "http".to_owned(),
                value: ParsedValue::Block(vec![
                    ParsedConfig {
                        label: "listen".to_owned(),
                        value: ParsedValue::Value(vec!["80".to_owned()])
                    },
                    ParsedConfig {
                        label: "server_name".to_owned(),
                        value: ParsedValue::Value(vec!["example.com".to_owned()])
                    },
                    ParsedConfig {
                        label: "index".to_owned(),
                        value: ParsedValue::Value(vec![
                            "index.html".to_owned(),
                            "index.htm".to_owned()
                        ])
                    },
                ]),
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
            vec![ParsedConfig {
                label: "listen".to_owned(),
                value: ParsedValue::Value(vec!["80".to_owned()])
            },]
        );
    }

    #[test]
    fn test_parse_inline_single_value() {
        let (data, result) = parse_inline_multi_value("example.com;".as_bytes()).unwrap();

        assert_eq!(data, vec![]);
        assert_eq!(result, vec![b"example.com"]);
    }

    #[test]
    fn test_parse_inline_multi_value() {
        let (data, result) = parse_inline_multi_value("index.html index.htm;".as_bytes()).unwrap();

        assert_eq!(data, vec![]);
        assert_eq!(
            result,
            vec!["index.html".as_bytes(), "index.htm".as_bytes()]
        );
    }
}
