use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

use crate::errors::{SgfParseError, SgfPropError};
use crate::props::utils::parse_tuple;
use crate::traits::{Game, ToSgf};
use crate::SgfNode;

// TODO: Organize this file

/// An SGF [GameTree](https://www.red-bean.com/sgf/sgf4.html#ebnf-def) value.
#[derive(Clone, Debug, PartialEq)]
pub enum GameTree {
    GoGame(SgfNode<GoGame>),
    // TODO: Add at least Unknown
}

/// TODO: docs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameType {
    Go,
    Unknown,
}

/// TODO: docs
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GoGame {}

impl GameTree {
    /// TODO: Docs and examples
    pub fn into_go_node(self) -> Result<SgfNode<GoGame>, SgfParseError> {
        match self {
            GameTree::GoGame(sgf_node) => Ok(sgf_node),
            _ => Err(SgfParseError::UnexpectedGameType),
        }
    }

    /// TODO: Docs and examples
    pub fn gametype(&self) -> GameType {
        match self {
            GameTree::GoGame(_) => GameType::Go,
        }
    }
}

impl std::fmt::Display for GameTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GoGame(sgf_node) => std::fmt::Display::fmt(sgf_node, f),
        }
    }
}

impl std::convert::From<SgfNode<GoGame>> for GameTree {
    fn from(sgf_node: SgfNode<GoGame>) -> Self {
        GameTree::GoGame(sgf_node)
    }
}

impl Game for GoGame {
    type Move = GoMove;
    type Stone = GoPoint;
    type Point = GoPoint;

    fn parse_point_list(values: &[String]) -> Result<HashSet<Self::Point>, SgfPropError> {
        let mut points = HashSet::new();
        for value in values.iter() {
            if value.contains(':') {
                let (upper_left, lower_right): (Self::Point, Self::Point) = parse_tuple(value)?;
                if upper_left.x > lower_right.x || upper_left.y > lower_right.y {
                    return Err(SgfPropError {});
                }
                for x in upper_left.x..=lower_right.x {
                    for y in upper_left.y..=lower_right.y {
                        let point = Self::Point { x, y };
                        if points.contains(&point) {
                            return Err(SgfPropError {});
                        }
                        points.insert(point);
                    }
                }
            } else {
                let point = value.parse()?;
                if points.contains(&point) {
                    return Err(SgfPropError {});
                }
                points.insert(point);
            }
        }

        Ok(points)
    }

    fn parse_stone_list(values: &[String]) -> Result<HashSet<Self::Stone>, SgfPropError> {
        GoGame::parse_point_list(values)
    }
}

/// An SGF [Point](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
///
/// TODO: Examples
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct GoPoint {
    pub x: u8,
    pub y: u8,
}

/// An SGF [Move](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
///
/// Moves may either be a pass, or a [Point](struct.Point.html)
///
/// # Examples
/// ```
/// use sgf_parse::{parse_go, SgfProp};
/// use sgf_parse::game::GoMove;
///
/// let node = parse_go("(;B[de])").unwrap().into_iter().next().unwrap();
/// for prop in node.properties() {
///     match prop {
///         SgfProp::B(GoMove::Move(point)) => println!("B move at {:?}", point),
///         _ => {}
///     }
/// }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum GoMove {
    Pass,
    Move(GoPoint),
}

impl FromStr for GoMove {
    type Err = SgfPropError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Self::Pass),
            _ => Ok(Self::Move(s.parse()?)),
        }
    }
}

impl ToSgf for GoMove {
    fn to_sgf(&self) -> String {
        match self {
            Self::Pass => "".to_string(),
            Self::Move(point) => point.to_sgf(),
        }
    }
}

impl FromStr for GoPoint {
    type Err = SgfPropError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn map_char(c: char) -> Result<u8, SgfPropError> {
            if c.is_ascii_lowercase() {
                Ok(c as u8 - b'a')
            } else if c.is_ascii_uppercase() {
                Ok(c as u8 - b'A')
            } else {
                Err(SgfPropError {})
            }
        }

        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 2 {
            return Err(SgfPropError {});
        }

        Ok(Self {
            x: map_char(chars[0])?,
            y: map_char(chars[1])?,
        })
    }
}

impl ToSgf for GoPoint {
    fn to_sgf(&self) -> String {
        format!("{}{}", (self.x + b'a') as char, (self.y + b'a') as char)
    }
}
