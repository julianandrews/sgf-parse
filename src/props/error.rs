// Error type for invalid SGF properties.
#[derive(Debug)]
pub struct SgfPropError {}

impl std::fmt::Display for SgfPropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid property value")
    }
}

impl std::error::Error for SgfPropError {}
