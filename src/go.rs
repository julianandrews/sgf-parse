use std::collections::HashSet;

use crate::games::Game;
use crate::props::utils::parse_tuple;
use crate::props::SgfPropError;
use crate::serialize::ToSgf;
use crate::{SgfNode, SgfParseError};

/// Returns the [`SgfNode`] values for Go games parsed from the provided text.
///
/// This is a convenience wrapper around [`parse`] for dealing with Go only collections.
///
/// # Errors
/// If the text can't be parsed as an SGF FF\[4\] collection, then an error is returned.
///
/// # Examples
/// ```
/// use sgf_parse::go::parse;
///
/// // Prints the all the properties for the two root nodes in the SGF
/// let sgf = "(;SZ[9]C[Some comment];B[de];W[fe])(;B[de];W[ff])";
/// for node in parse(&sgf).unwrap().iter() {
///     for prop in node.properties() {
///         println!("{:?}", prop);
///     }
/// }
/// ```
pub fn parse(text: &str) -> Result<Vec<SgfNode<Go>>, SgfParseError> {
    let gametrees = crate::parse(text)?;
    gametrees
        .into_iter()
        .map(|gametree| gametree.into_go_node())
        .collect::<Result<Vec<_>, _>>()
}

/// Zero-Sized type for the game of Go.
///
/// This type is used to construct [`crate::SgfNode`] and [`crate::SgfProp`] types for the game of Go.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Go {}

/// An SGF [Point](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
///
/// # Examples
/// ```
/// use sgf_parse::SgfProp;
/// use sgf_parse::go::{Go, Move, Point};
///
/// let point = Point {x: 10, y: 10};
/// let prop = SgfProp::<Go>::B(Move::Move(point));
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

/// An SGF [Move](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
///
/// # Examples
/// ```
/// use sgf_parse::SgfProp;
/// use sgf_parse::go::{parse, Move};
///
/// let node = parse("(;B[de])").unwrap().into_iter().next().unwrap();
/// for prop in node.properties() {
///     match prop {
///         SgfProp::B(Move::Move(point)) => println!("B move at {:?}", point),
///         _ => {}
///     }
/// }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Move {
    Pass,
    Move(Point),
}

impl Game for Go {
    type Move = Move;
    type Stone = Point;
    type Point = Point;

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
        Self::parse_point_list(values)
    }
}

impl ToSgf for Move {
    fn to_sgf(&self) -> String {
        match self {
            Self::Pass => "".to_string(),
            Self::Move(point) => point.to_sgf(),
        }
    }
}

impl ToSgf for Point {
    fn to_sgf(&self) -> String {
        format!("{}{}", (self.x + b'a') as char, (self.y + b'a') as char)
    }
}

impl std::str::FromStr for Move {
    type Err = SgfPropError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Self::Pass),
            _ => Ok(Self::Move(s.parse()?)),
        }
    }
}

impl std::str::FromStr for Point {
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
