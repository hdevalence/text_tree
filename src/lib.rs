pub mod content_tree;
pub mod display;
pub mod layout;
pub mod style;
pub mod style_tree;

pub fn print_boxes(b: &layout::LayoutBox) {
    fn print_boxes2(b: &layout::LayoutBox, i: usize) {
        for _ in 0..i {
            print!(" ");
        }
        print!("{:?}", b.dimensions.border_box);
        print!("\n");
        for child in &b.children {
            print_boxes2(child, i + 1);
        }
    }
    print_boxes2(b, 0)
}

#[cfg(test)]
mod tests {
    use super::content_tree::*;
    use super::display::*;
    use super::layout::*;
    use super::style::*;
    use super::style_tree::*;

    fn trace_init() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .with_max_level(tracing::Level::TRACE)
            .try_init();
    }

    #[test]
    fn build_and_style_and_layout_and_paint_dom() {
        trace_init();

        let root = Node::new(
            vec![
                Node::new(
                    vec![
                        Node::from("some 🥰 text".to_string()),
                        //Node::from("more text".to_string()),
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
                        Node::from(
                            "some very long text that needs line breaking to work".to_string(),
                        ),
                        Node::from("more text".to_string()),
                    ],
                    None,
                    ["c", "block"].iter().map(|s| s.to_string()).collect(),
                ),
                Node::new(
                    vec![
                        Node::from("indented".to_string()),
                        Node::new(
                            vec![
                                Node::from("indented".to_string()),
                                Node::new(
                                    vec![
                                        Node::from("indented".to_string()),
                                        Node::new(
                                            vec![Node::from("indented".to_string())],
                                            None,
                                            ["d", "block"].iter().map(|s| s.to_string()).collect(),
                                        ),
                                    ],
                                    None,
                                    ["d", "block"].iter().map(|s| s.to_string()).collect(),
                                ),
                            ],
                            None,
                            ["d", "block"].iter().map(|s| s.to_string()).collect(),
                        ),
                    ],
                    None,
                    ["d", "block"].iter().map(|s| s.to_string()).collect(),
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
                    declarations: vec![
                        Declaration {
                            name: "padding".to_string(),
                            value: Value::AbsoluteLength(2),
                        },
                        Declaration {
                            name: "margin".to_string(),
                            value: Value::AbsoluteLength(3),
                        },
                        Declaration {
                            name: "border".to_string(),
                            value: Value::Border(Border::Double),
                        },
                    ],
                },
                Rule {
                    selectors: vec![Selector {
                        id: None,
                        classes: vec!["block".to_string()],
                    }],
                    declarations: vec![Declaration {
                        name: "display".to_string(),
                        value: Value::Display(DisplayKind::Block),
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
                            value: Value::Border(Border::Light),
                        },
                        Declaration {
                            name: "height".to_string(),
                            value: Value::AbsoluteLength(12),
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
                            value: Value::Auto,
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
                            name: "border-left".to_string(),
                            value: Value::Border(Border::Double),
                        },
                        Declaration {
                            name: "border-right".to_string(),
                            value: Value::Border(Border::Double),
                        },
                        Declaration {
                            name: "border-top".to_string(),
                            value: Value::Border(Border::Light),
                        },
                        Declaration {
                            name: "border-bottom".to_string(),
                            value: Value::Border(Border::Heavy),
                        },
                    ],
                },
                Rule {
                    selectors: vec![Selector {
                        id: None,
                        classes: vec!["d".to_string()],
                    }],
                    declarations: vec![Declaration {
                        name: "padding-left".to_string(),
                        value: Value::AbsoluteLength(2),
                    }],
                },
            ],
        };

        let styled_root = style_tree(&root, &stylesheet);

        //println!("{:#?}", styled_root);

        let mut layout_root = build_layout_tree(&styled_root);

        //println!("{:#?}", layout_root);

        layout_root.layout(&Dimensions {
            border_box: Rect {
                x: 0,
                y: 0,
                width: 80,
                height: 0,
            },
            margin: Default::default(),
            padding: Default::default(),
            border: Default::default(),
        });

        //println!("{:#?}", layout_root);

        //println!("{:?}", layout_root.dimensions);
        super::print_boxes(&layout_root);

        let mut c = DebugCanvas::new(80, 35);

        c.paint(&build_display_list(&layout_root));

        c.print();

        panic!();
    }
}
