use std::collections::HashSet;

use super::{PropertyType, SgfParseError, SgfProp};

/// A node in an SGF Game Tree.
///
/// By design `SgfNode` is immutable and can any succesfully constructed `SgfNode` should be valid
/// and serializable.
///
/// If you want to 'edit' an `SgfNode` without copying the properties and children, you must
/// destructure it, make your changes, and then build a new `SgfNode` from the parts. See the
/// the [destructure](struct.SgfNode.html#method.destructure) method.
#[derive(Clone, Debug, PartialEq)]
pub struct SgfNode {
    properties: Vec<SgfProp>,
    children: Vec<SgfNode>,
    is_root: bool,
    has_game_info: bool,
}

impl SgfNode {
    /// Returns a new `SgfNode`.
    ///
    /// # Errors
    /// If the provided children and properties don't correspond to a valid SGF node, then an error
    /// is returned.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{serialize, SgfNode, SgfProp, Move, Point};
    ///
    /// let children = vec![
    ///     SgfNode::new(
    ///         vec![SgfProp::B(Move::Move(Point { x: 3, y: 3 }))],
    ///         vec![],
    ///         false,
    ///     ).unwrap()
    /// ];
    ///
    /// let node = SgfNode::new(vec![SgfProp::SZ((19, 19))], children, true).unwrap();
    /// assert_eq!(serialize(std::iter::once(&node)), "(;SZ[19:19];B[dd])");
    /// ```
    pub fn new(
        properties: Vec<SgfProp>,
        children: Vec<Self>,
        is_root: bool,
    ) -> Result<Self, SgfParseError> {
        let (has_root_props, has_game_info_props) = validate_node_props(&properties)?;
        if has_root_props && !is_root {
            return Err(SgfParseError::InvalidNode(
                "Root properties in non-root node".to_string(),
            ));
        }
        let children_have_game_info = children.iter().any(|child| child.has_game_info);
        if has_game_info_props && children_have_game_info {
            return Err(SgfParseError::InvalidNode(
                "Multiple GameInfo nodes in path.".to_string(),
            ));
        }
        Ok(Self {
            properties,
            children,
            is_root,
            has_game_info: has_game_info_props || children_have_game_info,
        })
    }

    /// Returns the property with the provided identifier for the node (if present).
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{parse, SgfProp};
    ///
    /// let node = parse("(;SZ[13:13];B[de])").unwrap().into_iter().next().unwrap();
    /// let board_size = match node.get_property("SZ") {
    ///     Some(SgfProp::SZ(size)) => size.clone(),
    ///     None => (19, 19),
    ///     _ => unreachable!(),
    /// };
    /// ```
    pub fn get_property(&self, identifier: &str) -> Option<&SgfProp> {
        for prop in &self.properties {
            if prop.identifier() == identifier {
                return Some(prop);
            }
        }

        None
    }

    /// Returns an iterator over the children of this node.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::parse;
    ///
    /// let node = parse("(;SZ[19](;B[de])(;B[dd]HO[2]))").unwrap().into_iter().next().unwrap();
    /// for child in node.children() {
    ///     if let Some(prop) = child.get_property("HO") {
    ///        println!("Found a hotspot!")
    ///     }
    /// }
    /// ```
    pub fn children<'a>(&'a self) -> impl Iterator<Item = &Self> + 'a {
        self.children.iter()
    }

    /// Returns an iterator over the properties of this node.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{parse, SgfProp, Move};
    ///
    /// let node = parse("(;B[de]C[A comment])").unwrap().into_iter().next().unwrap();
    /// for prop in node.properties() {
    ///     match prop {
    ///         SgfProp::B(mv) => match mv {
    ///             Move::Move(p) => println!("B Move at {}, {}", p.x, p.y),
    ///             Move::Pass => println!("B Pass"),
    ///         }
    ///         SgfProp::W(mv) => match mv {
    ///             Move::Move(p) => println!("W Move at {}, {}", p.x, p.y),
    ///             Move::Pass => println!("W Pass"),
    ///         }
    ///         _ => {},
    ///     }
    /// }
    /// ```
    pub fn properties<'a>(&'a self) -> impl Iterator<Item = &SgfProp> + 'a {
        self.properties.iter()
    }

    /// Returns the fields of the node, consuming it in the process.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{parse, serialize, SgfNode, SgfProp, Text};
    ///
    /// let node = parse("(;SZ[13:13];B[de])").unwrap().into_iter().next().unwrap();
    /// let (mut properties, children, is_root) = node.destructure();
    /// properties.push(SgfProp::C(Text { text: "New comment".to_string() }));
    /// let node = SgfNode::new(properties, children, true);
    /// assert_eq!(serialize(&node), "(;SZ[13:13]C[New comment];B[de])");
    /// ```
    pub fn destructure(self) -> (Vec<SgfProp>, Vec<SgfNode>, bool) {
        match self {
            SgfNode { properties, children, is_root, has_game_info: _ } => (properties, children, is_root)
        }
    }
}

impl IntoIterator for SgfNode {
    type Item = Self;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

fn validate_node_props(props: &[SgfProp]) -> Result<(bool, bool), SgfParseError> {
    let mut identifiers = HashSet::new();
    let mut markup_points = HashSet::new();
    let mut setup_node = false;
    let mut move_node = false;
    let mut move_seen = false;
    let mut game_info_node = false;
    let mut root_node = false;
    let mut exclusive_node_annotations = 0;
    let mut move_annotation_count = 0;
    for prop in props.iter() {
        match prop {
            SgfProp::B(_) => {
                move_seen = true;
                if identifiers.contains("W") {
                    return Err(SgfParseError::InvalidNodeProps(props.to_owned()));
                }
            }
            SgfProp::W(_) => {
                move_seen = true;
                if identifiers.contains("B") {
                    return Err(SgfParseError::InvalidNodeProps(props.to_owned()));
                }
            }
            SgfProp::CR(ps)
            | SgfProp::MA(ps)
            | SgfProp::SL(ps)
            | SgfProp::SQ(ps)
            | SgfProp::TR(ps) => {
                for p in ps.iter() {
                    if markup_points.contains(&p) {
                        return Err(SgfParseError::InvalidNodeProps(props.to_owned()));
                    }
                    markup_points.insert(p);
                }
            }
            SgfProp::DM(_) | SgfProp::UC(_) | SgfProp::GW(_) | SgfProp::GB(_) => {
                exclusive_node_annotations += 1
            }
            SgfProp::BM(_) | SgfProp::DO | SgfProp::IT | SgfProp::TE(_) => {
                move_annotation_count += 1
            }
            _ => {}
        }
        match prop.property_type() {
            Some(PropertyType::Move) => move_node = true,
            Some(PropertyType::Setup) => setup_node = true,
            Some(PropertyType::GameInfo) => game_info_node = true,
            Some(PropertyType::Root) => root_node = true,
            _ => {}
        }
        let ident = prop.identifier();
        if identifiers.contains(&ident) {
            return Err(SgfParseError::InvalidNodeProps(props.to_owned()));
        }
        identifiers.insert(prop.identifier());
    }
    if setup_node && move_node {
        return Err(SgfParseError::InvalidNodeProps(props.to_owned()));
    }
    if identifiers.contains("KO") && !(identifiers.contains("B") || identifiers.contains("W")) {
        return Err(SgfParseError::InvalidNodeProps(props.to_owned()));
    }
    if move_annotation_count > 1 || (move_annotation_count == 1 && !move_seen) {
        return Err(SgfParseError::InvalidNodeProps(props.to_owned()));
    }
    if exclusive_node_annotations > 1 {
        return Err(SgfParseError::InvalidNodeProps(props.to_owned()));
    }
    Ok((root_node, game_info_node))
}
