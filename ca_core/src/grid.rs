use std::ops::{Index, IndexMut};

/// A `width` x `height` array of cells, stored row by row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T: Copy> Grid<T> {
    pub fn new(width: usize, height: usize, fill: T) -> Self {
        Self {
            width,
            height,
            cells: vec![fill; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn index_of(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width && y < self.height);
        y * self.width + x
    }

    pub fn coords_of(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    /// The cell at `(x, y)` moved by `(dx, dy)`, or `None` if that falls off the grid.
    pub fn offset(&self, x: usize, y: usize, dx: isize, dy: isize) -> Option<(usize, usize)> {
        let nx = x.checked_add_signed(dx)?;
        let ny = y.checked_add_signed(dy)?;
        (nx < self.width && ny < self.height).then_some((nx, ny))
    }

    pub fn get(&self, x: usize, y: usize) -> T {
        self.cells[self.index_of(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        let i = self.index_of(x, y);
        self.cells[i] = value;
    }

    pub fn cells(&self) -> &[T] {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut [T] {
        &mut self.cells
    }

    /// A copy of this grid with `margin` cells of `fill` added on every side.
    pub fn padded(&self, margin: usize, fill: T) -> Self {
        let mut out = Self::new(self.width + 2 * margin, self.height + 2 * margin, fill);
        for y in 0..self.height {
            let src = &self.cells[y * self.width..(y + 1) * self.width];
            let start = out.index_of(margin, y + margin);
            out.cells[start..start + self.width].copy_from_slice(src);
        }
        out
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.cells[i]
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.cells[i]
    }
}

/// The cells on the straight line from `a` to `b`, both included (Bresenham's algorithm).
///
/// The pointer can cross many cells between two frames, so painting only the cell under it
/// each frame leaves a dotted trail; painting this line instead keeps strokes solid.
pub fn line(a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
    let (mut x, mut y) = (a.0 as isize, a.1 as isize);
    let (x1, y1) = (b.0 as isize, b.1 as isize);
    let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
    let (sx, sy) = ((x1 - x).signum(), (y1 - y).signum());
    let mut err = dx + dy;
    let mut cells = Vec::with_capacity((dx - dy) as usize + 1);
    loop {
        cells.push((x as usize, y as usize));
        if x == x1 && y == y1 {
            return cells;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `offset` is how callers find neighbours safely. It must refuse to step off any of the
    /// four edges, including the low edges, where a careless `usize` subtraction would wrap
    /// to a huge index instead of failing.
    #[test]
    fn offset_stays_in_bounds() {
        let g = Grid::new(3, 2, 0u8);
        assert_eq!(g.offset(0, 0, -1, 0), None);
        assert_eq!(g.offset(2, 1, 1, 0), None);
        assert_eq!(g.offset(2, 1, 0, 1), None);
        assert_eq!(g.offset(1, 0, 1, 1), Some((2, 1)));
    }

    /// Growing a sandpile's grid relies on `padded` putting every cell exactly `margin`
    /// cells right and down, with nothing lost. A row-copy off by one would shear the pile.
    #[test]
    fn padded_keeps_contents_centred() {
        let mut g = Grid::new(2, 1, 0u8);
        g.set(0, 0, 7);
        g.set(1, 0, 9);
        let p = g.padded(2, 0);
        assert_eq!((p.width(), p.height()), (6, 5));
        assert_eq!(p.get(2, 2), 7);
        assert_eq!(p.get(3, 2), 9);
        assert_eq!(p.cells().iter().map(|&c| c as u32).sum::<u32>(), 16);
    }

    /// A stroke must be gap-free: every step moves to one of the 8 neighbours, and it starts
    /// and ends exactly on the two cells given, whichever direction it runs.
    #[test]
    fn line_is_connected_and_hits_both_ends() {
        for (a, b) in [
            ((0, 0), (7, 3)),
            ((7, 3), (0, 0)),
            ((2, 9), (2, 1)),
            ((5, 5), (5, 5)),
        ] {
            let cells = line(a, b);
            assert_eq!((cells[0], *cells.last().unwrap()), (a, b));
            for w in cells.windows(2) {
                let (dx, dy) = (w[0].0.abs_diff(w[1].0), w[0].1.abs_diff(w[1].1));
                assert!(
                    dx <= 1 && dy <= 1 && dx + dy > 0,
                    "gap or repeat in {cells:?}"
                );
            }
        }
    }
}
