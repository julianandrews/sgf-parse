use super::props::SgfProp;

#[derive(Clone, Debug)]
pub struct SgfNode {
    properties: Vec<SgfProp>,
    children: Vec<SgfNode>,
}

impl SgfNode {
    pub fn new(props: Vec<SgfProp>, children: Vec<SgfNode>) -> SgfNode {
        SgfNode {
            properties: props,
            children: children,
        }
    }

    // TODO: Write a generalized "get_property" method to use instead of this.
    pub fn get_size(&self) -> Option<(u8, u8)> {
        self.properties.iter().filter_map(|p| match p {
            SgfProp::SZ(size) => Some(size.clone()),
            _ => None
        }).next()
    }

    pub fn children<'a>(&'a self) -> impl Iterator<Item=&SgfNode> + 'a {
        self.children.iter()
    }

    pub fn properties<'a>(&'a self) -> impl Iterator<Item=&SgfProp> + 'a {
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
