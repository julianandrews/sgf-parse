//! Data structures and utilities for parsing [SGF FF\[4\] files](https://www.red-bean.com/sgf/).
//!
//! ## Quick Start
//!
//! Most common use case - parsing a Go game and iterating through its moves:
//! ```rust
//! use sgf_parse::{parse, go::Prop, go::Move};
//!
//! let sgf = "(;FF[4]GM[1]B[aa];W[ab])";
//!
//! let collection = parse(sgf).unwrap();
//! let root_node = collection.first().unwrap().as_go_node().unwrap();
//!
//! // Iterate through the main variation
//! for node in root_node.main_variation() {
//!     if let Some(prop) = node.get_move() {
//!         println!("Move: {}", prop);
//!     }
//! }
//! ```
//!
//! Working with multi-game collections:
//! ```rust
//! # use sgf_parse::parse;
//! let sgf = "(;FF[4]GM[1];B[aa])(;FF[4]GM[1];B[dd])";
//! let collection = parse(sgf).unwrap();
//!
//! for gametree in &collection {
//!     let root_node = gametree.as_go_node().unwrap();
//!     println!("Game has {} nodes", root_node.main_variation().count());
//! }
//! ```
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

mod game_tree;
mod lexer;
mod parser;
mod props;
mod serialize;
mod sgf_node;

pub use game_tree::{GameTree, GameType};
pub use lexer::LexerError;
pub use parser::{parse, parse_with_options, ParseOptions, SgfParseError};
pub use props::{Color, Double, PropertyType, SgfProp, SimpleText, Text};
pub use serialize::serialize;
pub use sgf_node::{InvalidNodeError, SgfNode};
