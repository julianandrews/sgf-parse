//! Data structures and utilities for parsing [SGF FF\[4\] files](https://www.red-bean.com/sgf/).
//!
//! For reading SGFs your starting point will be the [parse](fn.parse.html) function which will
//! return a `Vector` of `SgfNode` structs.
//!
//! For writing SGFs you'll want to build a collection of `SgfNode` structs, and then use
//! [serialize](fn.serialize.html). See `SgfNodeBuilder` and `SgfNode::to_builder`.

mod errors;
mod parser;
mod props;
mod serialize;
mod sgf_node;

pub use errors::SgfParseError;
pub use parser::parse;
pub use props::{Color, Double, Move, Point, PropertyType, SgfProp, SimpleText, Text};
pub use serialize::serialize;
pub use sgf_node::{SgfNode, SgfNodeBuilder};
