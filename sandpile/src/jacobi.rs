//! The fast stages of a pile: Jacobi sweeps in 16 bits.
//!
//! A raster sweep that topples cells in place is a chain: each cell adds grains to the next
//! one, which is read straight back, so the processor can't work ahead. A Jacobi sweep
//! instead computes every cell's new height from the *previous* heights into a second
//! buffer; then the cells are independent, and with 16-bit heights the compiler processes
//! many at once (SIMD). Measured on a near-final pile: 2.5 ns a cell in place, 0.43 ns with
//! Jacobi in 32 bits, 0.10 ns in 16 bits. Jacobi needs about twice the sweeps, since grains
//! no longer run ahead within a sweep, but that still leaves it far ahead.
//!
//! Toppling every unstable cell at once is a legal order of topplings, so by the abelian
//! property the final pile is the same.
//!
//! Two shapes share the same passes ([`pass`]):
//! - [`Quarter`], for a symmetric pile (every grain dropped on the centre of a growing square
//!   grid): one quarter of it, with *ghost* cells mirroring its neighbours across the axes.
//! - [`Patch`], for any other pile: a rectangle of the grid holding every grain.
//!
//! Heights must stay below 65,536: [`LIMIT`] guarantees that.

use crate::sim::Bounds as Rect;
use ca_core::Grid;

/// Enter a 16-bit stage only once every height is below this, and add grains to one only
/// while the cell stays below it. A sweep never pushes the largest height above
/// `N * floor(max / N) + N - 1`, so heights stay far below `u16::MAX`.
pub const LIMIT: u32 = 16_000;

/// One quarter of a symmetric pile.
pub struct Quarter {
    /// Heights now, and the buffer the next sweep writes. Index `(j + 1) * side + (i + 1)`
    /// for offset `(i, j)`; index 0 along either axis is the ghost at offset -1, and the last
    /// is padding beyond the grid, always 0.
    now: Vec<u16>,
    next: Vec<u16>,
    side: usize,
    /// Offsets `0..q` are real cells.
    q: usize,
    /// The largest offset holding grains along either axis.
    extent: usize,
    /// 4 or 8.
    neighbours: usize,
}

impl Quarter {
    /// Copy the quarter `x, y >= c` of `grid`, centred at `(c, c)`, out to offset `extent`.
    /// Every height there must be below [`LIMIT`].
    pub fn from_grid(grid: &Grid<u32>, extent: usize, neighbours: usize) -> Self {
        assert!(
            matches!(neighbours, 4 | 8),
            "a quarter has 4 or 8 neighbours"
        );
        let c = grid.width() / 2;
        let q = c + 1;
        let side = q + 2;
        let mut now = vec![0u16; side * side];
        for j in 0..=extent.min(c) {
            for i in 0..=extent.min(c) {
                let h = grid.get(c + i, c + j);
                debug_assert!(h < LIMIT);
                now[(j + 1) * side + i + 1] = h as u16;
            }
        }
        let mut quarter = Self {
            next: now.clone(),
            now,
            side,
            q,
            extent,
            neighbours,
        };
        quarter.refresh_ghosts(extent + 2);
        quarter
    }

    /// The largest offset holding grains.
    pub fn extent(&self) -> usize {
        self.extent
    }

    /// Add `n` grains to the centre, if it stays below [`LIMIT`]. Returns its new height.
    pub fn add_to_centre(&mut self, n: u32) -> Option<u32> {
        let cell = &mut self.now[self.side + 1];
        let h = u32::from(*cell) + n;
        (h < LIMIT).then(|| {
            *cell = h as u16;
            h
        })
    }

    /// Topple every unstable cell once, all at the same time. Returns the toppling events,
    /// counted over each cell's mirror images (0 means the pile is stable).
    pub fn sweep(&mut self) -> u64 {
        // One cell beyond the grains: they spread at most one cell a sweep.
        let e = (self.extent + 1).min(self.q - 1);
        let (s, t) = (self.side, self.neighbours as u16);
        // A cell stands for 4 mirror images, one on an axis for 2, the centre for 1: count
        // every toppling 4 times, then take back the extra ones on the axes. The x axis is
        // the first row, the y axis the first column, and the centre is on both.
        let unstable = |h: &u16| *h >= t;
        let on_x_axis = self.now[s + 1..=s + e + 1]
            .iter()
            .filter(|h| unstable(h))
            .count();
        let on_y_axis = (1..=e + 1)
            .filter(|&y| unstable(&self.now[y * s + 1]))
            .count();
        let centre = u64::from(unstable(&self.now[s + 1]));
        let r = Rect {
            x0: 1,
            y0: 1,
            x1: e + 1,
            y1: e + 1,
        };
        // Never hex, so the scratch rows `pass` wants are never used.
        let scratch = &mut Scratch::default();
        let all = pass(&self.now, &mut self.next, s, r, self.neighbours, scratch);
        let toppled = 4 * all - 2 * (on_x_axis + on_y_axis) as u64 + centre;
        std::mem::swap(&mut self.now, &mut self.next);
        self.refresh_ghosts(e + 2);
        if e > self.extent
            && (0..=e)
                .any(|m| self.now[(e + 1) * s + m + 1] != 0 || self.now[(m + 1) * s + e + 1] != 0)
        {
            self.extent = e;
        }
        toppled
    }

    /// Set the ghost row and column (offset -1) to mirror offset 1, out to buffer index `upto`.
    fn refresh_ghosts(&mut self, upto: usize) {
        let s = self.side;
        let upto = upto.min(s - 1);
        for y in 0..=upto {
            self.now[y * s] = self.now[y * s + 2];
        }
        let (ghost, rest) = self.now.split_at_mut(s);
        ghost[..=upto].copy_from_slice(&rest[s..s + upto + 1]);
    }

    /// Write the quarter back into all four quarters of `grid` (centred at `(c, c)`), out to
    /// one beyond the extent.
    pub fn write_back(&self, grid: &mut Grid<u32>) {
        let c = grid.width() / 2;
        let reach = (self.extent + 1).min(c);
        for j in 0..=reach {
            for i in 0..=reach {
                let h = u32::from(self.now[(j + 1) * self.side + i + 1]);
                for (x, y) in [
                    (c + i, c + j),
                    (c - i, c + j),
                    (c + i, c - j),
                    (c - i, c - j),
                ] {
                    grid.set(x, y, h);
                }
            }
        }
    }
}

/// A rectangle of the grid holding every grain of a pile, swept without symmetry.
///
/// Its outermost ring of cells is never toppled and never receives grains: grains sent
/// there vanish. That is right where the ring lies on the grid's frame and the pile drains
/// (a *sink* side); anywhere else the pile must leave this stage before grains get near the
/// ring, which [`Patch::needs_room`] says.
pub struct Patch {
    /// Heights now, and the buffer the next sweep writes, row by row.
    now: Vec<u16>,
    next: Vec<u16>,
    /// Where the patch lies on the grid: cell `(x, y)` of the grid is index
    /// `(y - at.y0) * width + (x - at.x0)` here. The ring included.
    at: Rect,
    width: usize,
    /// Every grain lies in this box, in patch coordinates (0 is the ring). It only grows.
    active: Rect,
    /// Whether grains may leave over each side: left, right, top, bottom.
    sinks: [bool; 4],
    neighbours: usize,
    scratch: Scratch,
}

impl Patch {
    /// Copy the cells of `grid` inside `active` into a patch covering `at`, which must hold
    /// `active` with at least one cell to spare on every side. Every height in `active` must
    /// be below [`LIMIT`], and every cell of `at` outside it must be empty.
    pub fn from_grid(
        grid: &Grid<u32>,
        at: Rect,
        active: Rect,
        neighbours: usize,
        sinks: [bool; 4],
    ) -> Self {
        assert!(
            matches!(neighbours, 4 | 6 | 8),
            "no patch for {neighbours} neighbours"
        );
        assert!(
            at.x0 < active.x0 && at.y0 < active.y0 && active.x1 < at.x1 && active.y1 < at.y1,
            "{active:?} needs room inside {at:?}"
        );
        let width = at.x1 + 1 - at.x0;
        let mut now = vec![0u16; at.area()];
        for y in active.y0..=active.y1 {
            let row = &grid.cells()[grid.index_of(active.x0, y)..=grid.index_of(active.x1, y)];
            let start = (y - at.y0) * width + active.x0 - at.x0;
            for (cell, &h) in now[start..start + row.len()].iter_mut().zip(row) {
                debug_assert!(h < LIMIT);
                *cell = h as u16;
            }
        }
        let active = Rect {
            x0: active.x0 - at.x0,
            y0: active.y0 - at.y0,
            x1: active.x1 - at.x0,
            y1: active.y1 - at.y0,
        };
        Self {
            next: now.clone(),
            now,
            at,
            width,
            active,
            sinks,
            neighbours,
            scratch: Scratch::default(),
        }
    }

    /// The last row and column inside the ring.
    fn inner_end(&self) -> (usize, usize) {
        (self.width - 2, self.at.y1 - self.at.y0 - 1)
    }

    /// The cells the next sweep visits: one beyond the grains, which spread at most one cell
    /// a sweep, but never the ring.
    fn region(&self) -> Rect {
        let (xe, ye) = self.inner_end();
        let a = self.active;
        Rect {
            x0: (a.x0 - 1).max(1),
            y0: (a.y0 - 1).max(1),
            x1: (a.x1 + 1).min(xe),
            y1: (a.y1 + 1).min(ye),
        }
    }

    /// Cells one sweep visits.
    pub fn area(&self) -> usize {
        self.region().area()
    }

    /// Whether the next sweep could send grains into the ring on a side that isn't a sink,
    /// where they must not be lost: the pile must then leave this stage and make room.
    pub fn needs_room(&self) -> bool {
        let (xe, ye) = self.inner_end();
        let a = self.active;
        let touches = [a.x0 <= 1, a.x1 >= xe, a.y0 <= 1, a.y1 >= ye];
        touches.iter().zip(self.sinks).any(|(&t, sink)| t && !sink)
    }

    /// Add `n` grains to grid cell `(x, y)`, if it lies inside the ring and stays below
    /// [`LIMIT`]. Returns its new height.
    pub fn add(&mut self, x: usize, y: usize, n: u32) -> Option<u32> {
        let (xe, ye) = self.inner_end();
        let (px, py) = (x.checked_sub(self.at.x0)?, y.checked_sub(self.at.y0)?);
        if !(1..=xe).contains(&px) || !(1..=ye).contains(&py) {
            return None;
        }
        let cell = &mut self.now[py * self.width + px];
        let h = u32::from(*cell) + n;
        if h >= LIMIT {
            return None;
        }
        *cell = h as u16;
        let a = &mut self.active;
        (a.x0, a.y0, a.x1, a.y1) = (a.x0.min(px), a.y0.min(py), a.x1.max(px), a.y1.max(py));
        Some(h)
    }

    /// Topple every unstable cell once, all at the same time. Returns how many toppled.
    pub fn sweep(&mut self) -> u64 {
        let r = self.region();
        let w = self.width;
        let toppled = pass(
            &self.now,
            &mut self.next,
            w,
            r,
            self.neighbours,
            &mut self.scratch,
        );
        std::mem::swap(&mut self.now, &mut self.next);
        // Every grain now lies in the region; grow the box to each of its edges that holds
        // one. (Cells outside the region are 0 in both buffers: the region never shrinks.)
        let a = &mut self.active;
        let busy_row = |y: usize| {
            self.now[y * w + r.x0..=y * w + r.x1]
                .iter()
                .any(|&h| h != 0)
        };
        let busy_col = |x: usize| (r.y0..=r.y1).any(|y| self.now[y * w + x] != 0);
        if r.y0 < a.y0 && busy_row(r.y0) {
            a.y0 = r.y0;
        }
        if r.y1 > a.y1 && busy_row(r.y1) {
            a.y1 = r.y1;
        }
        if r.x0 < a.x0 && busy_col(r.x0) {
            a.x0 = r.x0;
        }
        if r.x1 > a.x1 && busy_col(r.x1) {
            a.x1 = r.x1;
        }
        toppled
    }

    /// The box holding every grain, in grid coordinates.
    pub fn occupied(&self) -> Rect {
        let a = self.active;
        Rect {
            x0: a.x0 + self.at.x0,
            y0: a.y0 + self.at.y0,
            x1: a.x1 + self.at.x0,
            y1: a.y1 + self.at.y0,
        }
    }

    /// Write the heights back into `grid`. Returns the grains in the patch.
    pub fn write_back(&self, grid: &mut Grid<u32>) -> u64 {
        let (a, w) = (self.active, self.width);
        let mut grains = 0u64;
        for y in a.y0..=a.y1 {
            let gy = y + self.at.y0;
            let start = grid.index_of(a.x0 + self.at.x0, gy);
            let out = &mut grid.cells_mut()[start..start + a.x1 + 1 - a.x0];
            for (cell, &h) in out.iter_mut().zip(&self.now[y * w + a.x0..=y * w + a.x1]) {
                *cell = u32::from(h);
                grains += u64::from(h);
            }
        }
        grains
    }
}

/// One Jacobi pass: topple, all at once, every cell of the rectangle `r` of `now`, a grid
/// `width` cells wide, writing the new heights to the same cells of `next`. The cells
/// around `r` are read as neighbours but not written. Returns how many cells toppled.
///
/// `neighbours` is 4 (orthogonal), 8 (with the diagonals), or 6 (a hex grid in axial
/// coordinates, `sim::HEX`: up-and-right and down-and-left as well as the orthogonal 4).
/// Each is its own mirror image (with each offset, its negative), so a cell receives one
/// grain from each neighbour for each time that neighbour topples.
fn pass(
    now: &[u16],
    next: &mut [u16],
    width: usize,
    r: Rect,
    neighbours: usize,
    scratch: &mut Scratch,
) -> u64 {
    match neighbours {
        4 => pass_shift::<4>(now, next, width, r),
        6 => pass_hex(now, next, width, r, scratch),
        8 => pass_shift::<8>(now, next, width, r),
        n => unreachable!("no 16-bit stage for {n} neighbours"),
    }
}

/// [`pass`] for 4 or 8 neighbours, where a cell's topplings, `height / N`, are a shift:
/// cheap enough to work out again for each neighbour that reads them. (Working them out
/// once, as [`pass_hex`] does, measured 15% slower here.)
fn pass_shift<const N: usize>(now: &[u16], next: &mut [u16], w: usize, r: Rect) -> u64 {
    let t = N as u16;
    let mut toppled = 0u64;
    for y in r.y0..=r.y1 {
        let row = |y: usize| &now[y * w + r.x0 - 1..=y * w + r.x1 + 1];
        let (up, mid, dn) = (row(y - 1), row(y), row(y + 1));
        let out = &mut next[y * w + r.x0..=y * w + r.x1];
        // `out[x - 1]` is the cell at `mid[x]`. Slicing all four to matching lengths up
        // front lets the compiler drop the bounds checks and vectorise the loop.
        let n = out.len();
        let (up, mid, dn) = (&up[..n + 2], &mid[..n + 2], &dn[..n + 2]);
        // Written with an explicit shift and mask, indexed from 1, and counted per row: the
        // same arithmetic written as `v % t`, `v / t` and `mid[i + 1]` measured 4% slower
        // for a whole centred pile (6.7 s against 6.4 s), and the off-centre one 4% too.
        let (shift, mask) = (N.trailing_zeros(), t - 1);
        let mut row = 0u64;
        for x in 1..=n {
            let v = mid[x];
            row += (v >> shift != 0) as u64;
            let mut h = (v & mask) + (mid[x - 1] >> shift) + (mid[x + 1] >> shift);
            h += (up[x] >> shift) + (dn[x] >> shift);
            if N == 8 {
                h += (up[x - 1] >> shift) + (up[x + 1] >> shift);
                h += (dn[x - 1] >> shift) + (dn[x + 1] >> shift);
            }
            out[x - 1] = h;
        }
        toppled += row;
    }
    toppled
}

/// [`pass`] on a hex grid. Dividing by 6 costs a multiplication, so each cell's topplings
/// are worked out once, into three rolling rows of [`Scratch`], not once for each of the 7
/// cells that read them: 0.43 ns a cell against 0.69 ns, measured natively.
fn pass_hex(now: &[u16], next: &mut [u16], w: usize, r: Rect, scratch: &mut Scratch) -> u64 {
    const N: usize = 6;
    let t = N as u16;
    // Topplings are needed one cell beyond `r` on every side.
    let (x0, x1) = (r.x0 - 1, r.x1 + 1);
    let topplings = |k: &mut Vec<u16>, y: usize| {
        k.clear();
        k.extend(now[y * w + x0..=y * w + x1].iter().map(|&h| h / t));
    };
    let [up, mid, dn] = &mut scratch.0;
    topplings(up, r.y0 - 1);
    topplings(mid, r.y0);
    let mut toppled = 0u64;
    for y in r.y0..=r.y1 {
        topplings(dn, y + 1);
        let heights = &now[y * w + r.x0..=y * w + r.x1];
        let out = &mut next[y * w + r.x0..=y * w + r.x1];
        // `out[i]` is the cell whose topplings are `mid[i + 1]`.
        let n = out.len();
        let (u, m, d) = (&up[..n + 2], &mid[..n + 2], &dn[..n + 2]);
        for i in 0..n {
            let v = heights[i];
            toppled += (v >= t) as u64;
            out[i] = v - m[i + 1] * t + m[i] + m[i + 2] + u[i + 1] + d[i + 1] + u[i + 2] + d[i];
        }
        std::mem::swap(up, mid);
        std::mem::swap(mid, dn);
    }
    toppled
}

/// Three rows of each cell's topplings, `height / 6`, reused from sweep to sweep by
/// [`pass_hex`], so a sweep allocates nothing.
#[derive(Default)]
pub struct Scratch([Vec<u16>; 3]);

#[cfg(test)]
mod tests {
    use super::*;

    /// The ghost cells at the origin, checked directly: 4 grains at the centre topple once
    /// into a plus, and writing back fills all four quarters. A mistake in the ghosts (the
    /// centre seeing its own mirror image, say) would show here before anywhere else.
    #[test]
    fn four_grains_make_a_plus() {
        let mut g = Grid::new(11, 11, 0u32);
        g.set(5, 5, 4);
        let mut q = Quarter::from_grid(&g, 0, 4);
        assert_eq!(q.sweep(), 1);
        assert_eq!(q.sweep(), 0, "a plus is stable");
        q.write_back(&mut g);
        assert_eq!(g.get(5, 5), 0);
        for (x, y) in [(4, 5), (6, 5), (5, 4), (5, 6)] {
            assert_eq!(g.get(x, y), 1, "({x}, {y})");
        }
        assert_eq!(g.cells().iter().sum::<u32>(), 4);
    }

    /// With 8 neighbours, 8 grains at the centre must reach all 8 cells around it, the
    /// diagonals through the corner ghost.
    #[test]
    fn eight_grains_reach_all_eight_neighbours() {
        let mut g = Grid::new(11, 11, 0u32);
        g.set(5, 5, 8);
        let mut q = Quarter::from_grid(&g, 0, 8);
        q.sweep();
        q.write_back(&mut g);
        assert_eq!(g.get(5, 5), 0);
        assert_eq!(g.cells().iter().filter(|&&h| h == 1).count(), 8);
    }

    fn rect(x0: usize, y0: usize, x1: usize, y1: usize) -> Rect {
        Rect { x0, y0, x1, y1 }
    }

    /// On a hex grid, 6 grains on one cell must go to exactly its 6 axial neighbours (the
    /// row beside it, and up-and-right and down-and-left), checked by hand; a wrong pair of
    /// diagonals would land grains on the other two cells of the 3 x 3 square instead.
    #[test]
    fn six_grains_reach_the_hex_neighbours() {
        let mut g = Grid::new(9, 9, 0u32);
        g.set(4, 4, 6);
        let at = rect(0, 0, 8, 8);
        let mut p = Patch::from_grid(&g, at, rect(4, 4, 4, 4), 6, [false; 4]);
        assert_eq!(p.sweep(), 1);
        assert_eq!(p.sweep(), 0, "one grain each is stable");
        assert_eq!(p.write_back(&mut g), 6);
        assert_eq!(g.get(4, 4), 0);
        for (x, y) in [(5, 4), (3, 4), (4, 5), (4, 3), (5, 3), (3, 5)] {
            assert_eq!(g.get(x, y), 1, "({x}, {y})");
        }
        assert_eq!(p.occupied(), rect(3, 3, 5, 5));
    }

    /// Grains toppled into the ring vanish, so a patch whose grains reach its ring must say
    /// it needs room, unless that side is a sink; and a sink side must really lose them.
    #[test]
    fn the_ring_loses_grains_only_over_a_sink() {
        let g = {
            let mut g = Grid::new(5, 5, 0u32);
            g.set(1, 2, 4);
            g
        };
        let at = rect(0, 0, 4, 4);
        let closed = Patch::from_grid(&g, at, rect(1, 2, 1, 2), 4, [false; 4]);
        assert!(
            closed.needs_room(),
            "the next sweep would reach the left ring"
        );
        let mut open = Patch::from_grid(&g, at, rect(1, 2, 1, 2), 4, [true, false, false, false]);
        assert!(!open.needs_room());
        open.sweep();
        let mut g = g;
        assert_eq!(
            open.write_back(&mut g),
            3,
            "one grain fell off the left side"
        );
    }
}
