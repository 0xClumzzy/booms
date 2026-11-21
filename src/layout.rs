//! Layout: rectangles and how they divide a container.

/// A rectangle in grid coordinates: origin `(x, y)`, size `(width, height)`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// One past the last column.
    pub const fn right(&self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// One past the last row.
    pub const fn bottom(&self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// True when the rect has zero width or height.
    pub const fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// How a [`Constraint`] leaves a container's length to be used.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Constraint {
    /// Take exactly `n` cells (clamped to what is left).
    Exact(u16),
    /// Take up to `n` cells (0 if the leftovers are smaller; no minimum).
    Flex(u16),
    /// Take at least `n` cells (if any are left).
    Min(u16),
}

/// Divide `length` among `constraints`, smallest-first.
///
/// Pass 1 assigns [`Exact`](Constraint::Exact), pass 2 [`Min`](Constraint::Min),
/// pass 3 [`Flex`](Constraint::Flex) shares of what is left, with the last
/// flexible widget taking the remainder.
#[allow(clippy::manual_checked_ops)]
pub fn layout(constraints: &[Constraint], length: u16) -> Vec<u16> {
    let mut sizes = vec![0u16; constraints.len()];
    let mut remaining = length;

    // Pass 1: exact.
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Exact(n) = *c {
            let n = n.min(remaining);
            sizes[i] = n;
            remaining = remaining.saturating_sub(n);
        }
    }

    // Pass 2: minimum.
    for (i, c) in constraints.iter().enumerate() {
        if let Constraint::Min(n) = *c {
            if remaining > 0 && n > 0 {
                let n = n.min(remaining);
                sizes[i] += n;
                remaining = remaining.saturating_sub(n);
            }
        }
    }

    // Pass 3: flex shares.
    if remaining > 0 {
        let flex: Vec<(usize, u16)> = constraints
            .iter()
            .enumerate()
            .filter_map(|(i, c)| match *c {
                Constraint::Flex(w) => Some((i, w)),
                _ => None,
            })
            .collect();
        let total: u64 = flex.iter().map(|(_, w)| *w as u64).sum();
        if total > 0 {
            let n_flex = flex.len();
            let mut acc: u64 = 0;
            for (j, (i, w)) in flex.into_iter().enumerate() {
                if w == 0 {
                    continue;
                }
                let share: u64 = if j + 1 == n_flex {
                    remaining as u64 - acc
                } else {
                    ((remaining as u64 * w as u64) / total).min(remaining as u64 - acc)
                };
                sizes[i] = share as u16;
                acc += share;
            }
        }
    }

    sizes
}

/// Stack rects along the x axis inside `area`.
pub fn layout_row(area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
    let mut out = Vec::with_capacity(constraints.len());
    let mut x = area.x;
    for w in layout(constraints, area.width) {
        out.push(Rect::new(x, area.y, w, area.height));
        x = x.saturating_add(w);
    }
    out
}

/// Stack rects along the y axis inside `area`.
pub fn layout_col(area: Rect, constraints: &[Constraint]) -> Vec<Rect> {
    let mut out = Vec::with_capacity(constraints.len());
    let mut y = area.y;
    for h in layout(constraints, area.height) {
        out.push(Rect::new(area.x, y, area.width, h));
        y = y.saturating_add(h);
    }
    out
}
