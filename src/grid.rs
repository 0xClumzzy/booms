//! The cell grid: the back buffer widgets paint into.

use unicode_width::UnicodeWidthChar;

use crate::layout::Rect;
use crate::style::{Color, Style};

/// The display width of a character: 0 for control characters and
/// zero-width ones (e.g. combining marks), otherwise the Unicode width.
pub fn char_width(ch: char) -> u16 {
    match ch {
        '\u{0}'..='\u{1f}' | '\u{7f}'..='\u{9f}' => 0,
        _ => UnicodeWidthChar::width(ch).unwrap_or(0) as u16,
    }
}

/// The display width of a string: the sum of its characters' widths.
pub fn str_width(s: &str) -> usize {
    s.chars().map(|c| char_width(c) as usize).sum()
}

/// One cell of the grid: a character plus its [`Style`].
///
/// Semi-transparent cells (opacity < 255) are pre-blended into solid colors
/// when written, so the diff renderer only ever stores resolved colors.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Cell {
    pub const fn new(ch: char, style: Style) -> Cell {
        Cell { ch, style }
    }

    pub const fn blank() -> Cell {
        Cell::new(' ', Style::DEFAULT)
    }
}

/// The back buffer a widget tree paints into.
///
/// Widgets paint into a [`Grid`] as the [`App`](crate::app::App) lays them
/// out; the app diffs the grid against the previous frame and writes only
/// the changed cells to the terminal.
///
/// The grid holds an assumed *canvas* background color (default black) that
/// semi-transparent cells are blended toward, producing crisp transparency
/// on top of any dark terminal theme.
#[derive(Clone, Debug)]
pub struct Grid {
    width: usize,
    height: usize,
    /// The canvas foreground color semi-transparent cells blend toward.
    canvas_fg: Color,
    /// The canvas background color semi-transparent cells blend toward.
    canvas_bg: Color,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: u16, height: u16, fill: Cell) -> Grid {
        let width = width as usize;
        let height = height as usize;
        Grid {
            width,
            height,
            canvas_fg: Color::Default,
            canvas_bg: Color::Default,
            cells: vec![fill; width * height],
        }
    }

    /// Set the canvas colors that semi-transparent cells are blended toward.
    /// Defaults to black `(0, 0, 0)` on both sides; set these to match your
    /// terminal theme (e.g. Catppuccin mocha base `#1e1e2e`) so transparency
    /// looks right against it.
    pub fn set_canvas_colors(&mut self, fg: Color, bg: Color) {
        self.canvas_fg = fg;
        self.canvas_bg = bg;
    }

    /// Resolve a style into solid colors by alpha-blending semi-transparent
    /// cells over the canvas colors. Fully opaque styles pass through.
    fn resolve_style(&self, style: Style) -> Style {
        if style.opacity == 255 {
            return style;
        }
        let alpha = style.opacity;
        let fg = style.fg.alpha_over(self.canvas_fg, alpha);
        let bg = style.bg.alpha_over(self.canvas_bg, alpha);
        Style {
            fg,
            bg,
            mods: style.mods,
            opacity: 255,
        }
    }

    pub fn width(&self) -> u16 {
        self.width as u16
    }

    pub fn height(&self) -> u16 {
        self.height as u16
    }

    /// Read the cell at `(x, y)`; blank when out of bounds.
    pub fn get(&self, x: u16, y: u16) -> Cell {
        if (x as usize) < self.width && (y as usize) < self.height {
            self.cells[y as usize * self.width + x as usize]
        } else {
            Cell::blank()
        }
    }

    /// Write a cell at `(x, y)`; ignored when out of bounds. Semi-transparent
    /// styles are blended over the canvas before storing.
    pub fn set(&mut self, x: u16, y: u16, ch: char, style: Style) {
        if (x as usize) < self.width && (y as usize) < self.height {
            self.cells[y as usize * self.width + x as usize] =
                Cell::new(ch, self.resolve_style(style));
        }
    }

    /// Write `text` with `style` starting at `(x, y)`.
    ///
    /// Wide characters occupy two cells; the second cell is blanked to
    /// match the rendered output. Text is clipped to the grid edge and
    /// zero-width characters are skipped. Returns the next `x` after the
    /// last written character.
    pub fn set_str(&mut self, x: u16, y: u16, text: &str, style: Style) -> u16 {
        let mut x = x;
        for ch in text.chars() {
            let w = char_width(ch);
            if w == 0 {
                continue;
            }
            if (x as usize) >= self.width || (w as usize) > self.width - x as usize {
                break;
            }
            self.set(x, y, ch, style);
            if w > 1 {
                self.set(x + 1, y, ' ', style);
            }
            x += w;
        }
        x
    }

    /// Fill the rectangle with `ch` and `style`. Semi-transparent styles are
    /// blended over the canvas before storing.
    pub fn fill_rect(&mut self, area: Rect, ch: char, style: Style) {
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                self.set(x, y, ch, style);
            }
        }
    }

    /// Set every cell to blank with `style`. Semi-transparent styles are
    /// blended over the canvas before storing.
    pub fn clear(&mut self, style: Style) {
        let resolved = self.resolve_style(style);
        for cell in self.cells.iter_mut() {
            *cell = Cell::new(' ', resolved);
        }
    }

    /// The cells of row `y`; empty when out of bounds.
    pub fn row(&self, y: u16) -> &[Cell] {
        if (y as usize) < self.height {
            let start = y as usize * self.width;
            &self.cells[start..start + self.width]
        } else {
            &[]
        }
    }
}
