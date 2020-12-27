use std::collections::HashSet;
use std::fmt;
pub mod parse;

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub(super) node_data: NodeData,
    pub(super) children: Vec<Node>,
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

impl<T> From<T> for Node
where
    String: From<T>,
{
    fn from(s: T) -> Node {
        Node {
            children: Vec::new(),
            node_data: NodeData::Text(String::from(s)),
        }
    }
}

impl std::str::FromStr for Node {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use nom::Finish;
        match parse::document(s).finish() {
            Ok((remaining, node)) => {
                dbg!(remaining);
                Ok(node)
            }
            Err(e) => Err(nom::error::convert_error(s, e)),
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

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_at(node: &Node, f: &mut fmt::Formatter<'_>, i: usize) -> fmt::Result {
            let indent = std::iter::repeat("  ").take(i).collect::<String>(); // todo(eliza): ew lol
            match node.node_data {
                NodeData::Text(ref t) => writeln!(f, "{}{:?}", indent, t)?,
                NodeData::Element(ElementData {
                    ref id,
                    ref classes,
                }) => {
                    writeln!(f, "{}<tag id={:?} class={:?}>", indent, id, classes)?;

                    for child in &node.children {
                        fmt_at(child, f, i + 1)?;
                    }

                    writeln!(f, "{}</tag>", indent)?;
                }
            }
            Ok(())
        }
        fmt_at(self, f, 0)
    }
}
