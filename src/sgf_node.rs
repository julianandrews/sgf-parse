use std::collections::HashSet;

use super::props::{PropertyType, SgfProp};
use super::traits::Game;

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
    /// use sgf_parse::game::{GameTree, GoGame};
    ///
    /// let children = vec![
    ///     SgfNode::<GoGame>::new(
    ///         vec![SgfProp::new("B".to_string(),
    ///         vec!["dd".to_string()])], vec![],
    ///         false,
    ///     ),
    /// ];
    ///
    /// let node = SgfNode::new(vec![SgfProp::SZ((19, 19))], children, true);
    /// let gametree = GameTree::GoGame(node);
    /// assert_eq!(serialize(std::iter::once(&gametree)), "(;SZ[19:19];B[dd])");
    /// ```
    pub fn new(properties: Vec<SgfProp<G>>, children: Vec<Self>, is_root: bool) -> Self {
        Self {
            properties,
            children,
            is_root,
        }
    }

    /// TODO: docstring examples and tests
    pub fn validate(&self) -> Result<(), InvalidNodeError> {
        // TODO: move validate_node_props into impl.
        let (has_root_props, has_game_info_props) = validate_node_props(&self.properties)?;
        if has_root_props && !self.is_root {
            return Err(InvalidNodeError::UnexpectedRootNode(format!(
                "{:?}",
                self.properties
            )));
        }
        // TODO: validate game_info.
        // let children_have_game_info = self.children.iter().any(|child| child.has_game_info);
        // if has_game_info_props && children_have_game_info {
        //     return Err(InvalidNodeError::UnexpectedGameInfo(format!(
        //         "{:?}",
        //         self.properties
        //     )));
        // }
        // TODO:
        Ok(())
    }

    /// Returns the property with the provided identifier for the node (if present).
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{parse_go, SgfProp};
    ///
    /// let node = parse_go("(;SZ[13:13];B[de])").unwrap().into_iter().next().unwrap();
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
    /// use sgf_parse::parse_go;
    ///
    /// let node = parse_go("(;SZ[19](;B[de])(;B[dd]HO[2]))").unwrap().into_iter().next().unwrap();
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
    /// use sgf_parse::{parse_go, SgfProp};
    /// use sgf_parse::game::{GoGame, GoMove};
    ///
    /// let node = parse_go("(;B[de]C[A comment])").unwrap().into_iter().next().unwrap();
    /// for prop in node.properties() {
    ///     match prop {
    ///         SgfProp::<GoGame>::B(mv) => match mv {
    ///             GoMove::Move(p) => println!("B Move at {}, {}", p.x, p.y),
    ///             GoMove::Pass => println!("B Pass"),
    ///         }
    ///         SgfProp::<GoGame>::W(mv) => match mv {
    ///             GoMove::Move(p) => println!("W Move at {}, {}", p.x, p.y),
    ///             GoMove::Pass => println!("W Pass"),
    ///         }
    ///         _ => {},
    ///     }
    /// }
    /// ```
    pub fn properties<'a>(&'a self) -> impl Iterator<Item = &SgfProp<G>> + 'a {
        self.properties.iter()
    }
}

fn validate_node_props<G: Game>(props: &[SgfProp<G>]) -> Result<(bool, bool), InvalidNodeError> {
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
                    return Err(InvalidNodeError::MultipleMoves(format!(
                        "{:?}",
                        props.to_vec()
                    )));
                }
            }
            SgfProp::W(_) => {
                move_seen = true;
                if identifiers.contains("B") {
                    return Err(InvalidNodeError::MultipleMoves(format!(
                        "{:?}",
                        props.to_vec()
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
                            props.to_vec()
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
            return Err(InvalidNodeError::RepeatedIdentifier(format!(
                "{:?}",
                props.to_vec()
            )));
        }
        identifiers.insert(prop.identifier());
    }
    if setup_node && move_node {
        return Err(InvalidNodeError::SetupAndMove(format!(
            "{:?}",
            props.to_vec()
        )));
    }
    if identifiers.contains("KO") && !(identifiers.contains("B") || identifiers.contains("W")) {
        return Err(InvalidNodeError::KoWithoutMove(format!(
            "{:?}",
            props.to_vec()
        )));
    }
    if move_annotation_count > 1 {
        return Err(InvalidNodeError::MultipleMoveAnnotations(format!(
            "{:?}",
            props.to_vec()
        )));
    }
    if move_annotation_count == 1 && !move_seen {
        return Err(InvalidNodeError::UnexpectedMoveAnnotation(format!(
            "{:?}",
            props.to_vec()
        )));
    }
    if exclusive_node_annotations > 1 {
        return Err(InvalidNodeError::MultipleExclusiveAnnotations(format!(
            "{:?}",
            props.to_vec()
        )));
    }
    Ok((root_node, game_info_node))
}

/// Error type for invalid [SgfNode](struct.SgfNode.html) structs.
#[derive(Debug)]
pub enum InvalidNodeError {
    UnexpectedRootNode(String),
    UnexpectedGameInfo(String),
    RepeatedMarkup(String),
    MultipleMoves(String),
    RepeatedIdentifier(String),
    SetupAndMove(String),
    KoWithoutMove(String),
    MultipleMoveAnnotations(String),
    UnexpectedMoveAnnotation(String),
    MultipleExclusiveAnnotations(String),
}

impl std::fmt::Display for InvalidNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidNodeError::UnexpectedRootNode(context) => {
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
        }
    }
}

impl std::error::Error for InvalidNodeError {}
