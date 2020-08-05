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
