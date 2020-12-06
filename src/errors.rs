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
            Self::InvalidNode(s) => write!(f, "Invalid Node: {}", s),
            Self::InvalidNodeProps(props) => write!(f, "Invalid Node Properties: {:?}", props),
            Self::ParseError(s) => {
                let context = s.chars().take(20).collect::<String>();
                match context.len() {
                    0 => write!(f, "Unexpected end of file"),
                    _ => write!(f, "Parsing error at '{}'", context),
                }
            }
            Self::InvalidPropertyValue => write!(f, "Invalid Property Value"),
        }
    }
}

impl std::error::Error for SgfParseError {}
