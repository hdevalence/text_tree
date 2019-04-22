use std::collections::HashSet;

pub struct Node {
    pub(super) children: Vec<Node>,
    pub(super) node_data: NodeData,
}

pub enum NodeData {
    Text(String),
    Element(ElementData),
}

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
}
