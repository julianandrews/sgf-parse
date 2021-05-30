//! Data structures and utilities for parsing [SGF FF\[4\] files](https://www.red-bean.com/sgf/).
//!
//! For reading SGFs your starting point will be the [parse](fn.parse.html) function which will
//! return a `Vector` of [SgfNode](struct.SgfNode.html) structs.
//!
//! For writing SGFs you'll want to build a collection of [SgfNode](struct.SgfNode.html) structs, and then use
//! [serialize](fn.serialize.html). See [SgfNodeBuilder](struct.SgfNodeBuilder.html) and
//! [SgfNode::into_builder](struct.SgfNode.html#method.into_builder).

pub mod errors;
pub mod game;
mod lexer;
mod parser;
mod props;
mod serialize;
mod sgf_node;
mod traits;

pub use parser::{parse, parse_go};
pub use props::{Color, Double, PropertyType, SgfProp, SimpleText, Text};
pub use serialize::serialize;
pub use sgf_node::SgfNode;
