use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

use super::errors::SgfPropError;

pub trait ToSgf {
    fn to_sgf(&self) -> String;
}

pub trait Game: Debug + Clone + PartialEq + Default {
    type Move: FromStr<Err = SgfPropError> + ToSgf + Debug + Clone + Hash + PartialEq + Eq;
    type Stone: FromStr<Err = SgfPropError> + ToSgf + Debug + Clone + Hash + PartialEq + Eq;
    type Point: FromStr<Err = SgfPropError> + ToSgf + Debug + Clone + Hash + PartialEq + Eq;

    fn parse_point_list(value: &[String]) -> Result<HashSet<Self::Point>, SgfPropError>;

    fn parse_stone_list(value: &[String]) -> Result<HashSet<Self::Stone>, SgfPropError>;
}
