use super::layout::*;
use super::style::Border;

pub enum DisplayCommand {
    FilledBox(Rect, char),
    BorderBox(Rect, Borders),
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
    list.push(DisplayCommand::FilledBox(d.content_box(), bg));

    // draw border
    list.push(DisplayCommand::BorderBox(d.border_box(), d.border));

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
                for y in y0..y1 {
                    for x in x0..x1 {
                        self.data[y as usize][x as usize] = *bg;
                    }
                }
            }
            DisplayCommand::BorderBox(rect, borders) => {
                println!("rect, borders: {:?}, {:?}", rect, borders);
                let x0 = self.clamp_x(rect.x);
                let x1 = self.clamp_x(rect.x + rect.width);
                let y0 = self.clamp_y(rect.y);
                let y1 = self.clamp_y(rect.y + rect.height);

                // border characters
                let top_border = match borders.top {
                    Border::None => None,
                    Border::Light => Some('─'),
                    Border::Heavy => Some('━'),
                    Border::Double => Some('═'),
                };
                let bottom_border = match borders.bottom {
                    Border::None => None,
                    Border::Light => Some('─'),
                    Border::Heavy => Some('━'),
                    Border::Double => Some('═'),
                };
                let left_border = match borders.left {
                    Border::None => None,
                    Border::Light => Some('│'),
                    Border::Heavy => Some('┃'),
                    Border::Double => Some('║'),
                };
                let right_border = match borders.right {
                    Border::None => None,
                    Border::Light => Some('│'),
                    Border::Heavy => Some('┃'),
                    Border::Double => Some('║'),
                };
                // corner characters
                let top_left = match (borders.left, borders.top) {
                    (Border::None, _) => top_border,
                    (_, Border::None) => left_border,
                    (Border::Light, Border::Light) => Some('┌'),
                    (Border::Heavy, Border::Light) => Some('┎'),
                    (Border::Light, Border::Heavy) => Some('┍'),
                    (Border::Heavy, Border::Heavy) => Some('┏'),
                    (Border::Double, Border::Double) => Some('╔'),
                    (Border::Double, _) => Some('╓'), // no double-to-heavy corners
                    (_, Border::Double) => Some('╒'),
                };
                let top_right = match (borders.right, borders.top) {
                    (Border::None, _) => top_border,
                    (_, Border::None) => right_border,
                    (Border::Light, Border::Light) => Some('┐'),
                    (Border::Heavy, Border::Light) => Some('┒'),
                    (Border::Light, Border::Heavy) => Some('┑'),
                    (Border::Heavy, Border::Heavy) => Some('┓'),
                    (Border::Double, Border::Double) => Some('╗'),
                    (Border::Double, _) => Some('╖'),
                    (_, Border::Double) => Some('╕'),
                };
                let bottom_left = match (borders.left, borders.bottom) {
                    (Border::None, _) => bottom_border,
                    (_, Border::None) => left_border,
                    (Border::Light, Border::Light) => Some('└'),
                    (Border::Heavy, Border::Light) => Some('┖'),
                    (Border::Light, Border::Heavy) => Some('┕'),
                    (Border::Heavy, Border::Heavy) => Some('┗'),
                    (Border::Double, Border::Double) => Some('╚'),
                    (Border::Double, _) => Some('╙'),
                    (_, Border::Double) => Some('╘'),
                };
                let bottom_right = match (borders.right, borders.bottom) {
                    (Border::None, _) => bottom_border,
                    (_, Border::None) => right_border,
                    (Border::Light, Border::Light) => Some('┘'),
                    (Border::Heavy, Border::Light) => Some('┚'),
                    (Border::Light, Border::Heavy) => Some('┙'),
                    (Border::Heavy, Border::Heavy) => Some('┛'),
                    (Border::Double, Border::Double) => Some('╝'),
                    (Border::Double, _) => Some('╜'),
                    (_, Border::Double) => Some('╛'),
                };

                if let Some(border) = top_border {
                    for x in (x0 + 1)..(x1 - 1) {
                        self.data[y0 as usize][x as usize] = border;
                    }
                }
                if let Some(border) = bottom_border {
                    for x in (x0 + 1)..(x1 - 1) {
                        self.data[(y1 - 1) as usize][x as usize] = border;
                    }
                }
                if let Some(border) = left_border {
                    for y in (y0 + 1)..(y1 - 1) {
                        self.data[y as usize][x0 as usize] = border;
                    }
                }
                if let Some(border) = right_border {
                    for y in (y0 + 1)..(y1 - 1) {
                        self.data[y as usize][(x1 - 1) as usize] = border;
                    }
                }

                if let Some(corner) = top_left {
                    self.data[y0 as usize][x0 as usize] = corner;
                }
                if let Some(corner) = top_right {
                    self.data[y0 as usize][(x1 - 1) as usize] = corner;
                }
                if let Some(corner) = bottom_left {
                    self.data[(y1 - 1) as usize][x0 as usize] = corner;
                }
                if let Some(corner) = bottom_right {
                    self.data[(y1 - 1) as usize][(x1 - 1) as usize] = corner;
                }
            }
        }
    }
}
