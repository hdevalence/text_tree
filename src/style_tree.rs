use std::collections::HashMap;

use super::content_tree::*;
use super::style::*;

type PropertyMap = HashMap<String, Value>;

#[derive(Debug)]
pub struct StyledNode<'a> {
    pub(super) node: &'a Node,
    pub(super) specified_values: PropertyMap,
    pub(super) children: Vec<StyledNode<'a>>,
}

fn specified_values(element: &ElementData, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = PropertyMap::new();
    let mut rules = stylesheet.matching_rules(element);

    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in &rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    values
}

pub fn style_tree<'a>(root_node: &'a Node, style: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root_node,
        specified_values: match &root_node.node_data {
            NodeData::Element(element) => specified_values(&element, style),
            NodeData::Text(_) => PropertyMap::new(),
        },
        children: root_node
            .children
            .iter()
            .map(|c| style_tree(c, style))
            .collect(),
    }
}

impl<'a> StyledNode<'a> {
    pub fn node(&self) -> &'a Node {
        self.node
    }

    #[tracing::instrument]
    pub fn display(&self) -> DisplayKind {
        tracing::info!(display = ?self.value("display"));
        match self.value("display") {
            Some(Value::Display(d)) => d,
            _ => DisplayKind::Inline,
        }
    }

    pub fn value(&self, keyword: &str) -> Option<Value> {
        self.specified_values.get(keyword).map(|v| v.clone())
    }

    pub fn lookup(&self, keyword: &str, shorthand: &str, default: &Value) -> Value {
        self.specified_values
            .get(keyword)
            .or_else(|| self.specified_values.get(shorthand))
            .map(|v| v.clone())
            .unwrap_or(default.clone())
    }
}
