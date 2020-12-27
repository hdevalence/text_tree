use super::content_tree::*;
mod parse;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stylesheet {
    pub(super) rules: Vec<Rule>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rule {
    pub(super) selectors: Vec<Selector>,
    pub(super) declarations: Vec<Declaration>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Selector {
    //node_type: Option<String>,
    pub(super) id: Option<String>,
    pub(super) classes: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Declaration {
    pub(super) name: String,
    pub(super) value: Value,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Border {
    None,
    Light,
    Heavy,
    Double,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DisplayKind {
    Inline,
    Block,
}

impl Default for Border {
    fn default() -> Self {
        Border::None
    }
}

impl Border {
    pub fn size(&self) -> i32 {
        match self {
            Border::None => 0,
            _ => 1,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Keyword(String),
    Auto,
    /// An absolute length, in characters.
    AbsoluteLength(i32),
    // A relative length, between 0 and 1.
    //RelativeLength(f32),
    Border(Border),
    Display(DisplayKind),
}

impl Value {
    pub fn to_chars(&self) -> i32 {
        match self {
            // XXX handle relative lengths?
            Value::AbsoluteLength(l) => *l,
            _ => 0,
            Value::Border(b) => b.size(),
        }
    }
}

pub type Specificity = (usize, usize, usize);

impl Selector {
    pub fn specificity(&self) -> Specificity {
        (
            self.id.iter().count(),
            self.classes.len(),
            0, //self.node_type.iter().count(),
        )
    }

    pub fn matches(&self, element: &ElementData) -> bool {
        //println!("checking if {:?} matches {:?}", self, element);
        if self.id.iter().any(|id| element.id != Some(id.to_string())) {
            //println!("id does not match");
            return false;
        }

        if self
            .classes
            .iter()
            .any(|class| !element.classes.contains(class))
        {
            //println!("class does not match");
            return false;
        }

        //println!("matches");
        true
    }
}

type MatchedRule<'a> = (Specificity, &'a Rule);

impl Rule {
    pub fn match_rule<'a, 'b>(&'a self, element: &'b ElementData) -> Option<MatchedRule<'a>> {
        self.selectors
            .iter()
            .find(|selector| selector.matches(element))
            .map(|selector| (selector.specificity(), self))
    }
}

impl Stylesheet {
    pub fn matching_rules<'a, 'b>(&'a self, element: &'b ElementData) -> Vec<MatchedRule<'a>> {
        self.rules
            .iter()
            .filter_map(|rule| rule.match_rule(element))
            .collect()
    }
}

impl std::str::FromStr for Stylesheet {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use nom::Finish;
        match parse::stylesheet(s).finish() {
            Ok((_remaining, stylesheet)) => Ok(stylesheet),
            Err(e) => Err(nom::error::convert_error(s, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tuple_cmp() {
        let a = (1, 1, 3);
        let b = (2, 1, 2);
        assert!(a < b);
    }

    #[test]
    fn parse_example_stylesheet() {
        let text = include_str!("../../example.tss");
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
                    ],
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
                        classes: vec!["class-a".to_string()],
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
                        classes: vec!["class-b".to_string()],
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
                        classes: vec!["class-c".to_string()],
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
                        classes: vec!["class-d".to_string()],
                    }],
                    declarations: vec![Declaration {
                        name: "padding-left".to_string(),
                        value: Value::AbsoluteLength(2),
                    }],
                },
            ],
        };
        assert_eq!(text.parse::<Stylesheet>(), Ok(stylesheet))
    }
}
