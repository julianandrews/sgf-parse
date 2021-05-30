use std::collections::HashSet;

use crate::games::Game;
use crate::props::{PropertyType, SgfProp};

/// A node in an SGF Game Tree.
///
/// Any succesfully constructed node will be serializable, but may or may not be valid.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SgfNode<G: Game> {
    pub properties: Vec<SgfProp<G>>,
    pub children: Vec<SgfNode<G>>,
    pub is_root: bool,
}

impl<G: Game> SgfNode<G> {
    /// Returns a new `SgfNode`.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{serialize, SgfNode, SgfProp};
    /// use sgf_parse::go::Go;
    ///
    /// let children = vec![
    ///     SgfNode::<Go>::new(
    ///         vec![SgfProp::new("B".to_string(),
    ///         vec!["dd".to_string()])], vec![],
    ///         false,
    ///     ),
    /// ];
    ///
    /// let gametree = SgfNode::new(vec![SgfProp::SZ((19, 19))], children, true).into();
    /// assert_eq!(serialize(std::iter::once(&gametree)), "(;SZ[19:19];B[dd])");
    /// ```
    pub fn new(properties: Vec<SgfProp<G>>, children: Vec<Self>, is_root: bool) -> Self {
        Self {
            properties,
            children,
            is_root,
        }
    }

    /// Returns the property with the provided identifier for the node (if present).
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::SgfProp;
    /// use sgf_parse::go::parse;
    ///
    /// let node = parse("(;SZ[13:13];B[de])").unwrap().into_iter().next().unwrap();
    /// let board_size = match node.get_property("SZ") {
    ///     Some(SgfProp::SZ(size)) => size.clone(),
    ///     None => (19, 19),
    ///     _ => unreachable!(),
    /// };
    /// ```
    pub fn get_property(&self, identifier: &str) -> Option<&SgfProp<G>> {
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
    /// use sgf_parse::go::parse;
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
    /// use sgf_parse::SgfProp;
    /// use sgf_parse::go::{parse, Go, Move};
    ///
    /// let node = parse("(;B[de]C[A comment])").unwrap().into_iter().next().unwrap();
    /// for prop in node.properties() {
    ///     match prop {
    ///         SgfProp::<Go>::B(mv) => match mv {
    ///             Move::Move(p) => println!("B Move at {}, {}", p.x, p.y),
    ///             Move::Pass => println!("B Pass"),
    ///         }
    ///         SgfProp::<Go>::W(mv) => match mv {
    ///             Move::Move(p) => println!("W Move at {}, {}", p.x, p.y),
    ///             Move::Pass => println!("W Pass"),
    ///         }
    ///         _ => {},
    ///     }
    /// }
    /// ```
    pub fn properties<'a>(&'a self) -> impl Iterator<Item = &SgfProp<G>> + 'a {
        self.properties.iter()
    }

    /// Returns `Ok` if the [`SgfNode`]'s properties are valid according to the SGF FF\[4\] spec.
    ///
    /// # Errors
    /// If the node has invalid properties than an [`InvalidNodeError`] is returned with details
    /// about the invalid properties.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::InvalidNodeError;
    /// use sgf_parse::go::parse;
    ///
    /// let node = parse("(;B[de]C[A comment]C[Another])").unwrap().into_iter().next().unwrap();
    /// let result = node.validate();
    /// assert!(matches!(result, Err(InvalidNodeError::RepeatedIdentifier(_))));
    /// ```
    pub fn validate(&self) -> Result<(), InvalidNodeError> {
        // TODO: Rewrite this non-recursively
        self.validate_helper()?;
        Ok(())
    }

    // Helper that returns whether a child has any game info in its descendents.
    fn validate_helper(&self) -> Result<bool, InvalidNodeError> {
        self.validate_properties()?;
        let has_game_info = self.has_game_info();
        let mut child_has_game_info = false;
        for child in self.children() {
            child_has_game_info |= child.validate_helper()?;
        }
        if child_has_game_info && has_game_info {
            return Err(InvalidNodeError::UnexpectedGameInfo(format!(
                "{:?}",
                self.properties
            )));
        }
        Ok(has_game_info)
    }

    fn has_game_info(&self) -> bool {
        for prop in self.properties() {
            if let Some(PropertyType::GameInfo) = prop.property_type() {
                return true;
            }
        }
        false
    }

    fn validate_properties(&self) -> Result<(), InvalidNodeError> {
        let mut identifiers = HashSet::new();
        let mut markup_points = HashSet::new();
        let mut setup_node = false;
        let mut move_node = false;
        let mut move_seen = false;
        let mut exclusive_node_annotations = 0;
        let mut move_annotation_count = 0;
        for prop in self.properties() {
            match prop {
                SgfProp::B(_) => {
                    move_seen = true;
                    if identifiers.contains("W") {
                        return Err(InvalidNodeError::MultipleMoves(format!(
                            "{:?}",
                            self.properties.to_vec()
                        )));
                    }
                }
                SgfProp::W(_) => {
                    move_seen = true;
                    if identifiers.contains("B") {
                        return Err(InvalidNodeError::MultipleMoves(format!(
                            "{:?}",
                            self.properties.to_vec()
                        )));
                    }
                }
                SgfProp::CR(ps)
                | SgfProp::MA(ps)
                | SgfProp::SL(ps)
                | SgfProp::SQ(ps)
                | SgfProp::TR(ps) => {
                    for p in ps.iter() {
                        if markup_points.contains(&p) {
                            return Err(InvalidNodeError::RepeatedMarkup(format!(
                                "{:?}",
                                self.properties.to_vec()
                            )));
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
                SgfProp::Invalid(identifier, values) => {
                    return Err(InvalidNodeError::InvalidProperty(format!(
                        "{}, {:?}",
                        identifier, values
                    )))
                }
                _ => {}
            }
            match prop.property_type() {
                Some(PropertyType::Move) => move_node = true,
                Some(PropertyType::Setup) => setup_node = true,
                Some(PropertyType::Root) => {
                    if !self.is_root {
                        return Err(InvalidNodeError::UnexpectedRootProperties(format!(
                            "{:?}",
                            self.properties
                        )));
                    }
                }
                _ => {}
            }
            let ident = prop.identifier();
            if identifiers.contains(&ident) {
                return Err(InvalidNodeError::RepeatedIdentifier(format!(
                    "{:?}",
                    self.properties.to_vec()
                )));
            }
            identifiers.insert(prop.identifier());
        }
        if setup_node && move_node {
            return Err(InvalidNodeError::SetupAndMove(format!(
                "{:?}",
                self.properties.to_vec()
            )));
        }
        if identifiers.contains("KO") && !(identifiers.contains("B") || identifiers.contains("W")) {
            return Err(InvalidNodeError::KoWithoutMove(format!(
                "{:?}",
                self.properties.to_vec()
            )));
        }
        if move_annotation_count > 1 {
            return Err(InvalidNodeError::MultipleMoveAnnotations(format!(
                "{:?}",
                self.properties.to_vec()
            )));
        }
        if move_annotation_count == 1 && !move_seen {
            return Err(InvalidNodeError::UnexpectedMoveAnnotation(format!(
                "{:?}",
                self.properties.to_vec()
            )));
        }
        if exclusive_node_annotations > 1 {
            return Err(InvalidNodeError::MultipleExclusiveAnnotations(format!(
                "{:?}",
                self.properties.to_vec()
            )));
        }
        Ok(())
    }
}

/// Err type for [`SgfNode::validate`].
#[derive(Debug)]
pub enum InvalidNodeError {
    UnexpectedRootProperties(String),
    UnexpectedGameInfo(String),
    RepeatedMarkup(String),
    MultipleMoves(String),
    RepeatedIdentifier(String),
    SetupAndMove(String),
    KoWithoutMove(String),
    MultipleMoveAnnotations(String),
    UnexpectedMoveAnnotation(String),
    MultipleExclusiveAnnotations(String),
    InvalidProperty(String),
}

impl std::fmt::Display for InvalidNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidNodeError::UnexpectedRootProperties(context) => {
                write!(f, "Root properties in non-root node: {:?}", context)
            }
            InvalidNodeError::UnexpectedGameInfo(context) => {
                write!(f, "GameInfo properties in node and a child {:?}", context)
            }
            InvalidNodeError::RepeatedMarkup(context) => {
                write!(f, "Multiple markup properties on same point {:?}", context)
            }
            InvalidNodeError::MultipleMoves(context) => {
                write!(f, "B and W moves in same node {:?}", context)
            }
            InvalidNodeError::RepeatedIdentifier(context) => {
                write!(f, "Identifier repeated in node {:?}", context)
            }
            InvalidNodeError::SetupAndMove(context) => {
                write!(f, "Setup and move properties in same node {:?}", context)
            }
            InvalidNodeError::KoWithoutMove(context) => {
                write!(f, "Ko in node without B or W {:?}", context)
            }
            InvalidNodeError::MultipleMoveAnnotations(context) => {
                write!(f, "Multiple move annotations in same node {:?}", context)
            }
            InvalidNodeError::UnexpectedMoveAnnotation(context) => {
                write!(f, "Move annotation without move in node {:?}", context)
            }
            InvalidNodeError::MultipleExclusiveAnnotations(context) => {
                write!(
                    f,
                    "Multiple DM, UC, GW or GB properties in node {:?}",
                    context
                )
            }
            InvalidNodeError::InvalidProperty(context) => {
                write!(f, "Invalid property: {}", context)
            }
        }
    }
}

impl std::error::Error for InvalidNodeError {}

#[cfg(test)]
mod tests {
    use super::InvalidNodeError;
    use crate::go::parse;

    #[test]
    fn validate_sample_sgf_valid() {
        let mut sgf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        sgf_path.push("resources/test/ff4_ex.sgf");
        let sgf = std::fs::read_to_string(sgf_path).unwrap();
        let node = &parse(&sgf).unwrap()[0];
        assert!(node.validate().is_ok());
    }

    #[test]
    fn validate_valid_node() {
        let sgf = "(;SZ[9]HA[3]C[Some comment];B[de];W[fe])";
        let node = &parse(sgf).unwrap()[0];
        assert!(node.validate().is_ok());
    }

    #[test]
    fn validate_unexpected_root_properties() {
        let sgf = "(;SZ[9]C[Some comment];GM[1])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::UnexpectedRootProperties(_))
        ));
    }

    #[test]
    fn validate_unexpected_game_info() {
        let sgf = "(;SZ[9]KM[3.5]C[Some comment];HA[3])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::UnexpectedGameInfo(_))
        ));
    }

    #[test]
    fn validate_repeated_markup() {
        let sgf = "(;SZ[9]KM[3.5]C[Some comment];CR[dd]TR[dd])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::RepeatedMarkup(_))
        ));
    }

    #[test]
    fn validate_multiple_moves() {
        let sgf = "(;SZ[9]C[Some comment];B[dd]W[cd])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::MultipleMoves(_))
        ));
    }

    #[test]
    fn validate_repeated_identifier() {
        let sgf = "(;SZ[9]HA[3]HA[4])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::RepeatedIdentifier(_))
        ));
    }

    #[test]
    fn validate_setup_and_move() {
        let sgf = "(;AB[dd]B[cc])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::SetupAndMove(_))
        ));
    }

    #[test]
    fn validate_ko_without_move() {
        let sgf = "(;KO[])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::KoWithoutMove(_))
        ));
    }

    #[test]
    fn validate_multiple_move_annotations() {
        let sgf = "(;B[dd]DO[]BM[1])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::MultipleMoveAnnotations(_))
        ));
    }

    #[test]
    fn validate_unexpected_move_annotation() {
        let sgf = "(;BM[1])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::UnexpectedMoveAnnotation(_))
        ));
    }

    #[test]
    fn validate_multiple_exclusive_annotations() {
        let sgf = "(;UC[2]GW[2])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::MultipleExclusiveAnnotations(_))
        ));
    }

    #[test]
    fn validate_invalid_property() {
        let sgf = "(;BM[Invalid])";
        let node = &parse(sgf).unwrap()[0];
        assert!(matches!(
            node.validate(),
            Err(InvalidNodeError::InvalidProperty(_))
        ));
    }
}
