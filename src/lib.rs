pub mod content_tree;
pub mod display;
pub mod layout;
pub mod style;
pub mod style_tree;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::content_tree::*;
    use super::display::*;
    use super::layout::*;
    use super::style::*;
    use super::style_tree::*;

    #[test]
    fn build_and_style_and_layout_and_paint_dom() {
        let root = Node::new(
            vec![
                Node::new(
                    vec![
                        Node::from("some text".to_string()),
                        Node::from("more text".to_string()),
                    ],
                    None,
                    ["a", "block"].iter().map(|s| s.to_string()).collect(),
                ),
                Node::new(
                    vec![
                        Node::from("some text".to_string()),
                        Node::from("more text".to_string()),
                    ],
                    None,
                    ["b", "block"].iter().map(|s| s.to_string()).collect(),
                ),
                Node::new(
                    vec![
                        Node::from("some text".to_string()),
                        Node::from("more text".to_string()),
                    ],
                    None,
                    ["c", "block"].iter().map(|s| s.to_string()).collect(),
                ),
            ],
            Some("root".to_string()),
            ["block"].iter().map(|s| s.to_string()).collect(),
        );

        let stylesheet = Stylesheet {
            rules: vec![
                Rule {
                    selectors: vec![Selector {
                        id: Some("root".to_string()),
                        classes: vec![],
                    }],
                    declarations: vec![Declaration {
                        name: "padding".to_string(),
                        value: Value::AbsoluteLength(2),
                    }],
                },
                Rule {
                    selectors: vec![Selector {
                        id: None,
                        classes: vec!["block".to_string()],
                    }],
                    declarations: vec![Declaration {
                        name: "display".to_string(),
                        value: Value::Keyword("block".to_string()),
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
                            name: "border-width".to_string(),
                            value: Value::AbsoluteLength(1),
                        },
                        Declaration {
                            name: "height".to_string(),
                            value: Value::AbsoluteLength(4),
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
                Rule {
                    selectors: vec![Selector {
                        id: None,
                        classes: vec!["c".to_string()],
                    }],
                    declarations: vec![
                        Declaration {
                            name: "width".to_string(),
                            //value: Value::RelativeLength(0.5),
                            value: Value::AbsoluteLength(40),
                        },
                        Declaration {
                            name: "height".to_string(),
                            value: Value::AbsoluteLength(4),
                        },
                        Declaration {
                            name: "margin-left".to_string(),
                            value: Value::AbsoluteLength(6),
                        },
                        Declaration {
                            name: "margin-bottom".to_string(),
                            value: Value::AbsoluteLength(2),
                        },
                        Declaration {
                            name: "border-left-width".to_string(),
                            value: Value::AbsoluteLength(2),
                        },
                        Declaration {
                            name: "border-right-width".to_string(),
                            value: Value::AbsoluteLength(2),
                        },
                        Declaration {
                            name: "border-top-width".to_string(),
                            value: Value::AbsoluteLength(1),
                        },
                        Declaration {
                            name: "border-bottom-width".to_string(),
                            value: Value::AbsoluteLength(1),
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
                print_boxes(child, i + 1);
            }
        }

        //println!("{:?}", layout_root.dimensions);
        print_boxes(&layout_root, 0);

        let mut c = DebugCanvas::new(80, 25);

        c.paint(&build_display_list(&layout_root));

        c.print();


        panic!();
    }

}
