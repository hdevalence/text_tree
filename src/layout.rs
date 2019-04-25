use crate::style::*;
use crate::style_tree::*;

#[derive(Debug, Default, Copy, Clone)]
pub struct Dimensions {
    pub(crate) content: Rect,
    pub(crate) padding: EdgeSizes,
    pub(crate) border: EdgeSizes,
    pub(crate) margin: EdgeSizes,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Rect {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct EdgeSizes {
    pub(crate) left: i32,
    pub(crate) right: i32,
    pub(crate) top: i32,
    pub(crate) bottom: i32,
}

impl Dimensions {
    // The area covered by the content area plus its padding.
    fn padding_box(self) -> Rect {
        self.content.expanded_by(&self.padding)
    }
    // The area covered by the content area plus padding and borders.
    fn border_box(self) -> Rect {
        self.padding_box().expanded_by(&self.border)
    }
    // The area covered by the content area plus padding, borders, and margin.
    fn margin_box(self) -> Rect {
        self.border_box().expanded_by(&self.margin)
    }
}

impl Rect {
    fn expanded_by(&self, edges: &EdgeSizes) -> Rect {
        Rect {
            x: self.x - edges.left,
            y: self.y - edges.top,
            width: self.width + edges.left + edges.right,
            height: self.height + edges.top + edges.bottom,
        }
    }
}

#[derive(Debug)]
pub struct LayoutBox<'a> {
    pub(crate) dimensions: Dimensions,
    pub(crate) box_type: BoxType<'a>,
    pub(crate) children: Vec<LayoutBox<'a>>,
}

#[derive(Debug)]
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

    pub fn layout(&mut self, containing_block: &Dimensions) {
        match self.box_type {
            BoxType::Anonymous => {}
            BoxType::InlineNode(_) => {}
            BoxType::BlockNode(_) => {
                self.layout_block(containing_block);
            }
        }
    }

    fn layout_block(&mut self, containing_block: &Dimensions) {
        // Child width can depend on parent width, so we need to calculate
        // this box's width before laying out its children.
        self.calculate_block_width(containing_block);

        // Determine where the box is located within its container.
        self.calculate_block_position(containing_block);

        // Recursively lay out the children of this box.
        self.layout_block_children();

        // Parent height can depend on child height, so `calculate_height`
        // must be called *after* the children are laid out.
        self.calculate_block_height();
    }

    fn get_style_node(&self) -> &StyledNode<'a> {
        match self.box_type {
            BoxType::Anonymous => {
                unimplemented!("Need to walk up the tree");
            }
            BoxType::InlineNode(s) | BoxType::BlockNode(s) => s,
        }
    }

    fn calculate_block_width(&mut self, containing_block: &Dimensions) {
        use Value::*;

        let style = self.get_style_node();

        let auto = Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());

        let zero = AbsoluteLength(0);

        let mut margin_left = style.lookup("margin-left", "margin", &zero).clone();
        let mut margin_right = style.lookup("margin-right", "margin", &zero).clone();

        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border-width", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let total = [
            &margin_left,
            &margin_right,
            &border_left,
            &border_right,
            &padding_left,
            &padding_right,
            &width,
        ]
        .iter()
        .map(|v| v.to_chars())
        .sum::<i32>();

        // If width is not auto and the total is wider than the container,
        // treat auto margins as 0.
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = AbsoluteLength(0);
            }
            if margin_right == auto {
                margin_right = AbsoluteLength(0);
            }
        }

        // Calculate box underflow
        let underflow = containing_block.content.width - total;

        match (width == auto, margin_left == auto, margin_right == auto) {
            // If the values are overconstrained, calculate margin_right.
            (false, false, false) => {
                margin_right = AbsoluteLength(margin_right.to_chars() + underflow);
            }

            // If exactly one size is auto, its used value follows from the equality.
            (false, false, true) => {
                margin_right = AbsoluteLength(underflow);
            }
            (false, true, false) => {
                margin_left = AbsoluteLength(underflow);
            }

            // If width is set to auto, any other auto values become 0.
            (true, _, _) => {
                if margin_left == auto {
                    margin_left = AbsoluteLength(0);
                }
                if margin_right == auto {
                    margin_right = AbsoluteLength(0);
                }

                if underflow >= 0 {
                    // Expand width to fill the underflow.
                    width = AbsoluteLength(underflow);
                } else {
                    // Width can't be negative. Adjust the right margin instead.
                    width = AbsoluteLength(0);
                    margin_right = AbsoluteLength(margin_right.to_chars() + underflow);
                }
            }

            // If margin-left and margin-right are both auto, their used values are equal.
            (false, true, true) => {
                // Computing this way ensures l + r = u in the presence of rounding.
                let l = underflow / 2;
                let r = underflow - l;
                margin_left = AbsoluteLength(l);
                margin_right = AbsoluteLength(r);
            }
        }

        let d = &mut self.dimensions;

        d.content.width = width.to_chars();
        d.padding.left = padding_left.to_chars();
        d.padding.right = padding_right.to_chars();
        d.border.left = border_left.to_chars();
        d.border.right = border_right.to_chars();
        d.margin.left = margin_left.to_chars();
        d.margin.right = margin_right.to_chars();
    }

    fn calculate_block_position(&mut self, containing_block: &Dimensions) {
        use Value::*;

        let d = &mut self.dimensions;
        //let style = self.get_style_node();
        let style = match self.box_type {
            BoxType::Anonymous => {
                unimplemented!("Need to walk up the tree");
            }
            BoxType::InlineNode(s) | BoxType::BlockNode(s) => s,
        };

        // margin, border, and padding have initial value 0.
        let zero = AbsoluteLength(0);

        // If margin-top or margin-bottom is `auto`, the used value is zero.
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_chars();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_chars();

        d.border.top = style
            .lookup("border-top-width", "border-width", &zero)
            .to_chars();
        d.border.bottom = style
            .lookup("border-bottom-width", "border-width", &zero)
            .to_chars();

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_chars();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_chars();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;

        // Position the box below all the previous boxes in the container.
        d.content.y = containing_block.content.height
            + containing_block.content.y
            + d.margin.top
            + d.border.top
            + d.padding.top;
    }

    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(d);
            // Track the height so each child is laid out below the previous content.
            d.content.height += child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        // If the height is set to an explicit length, use that exact length.
        // Otherwise, just keep the value set by `layout_block_children`.
        if let Some(Value::AbsoluteLength(h)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h;
        }
    }
}
