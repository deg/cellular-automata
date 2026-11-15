//! The fast stage for a symmetric pile: Jacobi sweeps over one quarter of it, in 16 bits.
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
//! The quarter holds offsets `0..q` from the centre along each axis. A row and a column of
//! *ghost* cells at offset -1 mirror offset 1, which is what the cells on the axes see as
//! their neighbours across it. Heights must stay below 65,536: [`LIMIT`] guarantees that.

use ca_core::Grid;

/// Enter the 16-bit stage only once every height is below this. A sweep never pushes the
/// largest height above `N * floor(max / N) + N - 1`, so heights stay far below `u16::MAX`.
pub const LIMIT: u32 = 16_000;

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

    /// Topple every unstable cell once, all at the same time. Returns the toppling events,
    /// counted over each cell's mirror images (0 means the pile is stable).
    pub fn sweep(&mut self) -> u64 {
        // One cell beyond the grains: they spread at most one cell a sweep.
        let e = (self.extent + 1).min(self.q - 1);
        let toppled = match self.neighbours {
            4 => self.pass::<4>(e),
            8 => self.pass::<8>(e),
            n => unreachable!("a {n}-neighbour pile can't be symmetric"),
        };
        std::mem::swap(&mut self.now, &mut self.next);
        self.refresh_ghosts(e + 2);
        let s = self.side;
        if e > self.extent
            && (0..=e)
                .any(|m| self.now[(e + 1) * s + m + 1] != 0 || self.now[(m + 1) * s + e + 1] != 0)
        {
            self.extent = e;
        }
        toppled
    }

    /// One Jacobi pass over offsets `0..=e`, for `N` neighbours (threshold `N`).
    fn pass<const N: usize>(&mut self, e: usize) -> u64 {
        let shift = N.trailing_zeros();
        let mask = (N - 1) as u16;
        let s = self.side;
        let mut toppled = 0u64;
        for y in 1..=e + 1 {
            let (up, mid, dn) = (
                &self.now[(y - 1) * s..y * s],
                &self.now[y * s..(y + 1) * s],
                &self.now[(y + 1) * s..(y + 2) * s],
            );
            let out = &mut self.next[y * s..(y + 1) * s];
            let mut row = 0u64;
            for x in 1..=e + 1 {
                let v = mid[x];
                row += (v >> shift != 0) as u64;
                let mut h = (v & mask) + (mid[x - 1] >> shift) + (mid[x + 1] >> shift);
                h += (up[x] >> shift) + (dn[x] >> shift);
                if N == 8 {
                    h += (up[x - 1] >> shift) + (up[x + 1] >> shift);
                    h += (dn[x - 1] >> shift) + (dn[x + 1] >> shift);
                }
                out[x] = h;
            }
            // A cell stands for 4 mirror images, one on an axis for 2, the centre for 1.
            let on_axis = (mid[1] >> shift != 0) as u64;
            toppled += if y == 1 {
                2 * row - on_axis
            } else {
                4 * row - 2 * on_axis
            };
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
}
