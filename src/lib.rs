//! Data structures and utilities for parsing [SGF FF\[4\] files](https://www.red-bean.com/sgf/).
//!
//! For reading SGFs your starting point will likely be [`go::parse`]. For parsing non-go games
//! check out the [`parse`](`parse()`) function.
//!
//! For writing SGFs check out [`SgfNode::serialize`] for writing single game trees or
//! [`serialize`](`serialize()`) for writing whole collections.

#[macro_use]
mod prop_macro;

pub mod go;
pub mod unknown_game;

mod games;
mod lexer;
mod parser;
mod props;
mod serialize;
mod sgf_node;

pub use games::{GameTree, GameType};
pub use lexer::LexerError;
pub use parser::{parse, SgfParseError};
pub use props::{Color, Double, PropertyType, SgfProp, SimpleText, Text};
pub use serialize::serialize;
pub use sgf_node::{InvalidNodeError, SgfNode};
