//! Data structures and utilities for parsing [SGF FF\[4\] files](https://www.red-bean.com/sgf/).
//!
//! For most purposes your starting point will likely be the [parse](fn.parse.html) function. The
//! main interface to the sgf is the `SgfNode` struct and the associated `SgfProp` values.

mod errors;
mod parser;
mod sgf_node;
mod props;

pub use errors::SgfParseError;
pub use parser::parse;
pub use props::{SgfProp, Color, Double, Point, Move, PropertyType};
pub use sgf_node::SgfNode;
