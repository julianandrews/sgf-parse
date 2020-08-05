use std::collections::HashSet;

use super::{PropertyType, SgfParseError, SgfProp};

/// A node in an SGF Game Tree.
#[derive(Clone, Debug)]
pub struct SgfNode {
    properties: Vec<SgfProp>,
    children: Vec<SgfNode>,
}

impl SgfNode {
    pub fn new(props: Vec<SgfProp>, children: Vec<SgfNode>) -> Result<SgfNode, SgfParseError> {
        validate_node_props(&props)?;
        Ok(SgfNode {
            properties: props,
            children: children,
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
        for prop in self.properties.iter() {
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
    pub fn children<'a>(&'a self) -> impl Iterator<Item = &SgfNode> + 'a {
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
}

impl IntoIterator for SgfNode {
    type Item = SgfNode;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

fn validate_node_props(props: &Vec<SgfProp>) -> Result<(), SgfParseError> {
    let mut identifiers = HashSet::new();
    let mut setup_node = false;
    let mut move_node = false;
    let mut exclusive_node_annotations = 0;
    for prop in props.iter() {
        match prop {
            SgfProp::B(_) => {
                if identifiers.contains("W") {
                    Err(SgfParseError::InvalidNodeProps(props.clone()))?;
                }
            }
            SgfProp::W(_) => {
                if identifiers.contains("B") {
                    Err(SgfParseError::InvalidNodeProps(props.clone()))?;
                }
            }
            SgfProp::DM(_) => exclusive_node_annotations += 1,
            SgfProp::UC(_) => exclusive_node_annotations += 1,
            SgfProp::GW(_) => exclusive_node_annotations += 1,
            SgfProp::GB(_) => exclusive_node_annotations += 1,
            _ => {}
        }
        match prop.property_type() {
            Some(PropertyType::Move) => move_node = true,
            Some(PropertyType::Setup) => setup_node = true,
            _ => {}
        }
        let ident = prop.identifier();
        if identifiers.contains(&ident) {
            Err(SgfParseError::InvalidNodeProps(props.clone()))?;
        }
        identifiers.insert(prop.identifier());
    }
    if setup_node && move_node {
        Err(SgfParseError::InvalidNodeProps(props.clone()))?;
    }
    if identifiers.contains("KO") && !(identifiers.contains("B") || identifiers.contains("W")) {
        Err(SgfParseError::InvalidNodeProps(props.clone()))?;
    }
    if exclusive_node_annotations > 1 {
        Err(SgfParseError::InvalidNodeProps(props.clone()))?;
    }
    // TODO: Validate no move annotations without move.
    // TODO: Validate no more than one markup property per point.
    // TODO: Validate root properties only on root nodes
    // TODO: Validate game-info properties only once per path.
    Ok(())
}
