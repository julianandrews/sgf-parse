use std::ptr::NonNull;

use crate::errors::SgfParseError;
use crate::game::{GameTree, GameType, GoGame};
use crate::lexer::{tokenize, Token};
use crate::traits::Game;
use crate::SgfNode;
use crate::SgfProp;

/// Returns a Vector of the root `SgfNodes` parsed from the provided text.
///
/// Any `SgfNode` returned by this function should be valid according to the SGF
/// specification.
///
/// # Errors
/// If the text isn't a valid SGF FF\[4\] collection, then an error is returned.
///
/// # Examples
/// ```
/// use sgf_parse::parse;
/// else if
/// // Prints the all the properties for the two root nodes in the SGF
/// let sgf = "(;SZ[9]C[Some comment];B[de];W[fe])(;B[de];W[ff])";
/// for node in parse(&sgf).unwrap().iter() {
///     for prop in node.properties() {
///         println!("{:?}", prop);
///     }
/// }
/// ```
pub fn parse(text: &str) -> Result<Vec<GameTree>, SgfParseError> {
    let tokens = tokenize(text)
        .map(|result| match result {
            Err(e) => Err(SgfParseError::LexerError(e)),
            Ok((token, _span)) => Ok(token),
        })
        .collect::<Result<Vec<_>, _>>()?;
    split_by_gametree(&tokens)?
        .into_iter()
        .map(|tokens| match find_gametype(tokens)? {
            GameType::Go => parse_gametree::<GoGame>(tokens),
            GameType::Unknown => todo!(),
        })
        .collect::<Result<_, _>>()
}

// Split the tokens up into individual gametrees.
//
// This will let us easily scan each gametree for GM properties.
// Only considers StartGameTree/EndGameTree tokens.
fn split_by_gametree(tokens: &[Token]) -> Result<Vec<&[Token]>, SgfParseError> {
    let mut gametrees = vec![];
    let mut gametree_depth: u64 = 0;
    let mut slice_start = 0;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::StartGameTree => gametree_depth += 1,
            Token::EndGameTree => {
                if gametree_depth == 0 {
                    return Err(SgfParseError::UnexpectedGameTreeEnd);
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
        return Err(SgfParseError::UnexpectedEndOfData);
    }

    Ok(gametrees)
}

// Parse a single gametree of a known type.
fn parse_gametree<G: Game>(tokens: &[Token]) -> Result<GameTree, SgfParseError>
where
    SgfNode<G>: std::convert::Into<GameTree>,
{
    // TODO: Rewrite this with safe code
    let mut collection: Vec<SgfNode<G>> = vec![];
    // //// Pointer to the `Vec` of children we're currently building.
    let mut current_node_list_ptr = NonNull::new(&mut collection).unwrap();
    // Stack of pointers to incomplete `Vec`s of children.
    let mut incomplete_child_lists: Vec<NonNull<Vec<SgfNode<G>>>> = vec![];
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
                        return Err(SgfParseError::UnexpectedGameTreeStart);
                    }
                }
                incomplete_child_lists.push(current_node_list_ptr);
            }
            Token::EndGameTree => match incomplete_child_lists.pop() {
                Some(node_list) => current_node_list_ptr = node_list,
                None => return Err(SgfParseError::UnexpectedGameTreeEnd),
            },
            Token::StartNode => {
                let mut new_node = SgfNode::default();
                let mut prop_tokens = vec![];
                while let Some(Token::Property(_)) = tokens.peek() {
                    prop_tokens.push(tokens.next().unwrap());
                }
                for token in prop_tokens {
                    match token {
                        // TODO: Consider refactoring to consume tokens and avoid clones.
                        Token::Property((identifier, values)) => new_node
                            .properties
                            .push(SgfProp::new(identifier.clone(), values.clone())),
                        _ => unreachable!(),
                    }
                }
                let node_list = unsafe { current_node_list_ptr.as_mut() };
                node_list.push(new_node);
                current_node_list_ptr =
                    NonNull::new(&mut node_list.last_mut().unwrap().children).unwrap();
            }
            Token::Property(_) => return Err(SgfParseError::UnexpectedProperty),
        }
    }

    if !incomplete_child_lists.is_empty() {
        return Err(SgfParseError::UnexpectedEndOfData);
    }
    // TODO: Check exactly one in collection
    Ok(collection.into_iter().next().unwrap().into())
}

// Figure out which game to parse from a slice of tokens.
//
// This function is necessary because we need to know the game before we can do the parsing.
fn find_gametype(tokens: &[Token]) -> Result<GameType, SgfParseError> {
    let gm_props: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Property((prop_ident, prop_values)) => {
                if prop_ident == "GM" {
                    Some(prop_values)
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    match gm_props.len() {
        0 => Ok(GameType::Go),
        1 => Ok(GameType::Go), // TODO
        _ => Err(SgfParseError::UnexpectedProperty),
    }
}

#[cfg(test)]
mod test {
    use super::{parse, GameTree, GoGame, SgfNode};
    use crate::props::*;
    use crate::serialize;

    fn load_test_sgf() -> Result<Vec<GameTree>, Box<dyn std::error::Error>> {
        // See https://www.red-bean.com/sgf/examples/
        let mut sgf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        sgf_path.push("resources/test/ff4_ex.sgf");
        let data = std::fs::read_to_string(sgf_path)?;

        Ok(parse(&data)?)
    }

    fn node_depth(mut sgf_node: &SgfNode<GoGame>) -> u64 {
        let mut depth = 1;
        while sgf_node.children().count() > 0 {
            depth += 1;
            sgf_node = sgf_node.children().next().unwrap();
        }
        depth
    }

    #[test]
    fn sgf_has_two_gametrees() {
        let sgf_nodes = load_test_sgf().unwrap();
        assert_eq!(sgf_nodes.len(), 2);
    }

    #[test]
    fn gametree_one_has_five_variations() {
        let game_trees = load_test_sgf().unwrap();
        let result = game_trees[0].get_go_node();
        assert!(result.is_ok());
        let sgf_node = result.unwrap();
        assert_eq!(sgf_node.children().count(), 5);
    }

    #[test]
    fn gametree_one_has_size_19() {
        let game_trees = load_test_sgf().unwrap();
        let result = game_trees[0].get_go_node();
        assert!(result.is_ok());
        let sgf_node = result.unwrap();
        match sgf_node.get_property("SZ") {
            Some(SgfProp::SZ(size)) => assert_eq!(size, &(19, 19)),
            _ => unreachable!("Expected size property"),
        }
    }

    #[test]
    fn gametree_variation_depths() {
        let game_trees = load_test_sgf().unwrap();
        let result = game_trees[0].get_go_node();
        assert!(result.is_ok());
        let sgf_node = result.unwrap();
        let children: Vec<_> = sgf_node.children().collect();
        assert_eq!(node_depth(children[0]), 13);
        assert_eq!(node_depth(children[1]), 4);
        assert_eq!(node_depth(children[2]), 4);
    }

    #[test]
    fn gametree_two_has_one_variation() {
        let game_trees = load_test_sgf().unwrap();
        let result = game_trees[1].get_go_node();
        assert!(result.is_ok());
        let sgf_node = result.unwrap();
        assert_eq!(sgf_node.children().count(), 1);
    }

    #[test]
    fn serialize_then_parse() {
        let sgf_nodes = load_test_sgf().unwrap();
        let text = serialize(&sgf_nodes);
        assert_eq!(sgf_nodes, parse(&text).unwrap());
    }

    #[test]
    fn invalid_property() {
        let input = "(;GM[1]W[rp.pmonpoqprpsornqmpm])";
        let game_trees = parse(&input).unwrap();
        let result = game_trees[0].get_go_node();
        assert!(result.is_ok());
        let sgf_node = result.unwrap();
        let expected = vec![
            SgfProp::GM(1),
            SgfProp::Invalid("W".to_string(), vec!["rp.pmonpoqprpsornqmpm".to_string()]),
        ];
        assert_eq!(game_trees.len(), 1);
        assert_eq!(sgf_node.properties().cloned().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn stack_overflow() {
        // This input generated a stack overflow with the old code
        let input = "(;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;)";
        let result = parse(&input);
        assert!(result.is_ok());
    }
}
