use nom::{
    branch::permutation,
    bytes::complete::take_while,
    character::{
        complete::{char, multispace0},
        is_alphanumeric,
    },
    combinator::peek,
    multi::many0,
    number::complete::be_u8,
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
    let (_, c) = peek(be_u8)(data)?;
    match c {
        123 => {
            let (data, result) = delimited(
                permutation((multispace0, char('{'), multispace0)),
                parse,
                permutation((multispace0, char('}'), multispace0)),
            )(data)?;
            Ok((data, ParsedValue::Block(result)))
        }
        _ => {
            let (data, c) = permutation((
                multispace0,
                terminated(take_while(is_allowed_string), char(';')),
                multispace0,
            ))(data)?;

            Ok((
                data,
                ParsedValue::Value(vec![String::from_utf8(c.1.to_vec()).unwrap()]),
            ))
        }
    }
}

fn is_allowed_string(c: u8) -> bool {
    is_alphanumeric(c) || c == b'.' || c == b'_'
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::parser::{ParsedConfig, ParsedValue};

    const CONFIG: &str = "
    http {
        listen 80;
        server_name example.com;
    }
    ";

    #[test]
    fn test_parse() {
        let (_, result) = parse(CONFIG.as_bytes()).unwrap();
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
                    }
                ]),
            }]
        );
    }
}
