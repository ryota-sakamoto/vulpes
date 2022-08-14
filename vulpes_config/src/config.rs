use crate::parser;

#[derive(Debug, PartialEq)]
pub struct Config {}

impl Config {}

impl TryFrom<&[u8]> for Config {
    type Error = ();

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        parser::parse(data).unwrap();

        return Err(());
    }
}
