use super::SgfProp;

/// Error type for all sgf parsing errors.
#[derive(Debug)]
pub enum SgfParseError {
    InvalidNode(String),
    InvalidNodeProps(Vec<SgfProp>),
    ParseError(String),
    InvalidPropertyValue,
}

impl std::fmt::Display for SgfParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SgfParseError::InvalidNode(s) => write!(f, "Invalid Node: {}", s),
            SgfParseError::InvalidNodeProps(props) => {
                write!(f, "Invalid Node Properties: {:?}", props)
            }
            SgfParseError::ParseError(s) => write!(f, "Parsing error at '{}'", &s[..20]),
            SgfParseError::InvalidPropertyValue => write!(f, "Invalid Property Value"),
        }
    }
}

impl std::error::Error for SgfParseError {}
