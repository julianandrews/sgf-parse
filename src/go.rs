//! Types specific to the game of Go.
//!
//! This module contains a go-specific [`SgfProp`] implementation which
//! includes go specific properties (HA, KM, TB, TW). Point and Stone values
//! map to [`Point`], and Move values map to [`Move`]. Properties with
//! invalid moves or points map to [`Prop::Invalid`] (as do any invalid
//! [general properties](https://www.red-bean.com/sgf/properties.html)).
//!
//! This module also includes a convenience [`parse`] function which fails
//! on non-go games and returns the [`SgfNode`] values directly instead of
//! returning [`GameTree`](crate::GameTree) values.
use std::collections::HashSet;

use crate::props::parse::{parse_elist, parse_single_value};
use crate::props::{FromCompressedList, PropertyType, SgfPropError, ToSgf};
use crate::{InvalidNodeError, SgfNode, SgfParseError, SgfProp};

/// Returns the [`SgfNode`] values for Go games parsed from the provided text.
///
/// This is a convenience wrapper around [`crate::parse`] for dealing with Go only collections.
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
pub fn parse(text: &str) -> Result<Vec<SgfNode<Prop>>, SgfParseError> {
    let gametrees = crate::parse(text)?;
    gametrees
        .into_iter()
        .map(|gametree| gametree.into_go_node())
        .collect::<Result<Vec<_>, _>>()
}

/// An SGF [Point](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
///
/// # Examples
/// ```
/// use sgf_parse::go::{Prop, Move, Point};
///
/// let point = Point {x: 10, y: 10};
/// let prop = Prop::B(Move::Move(point));
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

/// An SGF [Stone](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
pub type Stone = Point;

/// An SGF [Move](https://www.red-bean.com/sgf/go.html#types) value for the Game of Go.
///
/// # Examples
/// ```
/// use sgf_parse::go::{parse, Move, Prop};
///
/// let node = parse("(;B[de])").unwrap().into_iter().next().unwrap();
/// for prop in node.properties() {
///     match prop {
///         Prop::B(Move::Move(point)) => println!("B move at {:?}", point),
///         _ => {}
///     }
/// }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Move {
    Pass,
    Move(Point),
}

sgf_prop! {
    Prop, Move, Point, Point,
    {
        HA(i64),
        KM(f64),
        TB(HashSet<Point>),
        TW(HashSet<Point>),
    }
}

impl SgfProp for Prop {
    type Point = Point;
    type Stone = Stone;
    type Move = Move;

    fn new(identifier: String, values: Vec<String>) -> Self {
        match Prop::parse_general_prop(identifier, values) {
            Self::Unknown(identifier, values) => match &identifier[..] {
                "KM" => parse_single_value(&values)
                    .map_or_else(|_| Self::Invalid(identifier, values), Self::KM),

                "HA" => match parse_single_value(&values) {
                    Ok(value) => {
                        if value < 2 {
                            Self::Invalid(identifier, values)
                        } else {
                            Self::HA(value)
                        }
                    }
                    _ => Self::Invalid(identifier, values),
                },
                "TB" => parse_elist(&values)
                    .map_or_else(|_| Self::Invalid(identifier, values), Self::TB),
                "TW" => parse_elist(&values)
                    .map_or_else(|_| Self::Invalid(identifier, values), Self::TW),
                _ => Self::Unknown(identifier, values),
            },
            prop => prop,
        }
    }

    fn identifier(&self) -> String {
        match self.general_identifier() {
            Some(identifier) => identifier,
            None => match self {
                Self::KM(_) => "KM".to_string(),
                Self::HA(_) => "HA".to_string(),
                Self::TB(_) => "TB".to_string(),
                Self::TW(_) => "TW".to_string(),
                _ => panic!("Unimplemented identifier for {:?}", self),
            },
        }
    }

    fn property_type(&self) -> Option<PropertyType> {
        match self.general_property_type() {
            Some(property_type) => Some(property_type),
            None => match self {
                Self::HA(_) => Some(PropertyType::GameInfo),
                Self::KM(_) => Some(PropertyType::GameInfo),
                _ => None,
            },
        }
    }

    fn validate_properties(properties: &[Self], is_root: bool) -> Result<(), InvalidNodeError> {
        Self::general_validate_properties(properties, is_root)
    }
}

impl std::fmt::Display for Prop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prop_string = match self.serialize_prop_value() {
            Some(s) => s,
            None => match self {
                Self::HA(x) => x.to_sgf(),
                Self::KM(x) => x.to_sgf(),
                Self::TB(x) => x.to_sgf(),
                Self::TW(x) => x.to_sgf(),
                _ => panic!("Unimplemented identifier for {:?}", self),
            },
        };
        write!(f, "{}[{}]", self.identifier(), prop_string)
    }
}

impl FromCompressedList for Point {
    fn from_compressed_list(ul: &Self, lr: &Self) -> Result<HashSet<Self>, SgfPropError> {
        let mut points = HashSet::new();
        if ul.x > lr.x || ul.y > lr.y {
            return Err(SgfPropError {});
        }
        for x in ul.x..=lr.x {
            for y in ul.y..=lr.y {
                let point = Self { x, y };
                if points.contains(&point) {
                    return Err(SgfPropError {});
                }
                points.insert(point);
            }
        }
        Ok(points)
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
