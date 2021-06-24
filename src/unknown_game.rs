//! Generic types for SGFs without a known game.
//!
//! This module contains a generic [`SgfProp`] implementation appropriate
//! for use with any SGF file. This implementation recognizes all [general
//! properties](https://www.red-bean.com/sgf/properties.html), but any game
//! specific property will parse as [`Prop::Unknown`].
//!
//! SGF Move, Point, and Stone values are all simply stored as strings.

use crate::props::{FromCompressedList, PropertyType, SgfPropError, ToSgf};
use crate::{InvalidNodeError, SgfProp};
use std::collections::HashSet;

sgf_prop! {
    Prop, String, String, String,
    { }
}

impl SgfProp for Prop {
    type Move = String;
    type Point = String;
    type Stone = String;

    fn new(identifier: String, values: Vec<String>) -> Self {
        Self::parse_general_prop(identifier, values)
    }

    fn identifier(&self) -> String {
        match self.general_identifier() {
            Some(identifier) => identifier,
            None => panic!("Unimplemented identifier for {:?}", self),
        }
    }

    fn property_type(&self) -> Option<PropertyType> {
        self.general_property_type()
    }

    fn validate_properties(properties: &[Self], is_root: bool) -> Result<(), InvalidNodeError> {
        Self::general_validate_properties(properties, is_root)
    }
}

impl std::fmt::Display for Prop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prop_string = match self.serialize_prop_value() {
            Some(s) => s,
            None => panic!("Unimplemented identifier for {:?}", self),
        };
        write!(f, "{}[{}]", self.identifier(), prop_string)
    }
}

impl FromCompressedList for String {
    fn from_compressed_list(_ul: &Self, _lr: &Self) -> Result<HashSet<Self>, SgfPropError> {
        unimplemented!();
    }
}

impl ToSgf for String {
    fn to_sgf(&self) -> String {
        self.to_owned()
    }
}
