use std::collections::HashSet;
pub mod parse;

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub(super) children: Vec<Node>,
    pub(super) node_data: NodeData,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeData {
    Text(String),
    Element(ElementData),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ElementData {
    //pub(super) node_type: Option<String>,
    pub(super) id: Option<String>,
    pub(super) classes: HashSet<String>,
}

impl From<String> for Node {
    fn from(s: String) -> Node {
        Node {
            children: Vec::new(),
            node_data: NodeData::Text(s),
        }
    }
}

impl Node {
    pub fn new(children: Vec<Node>, id: Option<String>, classes: HashSet<String>) -> Node {
        Node {
            children,
            node_data: NodeData::Element(ElementData { id, classes }),
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self.node_data {
            NodeData::Text(ref s) => Some(s),
            NodeData::Element(_) => None,
        }
    }
}
