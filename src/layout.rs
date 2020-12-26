use crate::style::*;
use crate::style_tree::*;

#[derive(Debug, Default, Copy, Clone)]
pub struct Dimensions {
    pub(crate) border_box: Rect,
    pub(crate) padding: EdgeSizes,
    pub(crate) border: Borders,
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

#[derive(Debug, Default, Copy, Clone)]
pub struct Borders {
    pub(crate) left: Border,
    pub(crate) right: Border,
    pub(crate) top: Border,
    pub(crate) bottom: Border,
}

impl Borders {
    fn sizes(&self) -> EdgeSizes {
        EdgeSizes {
            left: self.left.size(),
            right: self.right.size(),
            top: self.top.size(),
            bottom: self.bottom.size(),
        }
    }
}

impl Dimensions {
    // The area covered by the border-box plus margin.
    pub fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(&self.margin)
    }
    // The area covered by the border-box (element width and height)
    pub fn border_box(&self) -> Rect {
        self.border_box
    }
    // The area covered by the border box minus the border.
    pub fn padding_box(&self) -> Rect {
        self.border_box.contracted_by(&self.border.sizes())
    }
    // The area covered by the border box minus the border and padding.
    pub fn content_box(&self) -> Rect {
        self.padding_box().contracted_by(&self.padding)
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

    fn contracted_by(&self, edges: &EdgeSizes) -> Rect {
        use std::cmp::max;
        Rect {
            x: self.x + edges.left,
            y: self.y + edges.top,
            width: max(self.width - edges.left - edges.right, 0),
            height: max(self.height - edges.top - edges.bottom, 0),
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
    fn get_style_node(&self) -> &StyledNode<'a> {
        match self.box_type {
            BoxType::Anonymous => {
                unimplemented!("Need to walk up the tree");
            }
            BoxType::InlineNode(s) | BoxType::BlockNode(s) => s,
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::Anonymous => self,
            BoxType::InlineNode(_) => self,
            BoxType::BlockNode(_) => {
                // If we just added a new anonymous box, keep using it.  Otherwise,
                // add a new one.
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
            BoxType::Anonymous => {
                println!("anonymous layout");
                self.dimensions = containing_block.clone();
                self.layout_inline_children();
            }
            BoxType::InlineNode(_) => {
                self.layout_inline(containing_block);
            }
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

    fn calculate_block_width(&mut self, containing_block: &Dimensions) {
        let style = self.get_style_node();

        let auto = Value::Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());

        use Value::AbsoluteLength;
        let zero = AbsoluteLength(0);

        let mut margin_left = style.lookup("margin-left", "margin", &zero).clone();
        let mut margin_right = style.lookup("margin-right", "margin", &zero).clone();

        let border_left = if let Value::Border(b) =
            style.lookup("border-left", "border", &Value::Border(Border::None))
        {
            b
        } else {
            Border::None
        };
        let border_right = if let Value::Border(b) =
            style.lookup("border-right", "border", &Value::Border(Border::None))
        {
            b
        } else {
            Border::None
        };

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let total = [&margin_left, &margin_right, &width]
            .iter()
            .map(|v| v.to_chars())
            .sum::<i32>();

        // If width is not auto and the total is wider than the container,
        // treat auto margins as 0.
        if width != auto && total > containing_block.content_box().width {
            if margin_left == auto {
                margin_left = AbsoluteLength(0);
            }
            if margin_right == auto {
                margin_right = AbsoluteLength(0);
            }
        }

        // Calculate box underflow
        let underflow = containing_block.content_box().width - total;

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

        d.border_box.width = width.to_chars();
        d.padding.left = padding_left.to_chars();
        d.padding.right = padding_right.to_chars();
        d.margin.left = margin_left.to_chars();
        d.margin.right = margin_right.to_chars();
        d.border.left = border_left;
        d.border.right = border_right;
    }

    fn calculate_block_position(&mut self, containing_block: &Dimensions) {
        use Value::AbsoluteLength;

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

        d.border.top = if let Value::Border(b) =
            style.lookup("border-top", "border", &Value::Border(Border::None))
        {
            b
        } else {
            Border::None
        };
        d.border.bottom = if let Value::Border(b) =
            style.lookup("border-bottom", "border", &Value::Border(Border::None))
        {
            b
        } else {
            Border::None
        };

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_chars();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_chars();
        // Initialize the height with the size of the vertical padding.
        d.border_box.height = d.padding.top + d.padding.bottom;

        d.border_box.x = containing_block.content_box().x + d.margin.left;

        // Position the box below all the previous boxes in the container.
        println!(
            "containing content\n\t{:?},\nd.margin\n\t{:?}",
            containing_block.content_box(),
            d.margin
        );
        d.border_box.y =
            containing_block.content_box().y + containing_block.content_box().height + d.margin.top;
    }

    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(d);
            // Track the height so each child is laid out below the previous content.
            d.border_box.height += child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        // If the height is set to an explicit length, use that exact length.
        // Otherwise, the height is the size set by `layout_block_children`.
        if let Some(Value::AbsoluteLength(h)) = self.get_style_node().value("height") {
            self.dimensions.border_box.height = h;
        }
    }

    fn layout_inline(&mut self, containing_block: &Dimensions) {
        println!("layout inline {:?}", containing_block);
        self.calculate_inline_position(containing_block);
        self.layout_inline_children();
        self.calculate_inline_width(containing_block);
    }

    fn calculate_inline_position(&mut self, containing_block: &Dimensions) {
        let style = self.get_style_node();

        use Value::AbsoluteLength;
        let zero = AbsoluteLength(0);

        let margin_left = style.lookup("margin-left", "margin", &zero).to_chars();
        let margin_right = style.lookup("margin-right", "margin", &zero).to_chars();
        let padding_left = style.lookup("padding-left", "padding", &zero).to_chars();
        let padding_right = style.lookup("padding-right", "padding", &zero).to_chars();

        let d = &mut self.dimensions;
        d.margin.left = margin_left;
        d.margin.right = margin_right;
        d.padding.left = padding_left;
        d.padding.right = padding_right;

        // Inline elements have height 1 (?)
        d.border_box.height = 1;
        d.border_box.width = d.padding.left + d.padding.right;
        d.border_box.x = containing_block.content_box().x + d.margin.left;
        d.border_box.y = containing_block.content_box().y + containing_block.content_box().height;
        println!("calculated inline position {:?}", d.border_box);
    }

    fn layout_inline_children(&mut self) {
        let mut left_space = self.dimensions.clone();
        for child in &mut self.children {
            child.layout(&mut left_space);
            // Move to the left so that each child is laid out after
            // the previous children. TODO; line breaks.
            let child_width = child.dimensions.margin_box().width;
            println!("laid out child, adding its width {}", child_width);
            left_space.border_box.x += child_width;
            self.dimensions.border_box.width += child_width;
        }
    }

    fn calculate_inline_width(&mut self, containing_block: &Dimensions) {
        // If this is a text element, add the length of the string.
        // Otherwise, the width is computed by `layout_inline_children`.
        if let Some(text) = self.get_style_node().node().text() {
            self.dimensions.border_box.width += text.chars().count() as i32;
        }
    }
}
