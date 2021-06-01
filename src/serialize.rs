use crate::GameTree;

/// Returns the serialized SGF text from a collection of [`GameTree`] objects.
///
/// # Examples
/// ```
/// use sgf_parse::{serialize, SgfNode, SgfProp};
/// use sgf_parse::go::Prop;
///
/// let first_node: SgfNode::<Prop> = {
///     let children = vec![
///         SgfNode::new(
///             vec![Prop::new("B".to_string(),
///             vec!["dd".to_string()])], vec![],
///             false,
///         ),
///     ];
///     SgfNode::new(vec![Prop::SZ((19, 19))], children, true)
/// };
/// let second_node = SgfNode::<Prop>::new(vec![Prop::C("A comment".into())], vec![], true);
/// let gametrees = vec![first_node.into(), second_node.into()];
/// let serialized = serialize(&gametrees);
///
/// assert_eq!(serialized, "(;SZ[19:19];B[dd])(;C[A comment])");
/// ```
pub fn serialize<'a>(gametrees: impl IntoIterator<Item = &'a GameTree>) -> String {
    let gametrees_text = gametrees
        .into_iter()
        .map(|gametree| gametree.to_string())
        .collect::<Vec<String>>()
        .join(")(");
    format!("({})", gametrees_text)
}

#[cfg(test)]
mod test {
    use super::serialize;
    use crate::parse;

    #[test]
    fn simple_sgf() {
        let sgf = "(;C[Some comment];B[de]FOO[bar][baz];W[fe])(;B[de];W[ff])";
        let game_trees = parse(sgf).unwrap();
        let result = serialize(&game_trees);
        assert_eq!(result, sgf);
    }
}
