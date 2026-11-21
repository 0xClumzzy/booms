//! Ratatui-inspired data widgets: progress gauges, sparklines, bar charts,
//! tabs and tables. They follow booms' reference-based [`Widget`] trait and
//! paint directly into the [`Grid`]: no extra dependencies.

use crate::grid::{Grid, char_width};
use crate::layout::{Constraint, Rect};
use crate::style::{Color, Modifiers, Style};
use crate::widgets::Widget;

/// A filled-block progress gauge with a percentage label.
pub struct Gauge {
    ratio: f32,
    label: Option<String>,
    filled: char,
    empty: char,
    color: Color,
}

impl Gauge {
    pub fn new(ratio: f32) -> Gauge {
        Gauge {
            ratio: ratio.clamp(0.0, 1.0),
            label: None,
            filled: '█',
            empty: '░',
            color: Color::Ansi(10),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Gauge {
        self.label = Some(label.into());
        self
    }

    pub fn color(mut self, color: Color) -> Gauge {
        self.color = color;
        self
    }

    pub fn ratio(mut self, ratio: f32) -> Gauge {
        self.ratio = ratio.clamp(0.0, 1.0);
        self
    }
}

impl Widget for Gauge {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let text = match &self.label {
            Some(l) => format!(" {} ", l),
            None => format!(" {}% ", (self.ratio * 100.0).round() as u16),
        };
        grid.set_str(area.x, area.y, &text, Style::DEFAULT.fg(Color::Ansi(15)));
        let bar_start = area.x + char_width_first(&text);
        let bar_w = area.right().saturating_sub(bar_start);
        if bar_w == 0 {
            return;
        }
        let filled_n = (self.ratio * bar_w as f32).round() as u16;
        let filled_style = Style::DEFAULT.fg(self.color).mods(Modifiers::BOLD);
        let empty_style = Style::DEFAULT.fg(Color::Ansi(8));
        for i in 0..bar_w {
            let ch = if i < filled_n {
                self.filled
            } else {
                self.empty
            };
            let st = if i < filled_n {
                filled_style
            } else {
                empty_style
            };
            grid.set(bar_start + i, area.y, ch, st);
        }
    }
}

fn char_width_first(s: &str) -> u16 {
    s.chars().map(char_width).sum()
}

/// A compact single-line gauge using a thin track and marker.
pub struct LineGauge {
    ratio: f32,
    label: Option<String>,
    color: Color,
}

impl LineGauge {
    pub fn new(ratio: f32) -> LineGauge {
        LineGauge {
            ratio: ratio.clamp(0.0, 1.0),
            label: None,
            color: Color::Ansi(6),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> LineGauge {
        self.label = Some(label.into());
        self
    }

    pub fn color(mut self, color: Color) -> LineGauge {
        self.color = color;
        self
    }
}

impl Widget for LineGauge {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let label = self.label.as_deref().unwrap_or("");
        let label_w = char_width_first(label);
        let bar_w = area.width.saturating_sub(label_w).saturating_sub(1);
        if label_w > 0 {
            grid.set_str(area.x, area.y, label, Style::DEFAULT.fg(Color::Ansi(7)));
        }
        let bx = area.x + label_w + 1;
        let pos = (self.ratio * bar_w as f32).round() as u16;
        for i in 0..bar_w {
            let ch = if i == pos {
                '┿'
            } else if i < pos {
                '━'
            } else {
                '─'
            };
            let st = if i == pos {
                Style::DEFAULT.fg(Color::Ansi(15)).mods(Modifiers::BOLD)
            } else if i < pos {
                Style::DEFAULT.fg(self.color)
            } else {
                Style::DEFAULT.fg(Color::Ansi(8))
            };
            grid.set(bx + i, area.y, ch, st);
        }
    }
}

/// A sparkline: a row of vertical bars from a series of values.
pub struct Sparkline {
    data: Vec<u64>,
    color: Color,
}

impl Sparkline {
    pub fn new<I: IntoIterator<Item = u64>>(data: I) -> Sparkline {
        Sparkline {
            data: data.into_iter().collect(),
            color: Color::Ansi(6),
        }
    }

    pub fn color(mut self, color: Color) -> Sparkline {
        self.color = color;
        self
    }
}

impl Widget for Sparkline {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.width == 0 || area.height == 0 || self.data.is_empty() {
            return;
        }
        let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        let max = self.data.iter().copied().max().unwrap_or(1).max(1) as f32;
        for i in 0..area.width as usize {
            let idx = (i * self.data.len() / area.width as usize).min(self.data.len() - 1);
            let v = self.data[idx] as f32 / max;
            let level = ((v * (bars.len() - 1) as f32).round() as usize).min(bars.len() - 1);
            grid.set(
                area.x + i as u16,
                area.y,
                bars[level],
                Style::DEFAULT.fg(self.color),
            );
        }
    }
}

/// A horizontal bar chart with labels.
pub struct BarChart {
    bars: Vec<(String, u64)>,
    color: Color,
}

impl BarChart {
    pub fn new<I, S>(bars: I) -> BarChart
    where
        I: IntoIterator<Item = (S, u64)>,
        S: Into<String>,
    {
        BarChart {
            bars: bars.into_iter().map(|(s, v)| (s.into(), v)).collect(),
            color: Color::Ansi(12),
        }
    }

    pub fn color(mut self, color: Color) -> BarChart {
        self.color = color;
        self
    }
}

impl Widget for BarChart {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(self.bars.len() as u16)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.height == 0 || self.bars.is_empty() {
            return;
        }
        let label_w = self
            .bars
            .iter()
            .map(|(l, _)| char_width_first(l) as usize)
            .max()
            .unwrap_or(0)
            .min(area.width as usize)
            + 1;
        let bar_w = area.width as usize - label_w;
        let max = self.bars.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1) as f32;
        for (i, (label, value)) in self.bars.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                break;
            }
            grid.set_str(area.x, y, label, Style::DEFAULT.fg(Color::Ansi(7)));
            let n = ((*value as f32 / max) * bar_w as f32).round() as usize;
            for j in 0..bar_w {
                grid.set(
                    area.x + label_w as u16 + j as u16,
                    y,
                    if j < n { '█' } else { ' ' },
                    Style::DEFAULT.fg(self.color),
                );
            }
        }
    }
}

/// A tab bar with a selected tab, styled after ratatui's `Tabs`.
pub struct Tabs {
    titles: Vec<String>,
    selected: usize,
    style: Style,
    selected_style: Style,
}

impl Tabs {
    pub fn new<I, S>(titles: I) -> Tabs
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Tabs {
            titles: titles.into_iter().map(Into::into).collect(),
            selected: 0,
            style: Style::DEFAULT.fg(Color::Ansi(7)),
            selected_style: Style::DEFAULT.fg(Color::Ansi(14)).mods(Modifiers::BOLD),
        }
    }

    pub fn selected(mut self, selected: usize) -> Tabs {
        self.selected = selected;
        self
    }

    pub fn select(&mut self, selected: usize) {
        self.selected = selected.min(self.titles.len().saturating_sub(1));
    }

    pub fn selection(&self) -> usize {
        self.selected
    }
}

impl Widget for Tabs {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.height == 0 {
            return;
        }
        let mut x = area.x;
        for (i, title) in self.titles.iter().enumerate() {
            let st = if i == self.selected {
                self.selected_style
            } else {
                self.style
            };
            let label = format!(" {} ", title);
            let end = grid.set_str(x, area.y, &label, st);
            x = end;
            if i + 1 < self.titles.len() {
                grid.set(x, area.y, '│', Style::DEFAULT.fg(Color::Ansi(8)));
                x += 1;
            }
            if x >= area.right() {
                break;
            }
        }
    }
}

/// A simple table with a header row and data rows.
pub struct Table {
    header: Vec<String>,
    rows: Vec<Vec<String>>,
    header_style: Style,
    row_style: Style,
}

impl Table {
    pub fn new(header: Vec<String>, rows: Vec<Vec<String>>) -> Table {
        Table {
            header,
            rows,
            header_style: Style::DEFAULT.fg(Color::Ansi(14)).mods(Modifiers::BOLD),
            row_style: Style::DEFAULT.fg(Color::Ansi(7)),
        }
    }
}

impl Widget for Table {
    fn constraint(&self) -> Constraint {
        Constraint::Exact(self.rows.len() as u16 + 1)
    }

    fn render(&self, area: Rect, grid: &mut Grid) {
        if area.height == 0 {
            return;
        }
        let cols = self.header.len();
        if cols == 0 {
            return;
        }
        let widths: Vec<usize> = (0..cols)
            .map(|c| {
                let mut w = self.header[c].chars().count();
                for row in &self.rows {
                    if let Some(cell) = row.get(c) {
                        w = w.max(cell.chars().count());
                    }
                }
                w + 2
            })
            .collect();
        let mut draw_row = |y: u16, cells: &[String], style: Style| {
            let mut x = area.x;
            for (c, cell) in cells.iter().enumerate() {
                if x >= area.right() {
                    break;
                }
                grid.set_str(x, y, cell, style);
                x += widths[c].min((area.right() - x) as usize) as u16;
            }
        };
        if area.y < area.bottom() {
            draw_row(area.y, &self.header, self.header_style);
        }
        for (i, row) in self.rows.iter().enumerate() {
            let y = area.y + 1 + i as u16;
            if y >= area.bottom() {
                break;
            }
            draw_row(y, row, self.row_style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_fills_proportionally() {
        let g = Gauge::new(0.5);
        let mut grid = Grid::new(12, 1, crate::grid::Cell::blank());
        g.render(Rect::new(0, 0, 12, 1), &mut grid);
        let filled = (0..12).filter(|&x| grid.get(x, 0).ch == '█').count();
        assert!(filled > 0 && filled < 12);
    }

    #[test]
    fn sparkline_uses_bars() {
        let s = Sparkline::new([1, 5, 3, 9]);
        let mut grid = Grid::new(4, 1, crate::grid::Cell::blank());
        s.render(Rect::new(0, 0, 4, 1), &mut grid);
        assert_eq!(grid.get(3, 0).ch, '█');
    }
}
