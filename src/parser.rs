use std::ptr::NonNull;

use crate::go;
use crate::lexer::{tokenize, LexerError, Token};
use crate::unknown_game;
use crate::{GameTree, GameType, SgfNode, SgfProp};

/// Returns the [`GameTree`] values parsed from the provided text using default parsing options.
///
/// This function will attempt to convert non-FF\[4\] files to FF\[4\] if possible. Check out
/// [`parse_with_options`] if you want to change the default behavior.
///
/// # Errors
/// If the text can't be parsed as an SGF FF\[4\] collection, then an error is returned.
///
/// # Examples
/// ```
/// use sgf_parse::{parse, GameType};
///
/// let sgf = "(;SZ[9]C[Some comment];B[de];W[fe])(;B[de];W[ff])";
/// let gametrees = parse(sgf).unwrap();
/// assert!(gametrees.len() == 2);
/// assert!(gametrees.iter().all(|gametree| gametree.gametype() == GameType::Go));
/// ```
pub fn parse(text: &str) -> Result<Vec<GameTree>, SgfParseError> {
    parse_with_options(text, &ParseOptions::default())
}

/// Returns the [`GameTree`] values parsed from the provided text.
///
/// # Errors
/// If the text can't be parsed as an SGF FF\[4\] collection, then an error is returned.
///
/// # Examples
/// ```
/// use sgf_parse::{parse_with_options, ParseOptions, GameType, SgfParseError};
///
/// // Default options
/// let sgf = "(;SZ[9]C[Some comment];B[de];W[fe])(;B[de];W[ff])";
/// let gametrees = parse_with_options(sgf, &ParseOptions::default()).unwrap();
/// assert!(gametrees.len() == 2);
/// assert!(gametrees.iter().all(|gametree| gametree.gametype() == GameType::Go));
///
/// // Strict FF[4] identifiers
/// let sgf = "(;SZ[9]CoPyright[Julian Andrews 2025];B[de];W[fe])(;B[de];W[ff])";
/// let parse_options = ParseOptions {
///     convert_mixed_case_identifiers: false,
///     ..ParseOptions::default()
/// };
/// let result = parse_with_options(sgf, &parse_options);
/// assert_eq!(result, Err(SgfParseError::InvalidFF4Property));
/// ```
pub fn parse_with_options(
    text: &str,
    options: &ParseOptions,
) -> Result<Vec<GameTree>, SgfParseError> {
    let text = text.trim();
    let tokens = if options.lenient {
        tokenize(text)
            .take_while(Result::is_ok)
            .map(|result| result.unwrap().0)
            .collect()
    } else {
        tokenize(text)
            .map(|result| match result {
                Err(e) => Err(SgfParseError::LexerError(e)),
                Ok((token, _span)) => Ok(token),
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    split_by_gametree(&tokens, options.lenient)?
        .into_iter()
        .map(|tokens| match find_gametype(tokens)? {
            GameType::Go => parse_gametree::<go::Prop>(tokens, options),
            GameType::Unknown => parse_gametree::<unknown_game::Prop>(tokens, options),
        })
        .collect::<Result<_, _>>()
}

/// Options for parsing SGF files.
///
/// # Examples
/// See [`parse_with_options`] for usage examples.
pub struct ParseOptions {
    /// Whether to allow conversion of FF\[3\] mixed case identifiers to FF\[4\].
    ///
    /// All lower case letters are dropped.
    /// This should allow parsing any older files which are valid, but not valid FF\[4\].
    pub convert_mixed_case_identifiers: bool,
    /// Whether to use lenient parsing.
    ///
    /// In lenient mode, the parser should never return an error, but will instead parse the SGF
    /// until it hits an error, and then return whatever it's managed to parse.
    pub lenient: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        ParseOptions {
            convert_mixed_case_identifiers: true,
            lenient: false,
        }
    }
}

/// Error type for failures parsing sgf from text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SgfParseError {
    LexerError(LexerError),
    UnexpectedGameTreeStart,
    UnexpectedGameTreeEnd,
    UnexpectedProperty,
    UnexpectedEndOfData,
    UnexpectedGameType,
    InvalidFF4Property,
}

impl From<LexerError> for SgfParseError {
    fn from(error: LexerError) -> Self {
        Self::LexerError(error)
    }
}

impl std::fmt::Display for SgfParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SgfParseError::LexerError(e) => write!(f, "Error tokenizing: {e}"),
            SgfParseError::UnexpectedGameTreeStart => write!(f, "Unexpected start of game tree"),
            SgfParseError::UnexpectedGameTreeEnd => write!(f, "Unexpected end of game tree"),
            SgfParseError::UnexpectedProperty => write!(f, "Unexpected property"),
            SgfParseError::UnexpectedEndOfData => write!(f, "Unexpected end of data"),
            SgfParseError::UnexpectedGameType => write!(f, "Unexpected game type"),
            SgfParseError::InvalidFF4Property => {
                write!(
                    f,
                    "Invalid FF[4] property without `convert_mixed_case_identifiers`"
                )
            }
        }
    }
}

impl std::error::Error for SgfParseError {}

// Split the tokens up into individual gametrees.
//
// This will let us easily scan each gametree for GM properties.
// Only considers StartGameTree/EndGameTree tokens.
fn split_by_gametree(tokens: &[Token], lenient: bool) -> Result<Vec<&[Token]>, SgfParseError> {
    let mut gametrees = vec![];
    let mut gametree_depth: u64 = 0;
    let mut slice_start = 0;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::StartGameTree => gametree_depth += 1,
            Token::EndGameTree => {
                if gametree_depth == 0 {
                    if lenient {
                        break;
                    } else {
                        return Err(SgfParseError::UnexpectedGameTreeEnd);
                    }
                }
                gametree_depth -= 1;
                if gametree_depth == 0 {
                    gametrees.push(&tokens[slice_start..=i]);
                    slice_start = i + 1;
                }
            }
            _ => {}
        }
    }
    if gametree_depth != 0 {
        if lenient {
            // For lenient parsing assume all remaining tokens are part of the last gametree.
            gametrees.push(&tokens[slice_start..]);
        } else {
            return Err(SgfParseError::UnexpectedEndOfData);
        }
    }

    Ok(gametrees)
}

// Parse a single gametree of a known type.
fn parse_gametree<Prop: SgfProp>(
    tokens: &[Token],
    options: &ParseOptions,
) -> Result<GameTree, SgfParseError>
where
    SgfNode<Prop>: std::convert::Into<GameTree>,
{
    // TODO: Rewrite this without `unsafe`
    let mut collection: Vec<SgfNode<Prop>> = vec![];
    // //// Pointer to the `Vec` of children we're currently building.
    let mut current_node_list_ptr = NonNull::new(&mut collection).unwrap();
    // Stack of pointers to incomplete `Vec`s of children.
    let mut incomplete_child_lists: Vec<NonNull<Vec<SgfNode<Prop>>>> = vec![];
    //// Using pointers involves some unsafe calls, but should be ok here.
    //// Since pointers are always initialized from real structs, and those structs
    //// live for the whole function body, our only safety concern is dangling pointers.
    ////
    //// Since we build the tree traversing depth-first those structs shouldn't be
    //// modified while the pointer is live. Heap-allocated contents of their
    //// `children` may be modified, but that shouldn't change anything.

    let mut tokens = tokens.iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            Token::StartGameTree => {
                // SGF game trees must have a root node.
                if let Some(node_list_ptr) = incomplete_child_lists.last() {
                    let node_list = unsafe { node_list_ptr.as_ref() };
                    if node_list.is_empty() {
                        if options.lenient {
                            break;
                        } else {
                            return Err(SgfParseError::UnexpectedGameTreeStart);
                        }
                    }
                }
                incomplete_child_lists.push(current_node_list_ptr);
            }
            Token::EndGameTree => match incomplete_child_lists.pop() {
                Some(node_list) => current_node_list_ptr = node_list,
                None => {
                    if options.lenient {
                        break;
                    } else {
                        return Err(SgfParseError::UnexpectedGameTreeEnd);
                    }
                }
            },
            Token::StartNode => {
                let mut new_node = SgfNode::default();
                let mut prop_tokens = vec![];
                while let Some(Token::Property(_)) = tokens.peek() {
                    prop_tokens.push(tokens.next().unwrap());
                }
                for token in prop_tokens {
                    match token {
                        // TODO: Consider refactoring to consume tokens and clone of values.
                        Token::Property((identifier, values)) => {
                            let identifier = {
                                if identifier.chars().all(|c| c.is_ascii_uppercase()) {
                                    identifier.clone()
                                } else if options.convert_mixed_case_identifiers {
                                    identifier
                                        .chars()
                                        .filter(|c| c.is_ascii_uppercase())
                                        .collect()
                                } else if options.lenient {
                                    break;
                                } else {
                                    return Err(SgfParseError::InvalidFF4Property);
                                }
                            };
                            new_node
                                .properties
                                .push(Prop::new(identifier, values.clone()))
                        }
                        _ => unreachable!(),
                    }
                }
                let node_list = unsafe { current_node_list_ptr.as_mut() };
                node_list.push(new_node);
                current_node_list_ptr =
                    NonNull::new(&mut node_list.last_mut().unwrap().children).unwrap();
            }
            Token::Property(_) => {
                if options.lenient {
                    break;
                } else {
                    return Err(SgfParseError::UnexpectedProperty);
                }
            }
        }
    }

    if !options.lenient && (!incomplete_child_lists.is_empty() || collection.len() != 1) {
        return Err(SgfParseError::UnexpectedEndOfData);
    }
    let mut root_node = if options.lenient {
        // A valid game tree must have at least a single (empty) node. So make one!
        collection.into_iter().next().unwrap_or_default()
    } else {
        collection
            .into_iter()
            .next()
            .ok_or(SgfParseError::UnexpectedEndOfData)?
    };
    root_node.is_root = true;
    Ok(root_node.into())
}

// Figure out which game to parse from a slice of tokens.
//
// This function is necessary because we need to know the game before we can do the parsing.
fn find_gametype(tokens: &[Token]) -> Result<GameType, SgfParseError> {
    match find_gametree_root_prop_values("GM", tokens)? {
        None => Ok(GameType::Go),
        Some(values) => {
            if values.len() != 1 {
                return Ok(GameType::Unknown);
            }
            match values[0].as_str() {
                "1" => Ok(GameType::Go),
                _ => Ok(GameType::Unknown),
            }
        }
    }
}

// Find the property values for a given identifier in the root node from the gametree's tokens.
//
// We use this to determine key root properties (like GM and FF) before parsing.
// Returns an error if there's more than one match.
fn find_gametree_root_prop_values<'a>(
    prop_ident: &'a str,
    tokens: &'a [Token],
) -> Result<Option<&'a Vec<String>>, SgfParseError> {
    // Find the matching property values in the first node.
    // Skip the initial StartGameTree, StartNode tokens; we'll handle any errors later.
    let matching_tokens: Vec<&Vec<String>> = tokens
        .iter()
        .skip(2)
        .take_while(|&token| matches!(token, Token::Property(_)))
        .filter_map(move |token| match token {
            Token::Property((ident, values)) if ident == prop_ident => Some(values),
            _ => None,
        })
        .collect();

    match matching_tokens.len() {
        0 => Ok(None),
        1 => Ok(Some(matching_tokens[0])),
        _ => Err(SgfParseError::UnexpectedProperty),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{go, serialize};

    fn load_test_sgf() -> Result<String, Box<dyn std::error::Error>> {
        // See https://www.red-bean.com/sgf/examples/
        let mut sgf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        sgf_path.push("resources/test/ff4_ex.sgf");
        let data = std::fs::read_to_string(sgf_path)?;

        Ok(data)
    }

    fn get_go_nodes() -> Result<Vec<SgfNode<go::Prop>>, Box<dyn std::error::Error>> {
        let data = load_test_sgf()?;

        Ok(go::parse(&data)?)
    }

    fn node_depth(mut sgf_node: &SgfNode<go::Prop>) -> u64 {
        let mut depth = 1;
        while sgf_node.children().count() > 0 {
            depth += 1;
            sgf_node = sgf_node.children().next().unwrap();
        }
        depth
    }

    #[test]
    fn sgf_has_two_gametrees() {
        let sgf_nodes = get_go_nodes().unwrap();
        assert_eq!(sgf_nodes.len(), 2);
    }

    #[test]
    fn gametree_one_has_five_variations() {
        let sgf_nodes = get_go_nodes().unwrap();
        let sgf_node = &sgf_nodes[0];
        assert_eq!(sgf_node.children().count(), 5);
    }

    #[test]
    fn gametree_one_has_size_19() {
        let sgf_nodes = get_go_nodes().unwrap();
        let sgf_node = &sgf_nodes[0];
        match sgf_node.get_property("SZ") {
            Some(go::Prop::SZ(size)) => assert_eq!(size, &(19, 19)),
            _ => unreachable!("Expected size property"),
        }
    }

    #[test]
    fn gametree_variation_depths() {
        let sgf_nodes = get_go_nodes().unwrap();
        let sgf_node = &sgf_nodes[0];
        let children: Vec<_> = sgf_node.children().collect();
        assert_eq!(node_depth(children[0]), 13);
        assert_eq!(node_depth(children[1]), 4);
        assert_eq!(node_depth(children[2]), 4);
    }

    #[test]
    fn gametree_two_has_one_variation() {
        let sgf_nodes = get_go_nodes().unwrap();
        let sgf_node = &sgf_nodes[1];
        assert_eq!(sgf_node.children().count(), 1);
    }

    #[test]
    fn serialize_then_parse() {
        let data = load_test_sgf().unwrap();
        let gametrees = parse(&data).unwrap();
        let text = serialize(&gametrees);
        assert_eq!(gametrees, parse(&text).unwrap());
    }

    #[test]
    fn invalid_property() {
        let input = "(;GM[1]W[rp.pmonpoqprpsornqmpm])";
        let sgf_nodes = go::parse(input).unwrap();
        let expected = vec![
            go::Prop::GM(1),
            go::Prop::Invalid("W".to_string(), vec!["rp.pmonpoqprpsornqmpm".to_string()]),
        ];

        assert_eq!(sgf_nodes.len(), 1);
        let sgf_node = &sgf_nodes[0];
        assert_eq!(sgf_node.properties().cloned().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn unknown_game() {
        let input = "(;GM[37]W[rp.pmonpoqprpsornqmpm])";
        let gametrees = parse(input).unwrap();
        assert_eq!(gametrees.len(), 1);
        assert_eq!(gametrees[0].gametype(), GameType::Unknown);
        let sgf_node = match &gametrees[0] {
            GameTree::Unknown(node) => node,
            _ => panic!("Unexpected game type"),
        };
        let expected = vec![
            unknown_game::Prop::GM(37),
            unknown_game::Prop::W("rp.pmonpoqprpsornqmpm".into()),
        ];

        assert_eq!(sgf_node.properties().cloned().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn mixed_games() {
        let input = "(;GM[1];W[dd])(;GM[37]W[rp.pmonpoqprpsornqmpm])";
        let gametrees = parse(input).unwrap();
        assert_eq!(gametrees.len(), 2);
        assert_eq!(gametrees[0].gametype(), GameType::Go);
        assert_eq!(gametrees[1].gametype(), GameType::Unknown);
    }

    #[test]
    fn stack_overflow() {
        // This input generated a stack overflow with the old code
        let input = "(;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;)";
        let result = parse(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn converts_up_ff3_property() {
        let input = "(;GM[1]FF[3]CoPyright[test])";
        let expected = vec![
            go::Prop::GM(1),
            go::Prop::FF(3),
            go::Prop::CP("test".into()),
        ];

        let sgf_nodes = go::parse(input).unwrap();

        assert_eq!(sgf_nodes.len(), 1);
        let properties = sgf_nodes[0].properties().cloned().collect::<Vec<_>>();
        assert_eq!(properties, expected);
    }

    #[test]
    fn doesnt_convert_if_not_allowed() {
        let input = "(;GM[1]FF[3]CoPyright[test])";
        let parse_options = ParseOptions {
            convert_mixed_case_identifiers: false,
            ..ParseOptions::default()
        };
        let result = parse_with_options(input, &parse_options);
        assert_eq!(result, Err(SgfParseError::InvalidFF4Property));
    }

    #[test]
    fn compressed_list_for_unknown_game() {
        let input = "(;GM[]MA[a:b])";
        let gametree = parse(&input).unwrap().pop().unwrap();
        let node = match gametree {
            GameTree::Unknown(node) => node,
            _ => panic!("Expected Unknown Game type"),
        };
        match node.get_property("MA") {
            Some(unknown_game::Prop::MA(values)) => {
                assert_eq!(values.len(), 1);
                assert!(values.contains("a:b"));
            }
            _ => panic!("MA prop not found"),
        }
    }

    #[test]
    fn strips_whitespace() {
        let input = "\n(;GM[1];B[cc])";
        let sgf_nodes = go::parse(&input).unwrap();
        assert_eq!(sgf_nodes.len(), 1);
    }

    #[test]
    fn lenient_parsing_unclosed_parens_ok() {
        let input = "\n(;GM[1];B[cc]";
        let parse_options = ParseOptions {
            lenient: true,
            ..ParseOptions::default()
        };
        let game_trees = parse_with_options(input, &parse_options).unwrap();
        assert_eq!(game_trees.len(), 1);
    }

    #[test]
    fn lenient_parsing_ignores_trailing_garbage() {
        let input = "\n(;GM[1];B[cc]))";
        let parse_options = ParseOptions {
            lenient: true,
            ..ParseOptions::default()
        };
        let game_trees = parse_with_options(input, &parse_options).unwrap();
        assert_eq!(game_trees.len(), 1);
    }

    #[test]
    fn lenient_parsing_handles_unescaped_property_end() {
        let input = "(;B[cc];W[dd];C[username [12k]: foo])";
        let parse_options = ParseOptions {
            lenient: true,
            ..ParseOptions::default()
        };
        let game_trees = parse_with_options(input, &parse_options).unwrap();
        assert_eq!(game_trees.len(), 1);
        let sgf_node = game_trees[0].as_go_node().unwrap();
        // Should parse up through "[12k]" successfully
        assert_eq!(sgf_node.main_variation().count(), 3);
    }

    #[test]
    fn lenient_parsing_handles_unclosed_property_value() {
        let input = "(;B[cc];W[dd];B[ee";
        let parse_options = ParseOptions {
            lenient: true,
            ..ParseOptions::default()
        };
        let game_trees = parse_with_options(input, &parse_options).unwrap();
        assert_eq!(game_trees.len(), 1);
        let sgf_node = game_trees[0].as_go_node().unwrap();
        // Should find 3 nodes. The last unfinished node has no properties since "B[ee" is unclosed.
        assert_eq!(sgf_node.main_variation().count(), 3);
        assert_eq!(
            sgf_node.main_variation().last().unwrap().properties.len(),
            0
        );
    }

    #[test]
    fn lenient_parsing_handles_missing_property_value() {
        let input = "(;B[cc];W[dd];B";
        let parse_options = ParseOptions {
            lenient: true,
            ..ParseOptions::default()
        };
        let game_trees = parse_with_options(input, &parse_options).unwrap();
        assert_eq!(game_trees.len(), 1);
        let sgf_node = game_trees[0].as_go_node().unwrap();
        // Should find 3 nodes. The last node has no properties since "B" is missing its value
        assert_eq!(sgf_node.main_variation().count(), 3);
        assert_eq!(
            sgf_node.main_variation().last().unwrap().properties.len(),
            0
        );
    }

    #[test]
    fn lenient_parsing_handles_missing_first_node_start() {
        let input = "(B[cc])";
        let parse_options = ParseOptions {
            lenient: true,
            ..ParseOptions::default()
        };
        let game_trees = parse_with_options(input, &parse_options).unwrap();
        assert_eq!(game_trees.len(), 1);
        let sgf_node = game_trees[0].as_go_node().unwrap();
        // A single empty node.
        assert_eq!(sgf_node.main_variation().count(), 1);
        assert_eq!(
            sgf_node.main_variation().last().unwrap().properties.len(),
            0
        );
    }
}
