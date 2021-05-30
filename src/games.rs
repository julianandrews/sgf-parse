use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

use crate::SgfParseError;
use crate::{props::SgfPropError, serialize::ToSgf, SgfNode};

use crate::go::Go;
use crate::unknown_game::UnknownGame;

pub trait Game: Debug + Clone + PartialEq + Default {
    type Move: FromStr<Err = SgfPropError> + ToSgf + Debug + Clone + Hash + PartialEq + Eq;
    type Stone: FromStr<Err = SgfPropError> + ToSgf + Debug + Clone + Hash + PartialEq + Eq;
    type Point: FromStr<Err = SgfPropError> + ToSgf + Debug + Clone + Hash + PartialEq + Eq;

    fn parse_point_list(value: &[String]) -> Result<HashSet<Self::Point>, SgfPropError>;

    fn parse_stone_list(value: &[String]) -> Result<HashSet<Self::Stone>, SgfPropError>;
}

/// The game recorded in a [`GameTree`].
///
/// Any [`GameTree`] retured by [`crate::parse()`] will have a game type which corresponds to
/// the SGF `GM` property of the root node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameType {
    Go,
    Unknown,
}

/// An SGF [GameTree](https://www.red-bean.com/sgf/sgf4.html#ebnf-def) value.
///
/// This type allows creating a collection of [`SgfNode`] values for different games. This is
/// used in the return type of the [`crate::parse()`] function. Users of the
/// [`crate::serialize()`] function will need to build these.
///
/// For now, all non-Go games will parse as [`GameTree::Unknown`] which should also be used for any
/// serialization of non-Go games.
#[derive(Clone, Debug, PartialEq)]
pub enum GameTree {
    GoGame(SgfNode<Go>),
    Unknown(SgfNode<UnknownGame>),
}

impl GameTree {
    /// Consumes a Go game `GameTree` and returns the contained [SgfNode](struct.SgfNode.html).
    ///
    /// This is a convenience method for go games.
    ///
    /// # Errors
    /// Returns an error if the variant isn't a [`GameTree::GoGame`].
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::parse;
    ///
    /// let gametree = parse("(;B[de]C[A comment])").unwrap().into_iter().next().unwrap();
    /// let sgf_node = gametree.into_go_node().unwrap();
    /// ```
    pub fn into_go_node(self) -> Result<SgfNode<Go>, SgfParseError> {
        match self {
            Self::GoGame(sgf_node) => Ok(sgf_node),
            _ => Err(SgfParseError::UnexpectedGameType),
        }
    }

    /// Returns the `GameType` for this `GameTree`.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{parse, GameType};
    ///
    /// let gametree = parse("(;GM[1]B[de]C[A comment])").unwrap().into_iter().next().unwrap();
    /// assert_eq!(gametree.gametype(), GameType::Go);
    /// ```
    pub fn gametype(&self) -> GameType {
        match self {
            Self::GoGame(_) => GameType::Go,
            Self::Unknown(_) => GameType::Unknown,
        }
    }
}

impl std::fmt::Display for GameTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GoGame(sgf_node) => std::fmt::Display::fmt(sgf_node, f),
            Self::Unknown(sgf_node) => std::fmt::Display::fmt(sgf_node, f),
        }
    }
}

impl std::convert::From<SgfNode<Go>> for GameTree {
    fn from(sgf_node: SgfNode<Go>) -> Self {
        Self::GoGame(sgf_node)
    }
}

impl std::convert::From<SgfNode<UnknownGame>> for GameTree {
    fn from(sgf_node: SgfNode<UnknownGame>) -> Self {
        Self::Unknown(sgf_node)
    }
}
