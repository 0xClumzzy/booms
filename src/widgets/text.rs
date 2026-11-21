//! Text: strings wrapped and painted into the grid.

use crate::grid::{Grid, char_width, str_width};
use crate::layout::{Constraint, Rect};
use crate::style::Style;
use crate::widgets::Widget;

/// A block of text, wrapped to the widget's width.
#[derive(Clone, Debug, Default)]
pub struct Text {
    lines: Vec<String>,
    style: Style,
}

impl Text {
    pub fn new(lines: impl IntoIterator<Item = impl Into<String>>) -> Text {
        Text {
            lines: lines.into_iter().map(Into::into).collect(),
            style: Style::DEFAULT,
        }
    }

    pub fn line(line: impl Into<String>) -> Text {
        Text::new([line.into()])
    }

    pub fn style(mut self, style: Style) -> Text {
        self.style = style;
        self
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

impl Widget for Text {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(self.lines.len() as u16)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        let mut y = area.y;
        for line in &self.lines {
            for piece in wrap_line(line, area.width as usize) {
                if y >= area.bottom() {
                    return;
                }
                grid.set_str(area.x, y, &piece, self.style);
                y += 1;
            }
        }
    }
}

/// Break `line` into pieces of at most `width` display columns, on
/// whitespace where possible and hard-splitting long words.
pub fn wrap_line(line: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut cur_w = 0usize;
    for word in line.split_whitespace() {
        let w = str_width(word);
        let sep = if cur_w == 0 { 0 } else { 1 };
        if cur_w + sep + w > width {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
                cur_w = 0;
            }
            if w > width {
                let mut rest = word;
                while !rest.is_empty() {
                    match take_prefix(rest, width) {
                        Some((piece, tail)) => {
                            out.push(piece);
                            rest = tail;
                        }
                        None => break,
                    }
                }
                continue;
            }
        }
        if cur_w > 0 {
            cur.push(' ');
            cur_w += 1;
        }
        cur.push_str(word);
        cur_w += w;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// The longest prefix of `s` whose width is at most `width`, plus the rest.
fn take_prefix(s: &str, width: usize) -> Option<(String, &str)> {
    let mut piece = String::new();
    let mut w = 0usize;
    for (i, ch) in s.char_indices() {
        let cw = char_width(ch) as usize;
        if w + cw > width && !piece.is_empty() {
            return Some((piece, &s[i..]));
        }
        piece.push(ch);
        w += cw;
    }
    if piece.is_empty() {
        None
    } else {
        Some((piece, ""))
    }
}
