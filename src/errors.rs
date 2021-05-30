pub use super::lexer::LexerError;
pub use super::sgf_node::InvalidNodeError;

/// Error type for failures parsing sgf from text.
#[derive(Debug)]
pub enum SgfParseError {
    LexerError(LexerError),
    NodeError(InvalidNodeError),
    UnexpectedGameTreeStart,
    UnexpectedGameTreeEnd,
    UnexpectedProperty,
    UnexpectedEndOfData,
    UnexpectedGameType,
}

/// Error type for invalid SGF properties.
#[derive(Debug)]
pub struct SgfPropError {}

impl std::fmt::Display for SgfPropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid property value")
    }
}

impl std::error::Error for SgfPropError {}

impl From<LexerError> for SgfParseError {
    fn from(error: LexerError) -> Self {
        Self::LexerError(error)
    }
}

impl From<InvalidNodeError> for SgfParseError {
    fn from(error: InvalidNodeError) -> Self {
        Self::NodeError(error)
    }
}

impl std::fmt::Display for SgfParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SgfParseError::LexerError(e) => write!(f, "Error tokenizing: {}", e),
            SgfParseError::NodeError(e) => write!(f, "Invalid node: {}", e),
            SgfParseError::UnexpectedGameTreeStart => write!(f, "Unexpected start of game tree"),
            SgfParseError::UnexpectedGameTreeEnd => write!(f, "Unexpected end of game tree"),
            SgfParseError::UnexpectedProperty => write!(f, "Unexpected property"),
            SgfParseError::UnexpectedEndOfData => write!(f, "Unexpected end of data"),
            SgfParseError::UnexpectedGameType => write!(f, "Unexpected game type"),
        }
    }
}

impl std::error::Error for SgfParseError {}
