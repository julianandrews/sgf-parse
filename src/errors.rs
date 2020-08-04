#[derive(Debug)]
pub enum SgfParseError {
    InvalidGameTree(String),
    InvalidNode(String),
    InvalidProperty(String),
    InvalidPropertyValue,
}

impl std::fmt::Display for SgfParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SgfParseError::InvalidGameTree(s) => write!(f, "Invalid GameTree at '{}'", &s[..20]),
            SgfParseError::InvalidNode(s) => write!(f, "Invalid Node at '{}'", &s[..20]),
            SgfParseError::InvalidProperty(s) => write!(f, "Invalid Property at '{}'", &s[..20]),
            SgfParseError::InvalidPropertyValue => write!(f, "Invalid Property Value"),
        }
    }
}

impl std::error::Error for SgfParseError {}
