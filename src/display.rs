use super::layout::*;

pub enum DisplayCommand {
    FilledBox(Rect, char),
    BorderBox(Rect, EdgeSizes),
}

type DisplayList = Vec<DisplayCommand>;

pub fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root, &mut 0);
    return list;
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox, char_idx: &mut usize) {
    let d = &layout_box.dimensions;

    // choose a bg character
    //let chars: Vec<char> = "'@,\"o&+#a*_!".chars().collect();
    let chars: Vec<char> = "🬠🬡🬢🬣🬤🬥🬦🬧🬨🬩🬪🬫🬬🬭🬮🬯".chars().collect();
    let bg = chars[*char_idx % chars.len()];
    *char_idx += 1;

    // draw background
    list.push(DisplayCommand::FilledBox(d.content, bg));

    // draw border
    list.push(DisplayCommand::BorderBox(d.content, d.border));

    for child in &layout_box.children {
        render_layout_box(list, child, char_idx);
    }
}

pub struct DebugCanvas {
    width: usize,
    height: usize,
    data: Vec<Vec<char>>,
}

impl DebugCanvas {
    pub fn new(width: usize, height: usize) -> DebugCanvas {
        DebugCanvas {
            width,
            height,
            data: (0..height)
                .map(|_| (0..width).map(|_| '⋅').collect())
                .collect(),
        }
    }

    pub fn print(&self) {
        println!("Debug Canvas {}x{}", self.width, self.height);
        for row in &self.data {
            println!("{}", row.iter().collect::<String>());
        }
    }

    pub fn paint(&mut self, display_list: &DisplayList) {
        for item in display_list {
            self.paint_item(item);
        }
    }

    fn clamp_x(&self, x: i32) -> i32 {
        if x < 0 {
            0
        } else if x >= self.width as i32 {
            self.width as i32 - 1
        } else {
            x
        }
    }

    fn clamp_y(&self, y: i32) -> i32 {
        if y < 0 {
            0
        } else if y >= self.height as i32 {
            self.height as i32 - 1
        } else {
            y
        }
    }

    fn paint_item(&mut self, item: &DisplayCommand) {
        match item {
            DisplayCommand::FilledBox(rect, bg) => {
                let x0 = self.clamp_x(rect.x);
                let x1 = self.clamp_x(rect.x + rect.width);
                let y0 = self.clamp_y(rect.y);
                let y1 = self.clamp_y(rect.y + rect.height);
                for y in (y0..y1) {
                    for x in (x0..x1) {
                        self.data[y as usize][x as usize] = *bg;
                    }
                }
            }
            DisplayCommand::BorderBox(rect, edges) => {
                println!("rect, edges: {:?}, {:?}", rect, edges);
                use std::cmp::{min, max};
                let x0 = self.clamp_x(rect.x);
                let x1 = self.clamp_x(rect.x + rect.width);
                let y0 = self.clamp_y(rect.y);
                let y1 = self.clamp_y(rect.y + rect.height);
                // is the border inside or outside the content?
                /* inside
                // top border
                println!("y = {}..{}", y0, min(y0 + edges.top, y1));
                for y in y0..min(y0 + edges.top, y1) {
                    for x in (x0..x1) {
                        self.data[y as usize][x as usize] = '-';
                    }
                }
                // right border
                for x in max(x0, x1 - edges.right)..x1 {
                    for y in (y0..y1) {
                        self.data[y as usize][x as usize] = '|';
                    }
                }
                // bottom border
                for y in max(y0, y1 - edges.top)..y1 {
                    for x in (x0..x1) {
                        self.data[y as usize][x as usize] = '-';
                    }
                }
                // left border
                for x in x0..min(x0 + edges.left, x1) {
                    for y in (y0..y1) {
                        self.data[y as usize][x as usize] = '|';
                    }
                }
                */
                // top border
                for y in (y0-edges.top)..y0 {
                    for x in (x0..x1) {
                        self.data[y as usize][x as usize] = '-';
                    }
                }
                // right border
                for x in x1..(x1+edges.right) {
                    for y in (y0..y1) {
                        self.data[y as usize][x as usize] = '|';
                    }
                }
                // bottom border
                for y in y1..(y1+edges.bottom) {
                    for x in (x0..x1) {
                        self.data[y as usize][x as usize] = '-';
                    }
                }
                // left border
                for x in (x0-edges.left)..x0 {
                    for y in (y0..y1) {
                        self.data[y as usize][x as usize] = '|';
                    }
                }
            }
        }
    }
}
