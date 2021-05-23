pub use super::lexer::LexerError;
pub use super::props::SgfPropError;
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
}

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
            SgfParseError::LexerError(_) => todo!(),
            SgfParseError::NodeError(_) => todo!(),
            SgfParseError::UnexpectedGameTreeStart => write!(f, "Unexpected start of game tree"),
            SgfParseError::UnexpectedGameTreeEnd => write!(f, "Unexpected end of game tree"),
            SgfParseError::UnexpectedProperty => write!(f, "Unexpected property"),
            SgfParseError::UnexpectedEndOfData => write!(f, "Unexpected end of data"),
        }
    }
}

impl std::error::Error for SgfParseError {}
