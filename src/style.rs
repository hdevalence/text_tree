use super::content_tree::*;

pub struct Stylesheet {
    pub(super) rules: Vec<Rule>,
}

pub struct Rule {
    pub(super) selectors: Vec<Selector>,
    pub(super) declarations: Vec<Declaration>,
}

pub struct Selector {
    //node_type: Option<String>,
    pub(super) id: Option<String>,
    pub(super) classes: Vec<String>,
}

pub struct Declaration {
    pub(super) name: String,
    pub(super) value: Value,
}

#[derive(Clone)]
pub enum Value {
    Keyword(String),
    // An absolute length, in characters.
    AbsoluteLength(u32),
    // A relative length, between 0 and 1.
    RelativeLength(f32),
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
        if self.id.iter().any(|id| element.id != Some(id.to_string())) {
            return false;
        }

        if self
            .classes
            .iter()
            .any(|class| !element.classes.contains(class))
        {
            return false;
        }

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

#[cfg(test)]
mod tests {
    #[test]
    fn tuple_cmp() {
        let a = (1, 1, 3);
        let b = (2, 1, 2);
        assert!(a < b);
    }
}
