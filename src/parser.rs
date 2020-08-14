use super::errors::SgfParseError;
use super::props::SgfProp;
use super::sgf_node::SgfNode;

/// Returns a Vector of the root SgfNodes parsed from the provided text.
///
/// Any [SgfNode](struct.SgfNode.html) returned by this function should be valid according to the SGF
/// specification.
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
    let mut nodes: Vec<SgfNode> = vec![];
    let mut text = text.trim();
    while !text.is_empty() {
        let (node, new_text) = parse_game_tree(text, true)?;
        nodes.push(node);
        text = new_text.trim();
    }
    if nodes.is_empty() {
        Err(SgfParseError::ParseError(text.to_string()))?;
    }
    Ok(nodes)
}

fn parse_game_tree(mut text: &str, is_root: bool) -> Result<(SgfNode, &str), SgfParseError> {
    if text.chars().next() != Some('(') {
        Err(SgfParseError::ParseError(text.to_string()))?;
    }
    text = &text[1..].trim();
    let (node, new_text) = parse_node(text, is_root)?;
    text = &new_text.trim();
    if text.chars().next() != Some(')') {
        Err(SgfParseError::ParseError(text.to_string()))?;
    }

    Ok((node, &text[1..]))
}

fn parse_node(mut text: &str, is_root: bool) -> Result<(SgfNode, &str), SgfParseError> {
    if text.chars().next() != Some(';') {
        Err(SgfParseError::ParseError(text.to_string()))?;
    }
    text = &text[1..].trim();

    let mut props: Vec<SgfProp> = vec![];
    while let Some(c) = text.chars().next() {
        if !c.is_ascii_uppercase() {
            break;
        }
        let (prop, new_text) =
            parse_property(text).map_err(|_| SgfParseError::ParseError(text.to_string()))?;
        text = new_text;
        props.push(prop);
    }

    text = &text.trim();
    let mut children: Vec<SgfNode> = vec![];
    while text.chars().next() == Some('(') {
        let (node, new_text) = parse_game_tree(text, false)?;
        text = &new_text.trim();
        children.push(node);
    }
    if text.chars().next() == Some(';') {
        let (node, new_text) = parse_node(text, false)?;
        text = &new_text;
        children.push(node);
    }

    let node = SgfNode::new(props, children, is_root)
        .map_err(|_| SgfParseError::ParseError(text.to_string()))?;
    Ok((node, text))
}

fn parse_property(mut text: &str) -> Result<(SgfProp, &str), SgfParseError> {
    let (prop_ident, new_text) = parse_prop_ident(text)?;
    text = &new_text;
    let (prop_values, new_text) = parse_prop_values(text)?;
    text = &new_text;

    Ok((SgfProp::new(prop_ident, prop_values)?, text))
}

fn parse_prop_ident(mut text: &str) -> Result<(String, &str), SgfParseError> {
    let mut prop_ident = vec![];
    loop {
        match text.chars().next() {
            Some('[') => break,
            Some(c) if c.is_ascii_uppercase() => {
                prop_ident.push(c);
                text = &text[1..];
            }
            _ => Err(SgfParseError::ParseError(text.to_string()))?,
        }
    }

    Ok((prop_ident.iter().collect(), text))
}

fn parse_prop_values(text: &str) -> Result<(Vec<String>, &str), SgfParseError> {
    let mut prop_values = vec![];
    let mut text = text;
    loop {
        let mut chars = text.chars();
        match chars.next() {
            Some('[') => {
                let (value, new_text) = parse_value(chars.as_str())?;
                text = new_text;
                prop_values.push(value);
            }
            Some(c) if c.is_whitespace() => text = chars.as_str(),
            _ => break,
        }
    }

    Ok((prop_values, text))
}

fn parse_value(text: &str) -> Result<(String, &str), SgfParseError> {
    let mut prop_value = vec![];
    let mut chars = text.chars();
    let mut escaped = false;
    loop {
        match chars.next() {
            Some(']') if !escaped => break,
            Some('\\') if !escaped => escaped = true,
            Some(c) => {
                escaped = false;
                prop_value.push(c);
            }
            None => Err(SgfParseError::ParseError(text.to_string()))?,
        }
    }

    Ok((prop_value.iter().collect(), chars.as_str()))
}

#[cfg(test)]
mod test {
    use super::*;

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
    pub fn test_sgf_has_two_gametrees() {
        let sgf_nodes = load_test_sgf().unwrap();
        assert_eq!(sgf_nodes.len(), 2);
    }

    #[test]
    pub fn test_gametree_one_has_five_variations() {
        let sgf_nodes = load_test_sgf().unwrap();
        assert_eq!(sgf_nodes[0].children().count(), 5);
    }

    #[test]
    pub fn test_gametree_one_has_size_19() {
        let sgf_nodes = load_test_sgf().unwrap();
        match sgf_nodes[0].get_property("SZ") {
            Some(SgfProp::SZ(size)) => assert_eq!(size, &(19, 19)),
            _ => assert!(false, "Expected size property"),
        }
    }

    #[test]
    pub fn test_gametree_variation_depths() {
        let sgf_nodes = load_test_sgf().unwrap();
        let children: Vec<_> = sgf_nodes[0].children().collect();
        assert_eq!(node_depth(children[0]), 13);
        assert_eq!(node_depth(children[1]), 4);
        assert_eq!(node_depth(children[2]), 4);
    }

    #[test]
    pub fn test_gametree_two_has_one_variation() {
        let sgf_nodes = load_test_sgf().unwrap();
        assert_eq!(sgf_nodes[1].children().count(), 1);
    }
}
