use crate::style_tree::*;
use crate::style::*;

#[derive(Default, Clone)]
pub struct Dimensions {
    content: Rect,
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}

#[derive(Default, Clone)]
pub struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Default, Clone)]
pub struct EdgeSizes {
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
}

pub struct LayoutBox<'a> {
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}

pub enum BoxType<'a> {
    InlineNode(&'a StyledNode<'a>),
    BlockNode(&'a StyledNode<'a>),
    Anonymous,
}

impl<'a> LayoutBox<'a> {
    pub fn new(box_type: BoxType<'a>) -> LayoutBox<'a> {
        LayoutBox {
            dimensions: Dimensions::default(),
            children: Vec::new(),
            box_type,
        }
    }
}

pub fn build_layout_tree<'a>(styled_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(match styled_node.display() {
        Display::None => panic!("root has display: none;"),
        Display::Inline => BoxType::InlineNode(styled_node),
        Display::Block => BoxType::BlockNode(styled_node),
    });

    for child in &styled_node.children {
        match child.display() {
            Display::None => {}
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root
                .get_inline_container()
                .children
                .push(build_layout_tree(child)),
        }
    }

    root
}

impl<'a> LayoutBox<'a> {
    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::Anonymous => self,
            BoxType::InlineNode(_) => self,
            BoxType::BlockNode(_) => {
                match self.children.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::Anonymous,
                        ..
                    }) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::Anonymous)),
                }
                self.children.last_mut().expect("we just added a box")
            }
        }
    }
}
