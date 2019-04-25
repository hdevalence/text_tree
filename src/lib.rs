pub mod content_tree;
pub mod layout;
pub mod style;
pub mod style_tree;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::content_tree::*;
    use super::layout::*;
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
                        Declaration {
                            name: "height".to_string(),
                            value: Value::AbsoluteLength(3),
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
                            //value: Value::RelativeLength(0.5),
                            value: Value::AbsoluteLength(20),
                        },
                        Declaration {
                            name: "height".to_string(),
                            value: Value::AbsoluteLength(2),
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

        println!("{:?}", styled_root);

        let mut layout_root = build_layout_tree(&styled_root);

        println!("{:?}", layout_root);

        layout_root.layout(&Dimensions {
            content: Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 0,
            },
            margin: Default::default(),
            padding: Default::default(),
            border: Default::default(),
        });

        println!("{:?}", layout_root);

        fn print_boxes(b: &LayoutBox, i: usize) {
            for _ in 0..i {
                print!(" ");
            }
            print!("{:?}", b.dimensions.content);
            print!("\n");
            for child in &b.children {
                print_boxes(child, i+1);
            }
        }

        //println!("{:?}", layout_root.dimensions);

        print_boxes(&layout_root, 0);

        panic!();
    }
}
