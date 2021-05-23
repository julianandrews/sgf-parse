use std::collections::HashSet;

use super::props::{PropertyType, SgfProp};

/// A node in an SGF Game Tree.
///
/// By design `SgfNode` is immutable and can any succesfully constructed `SgfNode` should be valid
/// and serializable.
///
/// If you want to edit an `SgfNode` convert it into an `SgfNodeBuilder` using `SgfNode::into_builder`.
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
    /// use sgf_parse::{serialize, SgfNode, SgfProp};
    ///
    /// let children = vec![
    ///     SgfNode::new(
    ///         vec![SgfProp::new("B".to_string(), vec!["dd".to_string()])],
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
    ) -> Result<Self, InvalidNodeError> {
        let (has_root_props, has_game_info_props) = validate_node_props(&properties)?;
        if has_root_props && !is_root {
            return Err(InvalidNodeError::UnexpectedRootNode(properties));
        }
        let children_have_game_info = children.iter().any(|child| child.has_game_info);
        if has_game_info_props && children_have_game_info {
            return Err(InvalidNodeError::UnexpectedGameInfo(properties));
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
    /// use sgf_parse::{parse, Move, SgfProp};
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

    /// Returns an editable [SgfNodeBuilder](struct.SgfNodeBuilder.html) for the node, consuming it in the process.
    ///
    /// # Examples
    /// ```
    /// use sgf_parse::{parse, serialize, SgfProp};
    ///
    /// let node = parse("(;SZ[13:13];B[de])").unwrap().into_iter().next().unwrap();
    /// let new_prop = SgfProp::new("C".to_string(), vec!["New comment".to_string()]);
    /// let mut builder = node.into_builder();
    /// builder.properties.push(new_prop);
    /// let new_node = builder.build();
    /// let sgf = serialize(&new_node);
    ///
    /// assert_eq!(sgf, "(;SZ[13:13]C[New comment];B[de])");
    /// ```
    pub fn into_builder(self) -> SgfNodeBuilder {
        let Self {
            properties,
            children,
            is_root,
            has_game_info: _,
        } = self;
        let children = children
            .into_iter()
            .map(Self::into_builder)
            .collect::<Vec<_>>();

        SgfNodeBuilder {
            properties,
            children,
            is_root,
        }
    }
}

fn validate_node_props(props: &[SgfProp]) -> Result<(bool, bool), InvalidNodeError> {
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
                    return Err(InvalidNodeError::MultipleMoves(props.to_vec()));
                }
            }
            SgfProp::W(_) => {
                move_seen = true;
                if identifiers.contains("B") {
                    return Err(InvalidNodeError::MultipleMoves(props.to_vec()));
                }
            }
            SgfProp::CR(ps)
            | SgfProp::MA(ps)
            | SgfProp::SL(ps)
            | SgfProp::SQ(ps)
            | SgfProp::TR(ps) => {
                for p in ps.iter() {
                    if markup_points.contains(&p) {
                        return Err(InvalidNodeError::RepeatedMarkup(props.to_vec()));
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
            return Err(InvalidNodeError::RepeatedIdentifier(props.to_vec()));
        }
        identifiers.insert(prop.identifier());
    }
    if setup_node && move_node {
        return Err(InvalidNodeError::SetupAndMove(props.to_vec()));
    }
    if identifiers.contains("KO") && !(identifiers.contains("B") || identifiers.contains("W")) {
        return Err(InvalidNodeError::KoWithoutMove(props.to_vec()));
    }
    if move_annotation_count > 1 {
        return Err(InvalidNodeError::MultipleMoveAnnotations(props.to_vec()));
    }
    if move_annotation_count == 1 && !move_seen {
        return Err(InvalidNodeError::UnexpectedMoveAnnotation(props.to_vec()));
    }
    if exclusive_node_annotations > 1 {
        return Err(InvalidNodeError::MultipleExclusiveAnnotations(
            props.to_vec(),
        ));
    }
    Ok((root_node, game_info_node))
}

/// A builder for [SgfNode](struct.SgfNode.html) structs.
///
/// `SgfNode`s are immutable and required to be valid from the time of creation. An `SgfNodeBuilder`
/// can be used to construct a complicated game tree which can then be converted to an `SgfNode`
/// with little overhead. If you're building an SGF file from scratch, this should be your starting
/// point. If you want to modify an existing SGF, `SgfNode::into_builder` will get you an
/// `SgfNodeBuilder` to work with.
///
/// Note that `SgfNodeBuilder` performs no validation until you call the `build` method. The user
/// is responsible for ensuring that no invalid combination of properties has been set.
///
/// # Examples
/// ```
/// use sgf_parse::{serialize, SgfNodeBuilder, SgfProp};
///
/// let mut node = SgfNodeBuilder::new();
/// node.properties.push(SgfProp::new("B".to_string(), vec!["jj".to_string()]));
/// let mut child = SgfNodeBuilder::new();
/// child.properties.push(SgfProp::new("W".to_string(), vec!["cd".to_string()]));
/// node.children.push(child);
///
/// let node = node.build();
/// let sgf = serialize(&node);
///
/// assert_eq!(sgf, "(;B[jj];W[cd])");
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SgfNodeBuilder {
    pub properties: Vec<SgfProp>,
    pub children: Vec<SgfNodeBuilder>,
    pub is_root: bool,
}

impl SgfNodeBuilder {
    /// Return a new empty `SgfNodeBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Consume the `SgfNodeBuilder` and its children and return an `SgfNode`.
    ///
    /// # Errors
    /// If the `SgfNode` or any of its children are invalid, then an error is returned.
    pub fn build(self) -> Result<SgfNode, InvalidNodeError> {
        // The obvious simple recursive approach can lead to a stack overflow, so we'll need to get
        // a little fancy.
        use core::cell::RefCell;
        use std::rc::Rc;

        let mut node_parts = vec![];

        // First traverse the tree associating the information needed to build an `SgfNode`
        // with a pointer to its parent's vector of children.
        let mut dfs_stack = vec![(self, None)];
        while let Some((node, parent_children)) = dfs_stack.pop() {
            let Self {
                properties,
                children,
                is_root,
            } = node;
            let built_children: Rc<RefCell<Vec<SgfNode>>> = Rc::new(RefCell::new(vec![]));
            for child in children {
                dfs_stack.push((child, Some(built_children.clone())));
            }
            node_parts.push((properties, built_children, is_root, parent_children));
        }

        // Now walk through the tree backwards building nodes and pushing them onto their parents'
        // child node vectors.
        for (properties, children, is_root, parent_children) in node_parts.into_iter().rev() {
            let children = Rc::try_unwrap(children)
                .expect("All children should already be built")
                .into_inner();
            let new_node = SgfNode::new(properties, children, is_root)?;
            if let Some(parent_children) = parent_children {
                parent_children.borrow_mut().push(new_node);
            } else {
                return Ok(new_node);
            }
        }
        unreachable!("The first node must have no parent")
    }
}

/// Error type for invalid [SgfNode](struct.SgfNode.html) structs.
#[derive(Debug)]
pub enum InvalidNodeError {
    UnexpectedRootNode(Vec<SgfProp>),
    UnexpectedGameInfo(Vec<SgfProp>),
    RepeatedMarkup(Vec<SgfProp>),
    MultipleMoves(Vec<SgfProp>),
    RepeatedIdentifier(Vec<SgfProp>),
    SetupAndMove(Vec<SgfProp>),
    KoWithoutMove(Vec<SgfProp>),
    MultipleMoveAnnotations(Vec<SgfProp>),
    UnexpectedMoveAnnotation(Vec<SgfProp>),
    MultipleExclusiveAnnotations(Vec<SgfProp>),
}

impl std::fmt::Display for InvalidNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidNodeError::UnexpectedRootNode(props) => {
                write!(f, "Root properties in non-root node: {:?}", props)
            }
            InvalidNodeError::UnexpectedGameInfo(props) => {
                write!(f, "GameInfo properties in node and a child {:?}", props)
            }
            InvalidNodeError::RepeatedMarkup(props) => {
                write!(f, "Multiple markup properties on same point {:?}", props)
            }
            InvalidNodeError::MultipleMoves(props) => {
                write!(f, "B and W moves in same node {:?}", props)
            }
            InvalidNodeError::RepeatedIdentifier(props) => {
                write!(f, "Identifier repeated in node {:?}", props)
            }
            InvalidNodeError::SetupAndMove(props) => {
                write!(f, "Setup and move properties in same node {:?}", props)
            }
            InvalidNodeError::KoWithoutMove(props) => {
                write!(f, "Ko in node without B or W {:?}", props)
            }
            InvalidNodeError::MultipleMoveAnnotations(props) => {
                write!(f, "Multiple move annotations in same node {:?}", props)
            }
            InvalidNodeError::UnexpectedMoveAnnotation(props) => {
                write!(f, "Move annotation without move in node {:?}", props)
            }
            InvalidNodeError::MultipleExclusiveAnnotations(props) => {
                write!(
                    f,
                    "Multiple DM, UC, GW or GB properties in node {:?}",
                    props
                )
            }
        }
    }
}

impl std::error::Error for InvalidNodeError {}
