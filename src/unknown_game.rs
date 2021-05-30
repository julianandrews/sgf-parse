use crate::games::Game;
use crate::props::SgfPropError;
use crate::serialize::ToSgf;
use std::collections::HashSet;

/// Zero-Sized type for unknown games.
///
/// This type is used to construct [`crate::SgfNode`] and [`crate::SgfProp`] types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnknownGame {}

/// Wrapper type around string for all UnknownGame types.
///
/// All properties are just stored as strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SgfString(String);

impl Game for UnknownGame {
    type Move = SgfString;
    type Stone = SgfString;
    type Point = SgfString;

    fn parse_point_list(values: &[String]) -> Result<HashSet<Self::Point>, SgfPropError> {
        Ok(values.iter().map(|value| value.parse().unwrap()).collect())
    }

    fn parse_stone_list(values: &[String]) -> Result<HashSet<Self::Stone>, SgfPropError> {
        Self::parse_point_list(values)
    }
}

impl ToSgf for SgfString {
    fn to_sgf(&self) -> String {
        self.0.to_owned()
    }
}

impl std::str::FromStr for SgfString {
    type Err = SgfPropError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

impl std::convert::From<&str> for SgfString {
    fn from(s: &str) -> Self {
        SgfString(s.to_owned())
    }
}
