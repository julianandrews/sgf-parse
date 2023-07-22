use crate::props::{PropertyType, SgfProp};

/// A node in an SGF Game Tree.
///
/// Any succesfully constructed node will be serializable, but may or may not be valid.
/// All game-specific information is encoded in the `Prop` type. Use
/// [`go::Prop`](`crate::go::Prop`) for go games, and
/// [`unknown_game::Prop`](`crate::unknown_game::Prop`) for all other games.
#[derive(Clone, Debug, PartialEq)]
pub struct SgfNode<Prop: SgfProp> {
    pub properties: Vec<Prop>,
    pub children: Vec<Self>,
    pub is_root: bool,
}

impl<Prop: SgfProp> Default for SgfNode<Prop> {
    fn default() -> Self {
        Self {
            properties: vec![],
            children: vec![],
            is_root: false,
        }
    }
}

impl<Prop: SgfProp> SgfNode<Prop> {
    /// Returns a new node.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{SgfNode, SgfProp};
    /// use sgf_parse::go::Prop;
    ///
    /// let children = vec![
    ///     SgfNode::<Prop>::new(
    ///         vec![Prop::new("B".to_string(), vec!["dd".to_string()])],
    ///         vec![],
    ///         false,
    ///     ),
    /// ];
    /// let node = SgfNode::new(vec![Prop::SZ((19, 19))], children, true);
    /// ```
    pub fn new(properties: Vec<Prop>, children: Vec<Self>, is_root: bool) -> Self {
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
    /// use sgf_parse::go::{parse, Prop};
    ///
    /// let node = parse("(;SZ[13:13];B[de])").unwrap().into_iter().next().unwrap();
    /// let board_size = match node.get_property("SZ") {
    ///     Some(Prop::SZ(size)) => size.clone(),
    ///     None => (19, 19),
    ///     _ => unreachable!(),
    /// };
    /// ```
    pub fn get_property(&self, identifier: &str) -> Option<&Prop> {
        self.properties
            .iter()
            .find(|&prop| prop.identifier() == identifier)
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
    pub fn children(&self) -> impl Iterator<Item = &Self> {
        self.children.iter()
    }

    /// Returns an iterator over the properties of this node.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::go::{parse, Move, Prop};
    ///
    /// let node = parse("(;B[de]C[A comment])").unwrap().into_iter().next().unwrap();
    /// for prop in node.properties() {
    ///     match prop {
    ///         Prop::B(mv) => match mv {
    ///             Move::Move(p) => println!("B Move at {}, {}", p.x, p.y),
    ///             Move::Pass => println!("B Pass"),
    ///         }
    ///         Prop::W(mv) => match mv {
    ///             Move::Move(p) => println!("W Move at {}, {}", p.x, p.y),
    ///             Move::Pass => println!("W Pass"),
    ///         }
    ///         _ => {},
    ///     }
    /// }
    /// ```
    pub fn properties(&self) -> impl Iterator<Item = &Prop> {
        self.properties.iter()
    }

    /// Returns the serialized SGF for this SgfNode as a complete GameTree.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::go::parse;
    ///
    /// let sgf = "(;SZ[13:13];B[de])";
    /// let node = parse(sgf).unwrap().into_iter().next().unwrap();
    /// assert_eq!(node.serialize(), sgf);
    /// ```
    pub fn serialize(&self) -> String {
        format!("({})", self)
    }

    /// Returns `Ok` if the node's properties are valid according to the SGF FF\[4\] spec.
    ///
    /// # Errors
    /// Returns an error if the node has invalid properties.
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
        // TODO: Implement this non-recursively
        self.validate_helper()?;
        Ok(())
    }

    // Helper that returns whether a child has any game info in its descendents.
    fn validate_helper(&self) -> Result<bool, InvalidNodeError> {
        Prop::validate_properties(&self.properties, self.is_root)?;
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

    /// Returns an iterator over the nodes of the main variation.
    ///
    /// This is a convenience method for iterating through the first child of each node until the
    /// main line ends.
    ///
    /// # Examples
    /// ```
    /// use crate::sgf_parse::SgfProp;
    /// use sgf_parse::go::{parse, Prop};
    ///
    /// let sgf = "(;B[ee];W[ce](;B[ge](;W[gd])(;W[gf]))(;B[ce]))";
    /// let node = &parse(sgf).unwrap()[0];
    ///
    /// let moves: Vec<Prop> = node
    ///     .main_variation()
    ///     .map(|n| {
    ///         n.get_property("B")
    ///             .or_else(|| n.get_property("W"))
    ///             .unwrap()
    ///             .clone()
    ///     })
    ///     .collect();
    /// let expected = vec![
    ///     Prop::new("B".to_string(), vec!["ee".to_string()]),
    ///     Prop::new("W".to_string(), vec!["ce".to_string()]),
    ///     Prop::new("B".to_string(), vec!["ge".to_string()]),
    ///     Prop::new("W".to_string(), vec!["gd".to_string()]),
    /// ];
    ///
    /// assert_eq!(moves, expected);
    /// ```
    pub fn main_variation(&self) -> impl Iterator<Item = &Self> {
        MainVariationIter {
            node: Some(self),
            started: false,
        }
    }

    /// Returns the move property (if present) on the node.
    ///
    /// # Examples
    /// ```
    /// use crate::sgf_parse::SgfProp;
    /// use sgf_parse::go::{parse, Prop};
    /// let sgf = "(;GM[1]B[tt]C[Comment])";
    /// let node = &parse(sgf).unwrap()[0];
    ///
    /// let mv = node.get_move();
    /// assert_eq!(mv, Some(&Prop::new("B".to_string(), vec!["tt".to_string()])));
    /// ```
    pub fn get_move(&self) -> Option<&Prop> {
        // Since there can only be one move per node in an sgf, this is safe.
        self.properties()
            .find(|p| p.property_type() == Some(PropertyType::Move))
    }

    fn has_game_info(&self) -> bool {
        for prop in self.properties() {
            if let Some(PropertyType::GameInfo) = prop.property_type() {
                return true;
            }
        }
        false
    }
}

impl<Prop: SgfProp> std::fmt::Display for SgfNode<Prop> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Implement this non-recursively
        let prop_string = self
            .properties()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("");
        let child_count = self.children().count();
        let child_string = match child_count {
            0 => "".to_string(),
            1 => self.children().next().unwrap().to_string(),
            _ => self
                .children()
                .map(|x| format!("({})", x))
                .collect::<Vec<_>>()
                .join(""),
        };
        write!(f, ";{}{}", prop_string, child_string)
    }
}

#[derive(Debug)]
struct MainVariationIter<'a, Prop: SgfProp> {
    node: Option<&'a SgfNode<Prop>>,
    started: bool,
}

impl<'a, Prop: SgfProp> Iterator for MainVariationIter<'a, Prop> {
    type Item = &'a SgfNode<Prop>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.started {
            self.node = self.node.and_then(|n| n.children().next());
        } else {
            self.started = true;
        }
        self.node
    }
}

/// Err type for [`SgfNode::validate`].
#[derive(Debug, Clone, PartialEq, Eq)]
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
