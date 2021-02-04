use std::ptr::NonNull;

use super::errors::SgfParseError;
use super::lexer::{Lexer, Token};
use super::sgf_node::{SgfNode, SgfNodeBuilder};

/// Returns a Vector of the root `SgfNodes` parsed from the provided text.
///
/// Any `SgfNode` returned by this function should be valid according to the SGF
/// specification.
///
/// # Errors
/// If the text isn't a valid SGF FF[4] collection, then an error is returned.
///
/// # Examples
/// ```
/// use sgf_parse::parse;
///
/// // Prints the all the properties for the two root nodes in the SGF
/// let sgf = "(;SZ[9]C[Some comment];B[de];W[fe])(;B[de];W[ff])";
/// for node in parse(&sgf).unwrap().iter() {
///     for prop in node.properties() {
///         println!("{:?}", prop);
///     }
/// }
/// ```
pub fn parse(text: &str) -> Result<Vec<SgfNode>, SgfParseError> {
    let mut collection: Vec<SgfNodeBuilder> = vec![];
    // Pointer to the `Vec` of children we're currently building.
    let mut current_node_list_ptr = NonNull::new(&mut collection).unwrap();
    // Stack of pointers to incomplete `Vec`s of children.
    let mut incomplete_child_lists: Vec<NonNull<Vec<SgfNodeBuilder>>> = vec![];
    // Using pointers involves some unsafe calls, but should be ok here.
    // Since pointers are always initialized from real structs, and thos structs
    // live for the whole function body, our only safety concern is dangling pointers.
    //
    // Since we build the tree traversing depth-first those structs shouldn't be
    // modified while the pointer is live. Heap-allocated contents of their
    // `children` may be modified, but that shouldn't change anything.

    let mut lexer = Lexer::new(text).peekable();
    while let Some(result) = lexer.next() {
        let (token, _span) = result?;
        match token {
            Token::StartGameTree => {
                // SGF game trees must have a root node.
                if let Some(node_list_ptr) = incomplete_child_lists.last() {
                    let node_list = unsafe { node_list_ptr.as_ref() };
                    if node_list.is_empty() {
                        return Err(SgfParseError::ParseError(
                            "Unexpected start of game tree".to_string(),
                        ));
                    }
                }
                incomplete_child_lists.push(current_node_list_ptr);
            }
            Token::EndGameTree => match incomplete_child_lists.pop() {
                Some(node_list) => current_node_list_ptr = node_list,
                None => {
                    return Err(SgfParseError::ParseError(
                        "Unexpected end of game tree".to_string(),
                    ))
                }
            },
            Token::StartNode => {
                let mut new_node = SgfNodeBuilder::new();
                while let Some(Ok((Token::Property(prop), _))) = lexer.peek() {
                    new_node.properties.push(prop.clone());
                    lexer.next();
                }
                let node_list = unsafe { current_node_list_ptr.as_mut() };
                node_list.push(new_node);
                current_node_list_ptr =
                    NonNull::new(&mut node_list.last_mut().unwrap().children).unwrap();
            }
            Token::Property(_) => {
                return Err(SgfParseError::ParseError("Unexpected property".to_string()))
            }
        }
    }
    if !incomplete_child_lists.is_empty() {
        return Err(SgfParseError::ParseError(
            "Unexpected end of data".to_string(),
        ));
    }

    collection
        .into_iter()
        .map(|mut node| {
            node.is_root = true;
            node.build()
        })
        .collect::<Result<_, _>>()
}

#[cfg(test)]
mod test {
    use super::super::props::*;
    use super::super::serialize;
    use super::{parse, SgfNode};

    fn load_test_sgf() -> Result<Vec<SgfNode>, Box<dyn std::error::Error>> {
        // See https://www.red-bean.com/sgf/examples/
        let mut sgf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        sgf_path.push("resources/test/ff4_ex.sgf");
        let data = std::fs::read_to_string(sgf_path)?;

        Ok(parse(&data)?)
    }

    fn node_depth(mut sgf_node: &SgfNode) -> u64 {
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
        let sgf_nodes = load_test_sgf().unwrap();
        assert_eq!(sgf_nodes[0].children().count(), 5);
    }

    #[test]
    fn gametree_one_has_size_19() {
        let sgf_nodes = load_test_sgf().unwrap();
        match sgf_nodes[0].get_property("SZ") {
            Some(SgfProp::SZ(size)) => assert_eq!(size, &(19, 19)),
            _ => unreachable!("Expected size property"),
        }
    }

    #[test]
    fn gametree_variation_depths() {
        let sgf_nodes = load_test_sgf().unwrap();
        let children: Vec<_> = sgf_nodes[0].children().collect();
        assert_eq!(node_depth(children[0]), 13);
        assert_eq!(node_depth(children[1]), 4);
        assert_eq!(node_depth(children[2]), 4);
    }

    #[test]
    fn gametree_two_has_one_variation() {
        let sgf_nodes = load_test_sgf().unwrap();
        assert_eq!(sgf_nodes[1].children().count(), 1);
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
        let result = parse(&input).unwrap();
        let expected = vec![
            SgfProp::GM(1),
            SgfProp::Invalid("W".to_string(), vec!["rp.pmonpoqprpsornqmpm".to_string()]),
        ];
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].properties().cloned().collect::<Vec<_>>(),
            expected
        );
    }

    #[test]
    fn stack_overflow() {
        // This input generated a stack overflow with the old code
        let input = "(;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;)";
        let result = parse(&input);
        assert!(result.is_ok());
    }
}
