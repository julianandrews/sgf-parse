pub mod parse;
mod serialize;
mod values;

use std::collections::HashSet;

use crate::InvalidNodeError;

pub use serialize::ToSgf;
pub use values::{Color, Double, SimpleText, Text};

/// A type that can be used for properties in an [`SgfNode`](`crate::SgfNode`).
///
/// This trait is sealed and cannot be implemented for types outside of `sgf_parse`.
pub trait SgfProp: std::fmt::Debug + std::fmt::Display + Sized + Clone + private::Sealed {
    type Point: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash + ToSgf;
    type Stone: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash + ToSgf;
    type Move: std::fmt::Debug + Clone + PartialEq + Eq + ToSgf;

    /// Returns a new property parsed from the provided identifier and values
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    /// use sgf_parse::go::Prop;
    ///
    /// // Prop::B(Point{ x: 2, y: 3 }
    /// let prop = Prop::new("B".to_string(), vec!["cd".to_string()]);
    /// // Prop::AB(vec![Point{ x: 2, y: 3 }, Point { x: 3, y: 3 }])
    /// let prop = Prop::new("AB".to_string(), vec!["cd".to_string(), "dd".to_string()]);
    /// // Prop::Unknown("FOO", vec!["Text"])
    /// let prop = Prop::new("FOO".to_string(), vec!["Text".to_string()]);
    /// ```
    fn new(identifier: String, values: Vec<String>) -> Self;

    /// Returns a the identifier associated with the [`SgfProp`].
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    /// use sgf_parse::go::Prop;
    ///
    /// let prop = Prop::new("W".to_string(), vec!["de".to_string()]);
    /// assert_eq!(prop.identifier(), "W");
    /// let prop = Prop::new("FOO".to_string(), vec!["de".to_string()]);
    /// assert_eq!(prop.identifier(), "FOO");
    /// ```
    fn identifier(&self) -> String;

    /// Returns the [`PropertyType`] associated with the property.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{PropertyType, SgfProp};
    /// use sgf_parse::go::Prop;
    ///
    /// let prop = Prop::new("W".to_string(), vec!["de".to_string()]);
    /// assert_eq!(prop.property_type(), Some(PropertyType::Move));
    /// let prop = Prop::new("FOO".to_string(), vec!["de".to_string()]);
    /// assert_eq!(prop.property_type(), None);
    /// ```
    fn property_type(&self) -> Option<PropertyType>;

    /// Validates a set of properties.
    ///
    /// # Errors
    /// Returns an error if the collection of properties isn't valid.
    fn validate_properties(properties: &[Self], is_root: bool) -> Result<(), InvalidNodeError>;
}

/// An SGF [property type](https://www.red-bean.com/sgf/sgf4.html#2.2.1).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PropertyType {
    Move,
    Setup,
    Root,
    GameInfo,
    Inherit,
}

pub trait FromCompressedList: Sized {
    fn from_compressed_list(
        upper_left: &Self,
        lower_right: &Self,
    ) -> Result<HashSet<Self>, SgfPropError>;
}

// Error type for invalid SGF properties.
#[derive(Debug)]
pub struct SgfPropError {}

impl std::fmt::Display for SgfPropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid property value")
    }
}

impl std::error::Error for SgfPropError {}

// Prevent users from implementing the SgfProp trait.
// Because `parse` has to return an enum, with the current design, implementing
// a new game outside the crate is a mess.
//
// If you'd like to implement this trait for a new game, PR's are very welcome!
mod private {
    pub trait Sealed {}
    impl Sealed for crate::go::Prop {}
    impl Sealed for crate::unknown_game::Prop {}
    impl<'a, T> Sealed for &'a T where T: ?Sized + Sealed {}
}
