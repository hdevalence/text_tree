pub mod content_tree;
pub mod style;
pub mod style_tree;
pub mod layout;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::content_tree::*;
    use super::style::*;
    use super::style_tree::*;

    #[test]
    fn build_tree() {
        let root = Node::new(
            vec![
                Node::new(
                    vec![
                        Node::from("some text".to_string()),
                        Node::from("more text".to_string()),
                    ],
                    None,
                    ["a"].iter().map(|s| s.to_string()).collect(),
                ),
                Node::new(
                    vec![
                        Node::from("some text".to_string()),
                        Node::from("more text".to_string()),
                    ],
                    None,
                    ["b"].iter().map(|s| s.to_string()).collect(),
                ),
            ],
            Some("root".to_string()),
            HashSet::new(),
        );

        let stylesheet = Stylesheet {
            rules: vec![
                Rule {
                    selectors: vec![Selector {
                        id: Some("root".to_string()),
                        classes: vec![],
                    }],
                    declarations: vec![Declaration {
                        name: "margin".to_string(),
                        value: Value::AbsoluteLength(2),
                    }],
                },
                Rule {
                    selectors: vec![Selector {
                        id: None,
                        classes: vec!["a".to_string()],
                    }],
                    declarations: vec![
                        Declaration {
                            name: "padding".to_string(),
                            value: Value::AbsoluteLength(2),
                        },
                        Declaration {
                            name: "border".to_string(),
                            value: Value::Keyword("solid".to_string()),
                        },
                    ],
                },
                Rule {
                    selectors: vec![Selector {
                        id: None,
                        classes: vec!["b".to_string()],
                    }],
                    declarations: vec![
                        Declaration {
                            name: "width".to_string(),
                            value: Value::RelativeLength(0.5),
                        },
                        Declaration {
                            name: "margin".to_string(),
                            value: Value::Keyword("auto".to_string()),
                        },
                    ],
                },
            ],
        };

        let styled_root = style_tree(&root, &stylesheet);

    }
}
