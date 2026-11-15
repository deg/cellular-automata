//! The Abelian sandpile (Bak, Tang and Wiesenfeld, 1987).
//!
//! Each cell holds a number of grains. A cell holding at least `threshold` grains is
//! unstable: it topples, sending one grain to each of its neighbours. The threshold equals
//! the number of neighbours, so a toppling conserves grains (except over a sink edge).
//!
//! The final stable pile does not depend on the order of topplings (the "abelian" property),
//! so we are free to topple in whatever order is fastest, and to topple a cell holding
//! `k * threshold` grains `k` times at once. Even the total number of topplings is fixed,
//! so speed comes only from making each one cheap.
//!
//! We relax by sweeping, in raster order and in place, over the box that holds every grain.
//! Measured against a worklist of unstable cells, sweeping was 2 to 3 times faster in a
//! native release build (100,000 grains: 1.1 s against 2.2 s; 1,000,000: 102 s against 307 s).

use ca_core::Grid;

/// The four orthogonal neighbours (von Neumann neighbourhood).
pub const ORTHOGONAL: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// All eight neighbours, diagonals included (Moore neighbourhood). The threshold is then 8.
pub const MOORE: [(isize, isize); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// Grids never grow past this many cells on a side: bigger textures fail in many browsers.
pub const MAX_SIDE: usize = 4001;

/// What happens to grains that topple off the edge of the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    /// The grid grows (up to [`MAX_SIDE`]), so no grain is ever lost.
    Grow,
    /// The grid keeps its size and grains falling off the edge are lost.
    Sink,
}

/// An inclusive rectangle of cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bounds {
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
}

impl Bounds {
    fn cell(x: usize, y: usize) -> Self {
        Self {
            x0: x,
            y0: y,
            x1: x,
            y1: y,
        }
    }

    fn include(&mut self, x: usize, y: usize) {
        self.x0 = self.x0.min(x);
        self.y0 = self.y0.min(y);
        self.x1 = self.x1.max(x);
        self.y1 = self.y1.max(y);
    }

    fn shifted(self, by: usize) -> Self {
        Self {
            x0: self.x0 + by,
            y0: self.y0 + by,
            x1: self.x1 + by,
            y1: self.y1 + by,
        }
    }
}

/// A sandpile on a square grid.
///
/// The grid carries a frame `reach` cells wide that is never toppled. Grains that land in
/// it have left the pile: [`Boundary::Sink`] removes them after each sweep, and
/// [`Boundary::Grow`] grows the grid before any grain can reach it.
pub struct Sandpile {
    grid: Grid<u32>,
    /// Each neighbour as an `(dx, dy)` offset, and as a difference of indices into `grid`.
    offsets: Vec<(isize, isize)>,
    deltas: Vec<isize>,
    threshold: u32,
    reach: usize,
    boundary: Boundary,
    /// Every grain lies inside this box, or nothing does when it is `None`.
    occupied: Option<Bounds>,
    settled: bool,
    /// Grains currently on the grid.
    pub grains: u64,
    /// Grains that fell off a sink edge (or off the edge of a grid at [`MAX_SIDE`]).
    pub lost: u64,
    /// Toppling events so far; a cell toppling `k` times at once counts once.
    pub topples: u64,
    /// Sweeps so far.
    pub sweeps: u64,
}

impl Sandpile {
    /// An empty `side` x `side` pile.
    ///
    /// `side` must be odd, so the grid has a true centre cell and growing it lands exactly on
    /// [`MAX_SIDE`] (also odd). An even side would stop growing one short of `MAX_SIDE` and
    /// never start draining, so grains would collect unseen in the frame.
    pub fn new(side: usize, offsets: &[(isize, isize)], boundary: Boundary) -> Self {
        assert!(side % 2 == 1, "a sandpile's side must be odd, not {side}");
        assert!(
            !offsets.is_empty(),
            "a sandpile needs at least one neighbour"
        );
        let reach = offsets
            .iter()
            .map(|&(dx, dy)| dx.unsigned_abs().max(dy.unsigned_abs()))
            .max()
            .unwrap_or(1);
        assert!(
            side > 2 * reach,
            "a {side}-cell grid has no room inside its frame"
        );
        let mut pile = Self {
            grid: Grid::new(side, side, 0),
            offsets: offsets.to_vec(),
            deltas: Vec::new(),
            threshold: offsets.len() as u32,
            reach,
            boundary,
            occupied: None,
            settled: true,
            grains: 0,
            lost: 0,
            topples: 0,
            sweeps: 0,
        };
        pile.compute_deltas();
        pile
    }

    pub fn grid(&self) -> &Grid<u32> {
        &self.grid
    }

    pub fn threshold(&self) -> u32 {
        self.threshold
    }

    pub fn boundary(&self) -> Boundary {
        self.boundary
    }

    pub fn center(&self) -> (usize, usize) {
        (self.grid.width() / 2, self.grid.height() / 2)
    }

    /// True once a sweep has found nothing to topple and no grain has been added since.
    pub fn is_stable(&self) -> bool {
        self.settled
    }

    /// Cells one sweep visits: the area of the box that holds every grain.
    pub fn sweep_area(&self) -> usize {
        self.occupied.map_or(0, |b| {
            let b = self.clamp_inside_frame(b);
            (b.x1 + 1 - b.x0) * (b.y1 + 1 - b.y0)
        })
    }

    /// Whether `n` more grains fit: the pile holds at most `u32::MAX` grains in all.
    ///
    /// Bounding the total, not each cell, is what keeps the counts safe: no cell can ever
    /// hold more than the whole pile, so no toppling in [`Sandpile::sweep`] can overflow.
    pub fn can_add(&self, n: u32) -> bool {
        self.grains + u64::from(n) <= u64::from(u32::MAX)
    }

    /// Drop `n` grains on cell `(x, y)`. They topple only when [`Sandpile::sweep`] runs.
    /// A drop on the frame grows the grid first, or is lost over a sink edge.
    /// Does nothing, and returns false, if the grains don't fit (see [`Sandpile::can_add`]).
    pub fn add(&mut self, mut x: usize, mut y: usize, n: u32) -> bool {
        if !self.can_add(n) {
            return false;
        }
        if self.in_frame(x, y) {
            let grown = if self.boundary == Boundary::Grow {
                self.grow()
            } else {
                None
            };
            let Some(margin) = grown else {
                self.lost += u64::from(n);
                return true;
            };
            x += margin;
            y += margin;
        }
        let i = self.grid.index_of(x, y);
        self.grid[i] += n; // can't overflow: see `can_add`
        self.grains += u64::from(n);
        match &mut self.occupied {
            Some(b) => b.include(x, y),
            None => self.occupied = Some(Bounds::cell(x, y)),
        }
        self.settled = false;
        true
    }

    /// Drop `n` grains on the cell `(dx, dy)` away from the centre.
    ///
    /// The grid grows evenly on every side, so the centre stays the centre and a position
    /// relative to it stays valid however much the grid grows, or across a Restart. A
    /// growing pile grows until the cell fits; on a sink pile a cell off the grid has already
    /// fallen off, so its grains count as lost. Returns false only if the grains don't fit
    /// (see [`Sandpile::can_add`]).
    pub fn add_relative(&mut self, dx: isize, dy: isize, n: u32) -> bool {
        if !self.can_add(n) {
            return false;
        }
        loop {
            let (cx, cy) = self.center();
            if let Some((x, y)) = self.grid.offset(cx, cy, dx, dy)
                && !self.in_frame(x, y)
            {
                return self.add(x, y, n);
            }
            if self.boundary == Boundary::Sink || self.grow().is_none() {
                self.lost += u64::from(n);
                return true;
            }
        }
    }

    /// Whether the cell `(dx, dy)` from the centre is on the grid, inside the frame, so that
    /// grains dropped there stay on the pile. On a sink pile, a drip point where this is
    /// false loses every grain dropped on it.
    pub fn holds_relative(&self, dx: isize, dy: isize) -> bool {
        let (cx, cy) = self.center();
        self.grid
            .offset(cx, cy, dx, dy)
            .is_some_and(|(x, y)| !self.in_frame(x, y))
    }

    /// Topple every unstable cell once, in raster order. Grains passed forward in the
    /// sweep can topple their new cell in the same sweep. Returns whether anything toppled.
    pub fn sweep(&mut self) -> bool {
        if self.settled {
            return false;
        }
        let Some(mut b) = self.occupied else {
            self.settled = true;
            return false;
        };
        if self.boundary == Boundary::Grow && self.touches_frame(b) && self.grow().is_some() {
            b = self.occupied.expect("growing keeps the occupied box");
        }
        let b = self.clamp_inside_frame(b);

        let threshold = self.threshold;
        let width = self.grid.width();
        let deltas = &self.deltas;
        let cells = self.grid.cells_mut();
        let mut toppled: Option<Bounds> = None;
        let mut topples = 0;
        for y in b.y0..=b.y1 {
            for x in b.x0..=b.x1 {
                let i = y * width + x;
                let height = cells[i];
                if height < threshold {
                    continue;
                }
                let k = height / threshold;
                cells[i] = height % threshold;
                for &d in deltas {
                    // In bounds: the box lies inside the frame, which is `reach` cells wide.
                    cells[i.wrapping_add_signed(d)] += k;
                }
                topples += 1;
                match &mut toppled {
                    Some(t) => t.include(x, y),
                    None => toppled = Some(Bounds::cell(x, y)),
                }
            }
        }
        self.topples += topples;
        self.sweeps += 1;

        let Some(t) = toppled else {
            self.settled = true;
            return false;
        };
        // Grains reach at most `reach` cells beyond any cell that toppled.
        let r = self.reach;
        let mut occupied = self.occupied.expect("checked above");
        occupied.include(t.x0 - r, t.y0 - r);
        occupied.include(t.x1 + r, t.y1 + r);
        self.occupied = Some(occupied);
        let draining = self.boundary == Boundary::Sink || self.grid.width() >= MAX_SIDE;
        if draining && self.touches_frame(t) {
            self.drain_frame();
        }
        true
    }

    /// Sweep until stable or until `max_sweeps` have run. Returns the sweeps that toppled.
    pub fn relax(&mut self, max_sweeps: usize) -> usize {
        let mut done = 0;
        while done < max_sweeps && self.sweep() {
            done += 1;
        }
        done
    }

    /// Relax until stable. Only for tests and small piles: it can take a long time.
    #[cfg(test)]
    pub fn relax_fully(&mut self) {
        while self.sweep() {}
    }

    fn compute_deltas(&mut self) {
        let w = self.grid.width() as isize;
        self.deltas = self.offsets.iter().map(|&(dx, dy)| dy * w + dx).collect();
    }

    fn in_frame(&self, x: usize, y: usize) -> bool {
        let (r, w, h) = (self.reach, self.grid.width(), self.grid.height());
        x < r || y < r || x + r >= w || y + r >= h
    }

    /// Whether a cell toppling inside `b` could send grains into the frame.
    fn touches_frame(&self, b: Bounds) -> bool {
        let r = 2 * self.reach;
        b.x0 < r || b.y0 < r || b.x1 + r >= self.grid.width() || b.y1 + r >= self.grid.height()
    }

    fn clamp_inside_frame(&self, b: Bounds) -> Bounds {
        let r = self.reach;
        Bounds {
            x0: b.x0.max(r),
            y0: b.y0.max(r),
            x1: b.x1.min(self.grid.width() - 1 - r),
            y1: b.y1.min(self.grid.height() - 1 - r),
        }
    }

    /// Remove the grains in the frame, counting them as lost.
    fn drain_frame(&mut self) {
        let (r, w, h) = (self.reach, self.grid.width(), self.grid.height());
        let mut drained = 0u64;
        let mut take = |grid: &mut Grid<u32>, x: usize, y: usize| {
            let i = grid.index_of(x, y);
            drained += u64::from(std::mem::take(&mut grid[i]));
        };
        for y in 0..h {
            if y < r || y + r >= h {
                (0..w).for_each(|x| take(&mut self.grid, x, y));
            } else {
                (0..r)
                    .chain(w - r..w)
                    .for_each(|x| take(&mut self.grid, x, y));
            }
        }
        self.lost += drained;
        self.grains -= drained;
    }

    /// Pad the grid on every side, keeping it odd-sized and no larger than [`MAX_SIDE`].
    /// Returns the margin added to every side, or `None` if the grid is at its largest.
    fn grow(&mut self) -> Option<usize> {
        let side = self.grid.width();
        let margin = (side / 2).max(8).min(MAX_SIDE.saturating_sub(side) / 2);
        if margin == 0 {
            return None;
        }
        self.grid = self.grid.padded(margin, 0);
        self.compute_deltas();
        self.occupied = self.occupied.map(|b| b.shifted(margin));
        Some(margin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn centred_pile(grains: u32, boundary: Boundary, side: usize) -> Sandpile {
        let mut pile = Sandpile::new(side, &ORTHOGONAL, boundary);
        let (cx, cy) = pile.center();
        pile.add(cx, cy, grains);
        pile.relax_fully();
        pile
    }

    fn sum(pile: &Sandpile) -> u64 {
        pile.grid().cells().iter().map(|&h| u64::from(h)).sum()
    }

    /// The smallest avalanche, checked by hand: four grains on one cell topple once, leaving
    /// it empty and one grain on each neighbour. Catches wrong neighbour offsets or deltas.
    #[test]
    fn four_grains_make_a_plus() {
        let pile = centred_pile(4, Boundary::Sink, 5);
        let (cx, cy) = pile.center();
        assert_eq!(pile.grid().get(cx, cy), 0);
        for (dx, dy) in ORTHOGONAL {
            let (x, y) = pile.grid().offset(cx, cy, dx, dy).unwrap();
            assert_eq!(pile.grid().get(x, y), 1);
        }
        assert_eq!(sum(&pile), 4);
    }

    /// With a growing edge no grain may ever be lost, and relaxing must end with every cell
    /// below the threshold. Starting from a 3-cell grid forces several growths mid-relax,
    /// which is where a bad index shift would lose or duplicate grains.
    #[test]
    fn growing_pile_conserves_grains_and_settles() {
        let n = 20_000;
        let pile = centred_pile(n, Boundary::Grow, 3);
        assert!(pile.grid().width() > 3, "the grid should have grown");
        assert_eq!(pile.lost, 0);
        assert_eq!(sum(&pile), u64::from(n));
        assert_eq!(pile.grains, u64::from(n));
        assert!(pile.is_stable());
        assert!(pile.grid().cells().iter().all(|&h| h < pile.threshold()));
    }

    /// The rule treats all four directions alike, so a centred drop must give a pile with
    /// the square's mirror and diagonal symmetry. A raster sweep favours no direction in its
    /// final result only if the abelian property truly holds in our implementation.
    #[test]
    fn centred_pile_has_the_symmetry_of_the_square() {
        let pile = centred_pile(10_000, Boundary::Grow, 3);
        let g = pile.grid();
        let n = g.width();
        for y in 0..n {
            for x in 0..n {
                let h = g.get(x, y);
                assert_eq!(h, g.get(n - 1 - x, y), "mirror at ({x}, {y})");
                assert_eq!(h, g.get(y, x), "diagonal at ({x}, {y})");
            }
        }
    }

    /// The abelian property, on which the fast sweep depends: settling after every grain must
    /// give exactly the same pile as dropping them all and settling once.
    #[test]
    fn toppling_order_does_not_matter() {
        // One grain at a time, settling after each, against all grains at once.
        let n = 3_000;
        let all_at_once = centred_pile(n, Boundary::Sink, 81);
        let mut one_by_one = Sandpile::new(81, &ORTHOGONAL, Boundary::Sink);
        let (cx, cy) = one_by_one.center();
        for _ in 0..n {
            one_by_one.add(cx, cy, 1);
            one_by_one.relax_fully();
        }
        assert_eq!(
            all_at_once.lost, 0,
            "grid too small for the test to mean anything"
        );
        assert_eq!(all_at_once.grid(), one_by_one.grid());
    }

    /// On a grid with no room to topple, a sink edge must swallow every grain and the
    /// counters must say so; the grid must not grow.
    #[test]
    fn sink_edge_loses_grains() {
        let pile = centred_pile(100, Boundary::Sink, 3);
        assert_eq!(pile.lost, 100);
        assert_eq!(pile.grains, 0);
        assert_eq!(sum(&pile), 0);
        assert_eq!(pile.grid().width(), 3);
    }

    /// With a sink edge, grains on the grid plus grains lost must equal grains dropped, and
    /// `grains` must agree with the grid. Catches the frame drain double- or under-counting.
    #[test]
    fn sink_pile_accounts_for_every_grain() {
        let n = 5_000;
        let pile = centred_pile(n, Boundary::Sink, 21);
        assert!(pile.lost > 0);
        assert_eq!(pile.lost + sum(&pile), u64::from(n));
        assert_eq!(pile.grains, sum(&pile));
        assert!(pile.grid().cells().iter().all(|&h| h < pile.threshold()));
    }

    /// The UI spreads relaxation across frames by capping sweeps per call; if `relax` ignored
    /// its limit, a big drop would freeze the window until the pile settled.
    #[test]
    fn relax_stops_at_the_sweep_limit() {
        let mut pile = Sandpile::new(101, &ORTHOGONAL, Boundary::Sink);
        let (cx, cy) = pile.center();
        pile.add(cx, cy, 5_000);
        assert_eq!(pile.relax(10), 10);
        assert!(!pile.is_stable());
    }

    /// A click on the outermost cells (the frame) of a growing pile must grow the grid and
    /// land the grain on the same spot, now `margin` cells further in. An off-by-one there
    /// would drop the grain beside the cell clicked, or back into the frame.
    #[test]
    fn grow_pile_takes_a_drop_on_its_frame() {
        let mut pile = Sandpile::new(3, &ORTHOGONAL, Boundary::Grow);
        assert!(pile.add(0, 0, 1));
        let margin = (pile.grid().width() - 3) / 2;
        assert!(margin > 0, "the grid should have grown");
        assert_eq!(pile.grid().get(margin, margin), 1);
        assert_eq!((pile.grains, pile.lost), (1, 0));
    }

    /// On a sink pile, a drop on the frame has already fallen off: it must count as lost and
    /// never reach the grid, where nothing would ever topple it.
    #[test]
    fn sink_pile_loses_a_drop_on_its_frame() {
        let mut pile = Sandpile::new(5, &ORTHOGONAL, Boundary::Sink);
        assert!(pile.add(0, 2, 7));
        assert_eq!((pile.grains, pile.lost), (0, 7));
        assert_eq!(sum(&pile), 0);
    }

    /// The pile refuses grains past `u32::MAX` in total, and that bound is enough: even with
    /// every grain on one cell, relaxing must not overflow any cell (which would wrap
    /// silently in a release build) and must account for every grain.
    #[test]
    fn total_grains_are_capped_and_relaxing_a_full_pile_is_safe() {
        let mut pile = Sandpile::new(9, &ORTHOGONAL, Boundary::Sink);
        let (cx, cy) = pile.center();
        assert!(pile.add(cx, cy, u32::MAX));
        assert!(!pile.can_add(1));
        assert!(!pile.add(cx, cy, 1), "a full pile must refuse more grains");
        assert_eq!(pile.grains, u64::from(u32::MAX));
        pile.relax_fully();
        assert_eq!(pile.lost + sum(&pile), u64::from(u32::MAX));
    }

    /// The per-frame work budget divides by `sweep_area`, so it must be 0 for an empty pile
    /// and grow to cover the cells the pile spreads into.
    #[test]
    fn sweep_area_tracks_the_occupied_box() {
        let mut pile = Sandpile::new(21, &ORTHOGONAL, Boundary::Sink);
        assert_eq!(pile.sweep_area(), 0);
        let (cx, cy) = pile.center();
        pile.add(cx, cy, 4);
        assert_eq!(pile.sweep_area(), 1);
        pile.relax_fully();
        assert_eq!(pile.sweep_area(), 9, "one toppling reaches a 3 x 3 box");
    }

    /// `sweep` skips all work once settled; a new grain must wake it, or drips and clicks
    /// on a settled pile would never topple.
    #[test]
    fn a_grain_on_a_settled_pile_wakes_it() {
        let mut pile = Sandpile::new(21, &ORTHOGONAL, Boundary::Sink);
        let (cx, cy) = pile.center();
        pile.add(cx, cy, 3);
        pile.relax_fully();
        assert!(pile.is_stable());
        pile.add(cx, cy, 1);
        assert!(!pile.is_stable());
        assert!(pile.sweep(), "the fourth grain must topple");
        assert_eq!(pile.grid().get(cx, cy), 0);
    }

    /// A sweep can grow the grid and topple nothing; the UI redraws on a size change as well
    /// as on topplings because of exactly this case. If it ever stops being possible, the
    /// extra check in the UI is dead code; if it is ignored, the picture goes stale and clicks
    /// land in the wrong cell.
    #[test]
    fn a_sweep_can_grow_the_grid_without_toppling() {
        let mut pile = Sandpile::new(101, &ORTHOGONAL, Boundary::Grow);
        pile.add(1, 50, 1);
        assert!(!pile.sweep(), "one grain cannot topple");
        assert!(
            pile.grid().width() > 101,
            "a grain next to the frame should grow the grid"
        );
    }

    /// An even side would stop growing at 4000 without ever draining, so it is refused.
    #[test]
    #[should_panic(expected = "must be odd")]
    fn an_even_side_is_refused() {
        Sandpile::new(100, &ORTHOGONAL, Boundary::Grow);
    }

    /// Drip points are kept relative to the centre, so one far outside a small grid must grow
    /// the grid until it fits, and land exactly that far from the centre.
    #[test]
    fn add_relative_grows_the_grid_to_reach_far_cells() {
        let mut pile = Sandpile::new(5, &ORTHOGONAL, Boundary::Grow);
        assert!(pile.add_relative(-40, 25, 3));
        let (cx, cy) = pile.center();
        assert_eq!(pile.grid().get(cx - 40, cy + 25), 3);
        assert_eq!((pile.grains, pile.lost), (3, 0));
    }

    /// On a sink pile a relative position off the grid has fallen off: its grains are lost,
    /// and the grid must not grow.
    #[test]
    fn add_relative_off_a_sink_pile_is_lost() {
        let mut pile = Sandpile::new(5, &ORTHOGONAL, Boundary::Sink);
        assert!(pile.add_relative(10, 0, 2));
        assert_eq!((pile.grains, pile.lost), (0, 2));
        assert_eq!(pile.grid().width(), 5);
    }

    /// With eight neighbours the threshold is 8, and the same invariants must hold: no grain
    /// lost on a growing grid, every cell below 8 at the end, and the square's symmetries
    /// (the diagonal neighbours are symmetric under them too).
    #[test]
    fn eight_neighbour_pile_conserves_settles_and_is_symmetric() {
        let n = 30_000;
        let mut pile = Sandpile::new(3, &MOORE, Boundary::Grow);
        assert_eq!(pile.threshold(), 8);
        let (cx, cy) = pile.center();
        pile.add(cx, cy, n);
        pile.relax_fully();
        assert_eq!((sum(&pile), pile.lost), (u64::from(n), 0));
        assert!(pile.grid().cells().iter().all(|&h| h < 8));
        let g = pile.grid();
        let side = g.width();
        for y in 0..side {
            for x in 0..side {
                assert_eq!(g.get(x, y), g.get(side - 1 - x, y), "mirror at ({x}, {y})");
                assert_eq!(g.get(x, y), g.get(y, x), "diagonal at ({x}, {y})");
            }
        }
    }

    /// The abelian property holds for any neighbourhood, so it must hold here too: one grain
    /// at a time gives the same pile as all at once.
    #[test]
    fn eight_neighbour_toppling_order_does_not_matter() {
        let n = 3_000;
        let mut all = Sandpile::new(61, &MOORE, Boundary::Sink);
        let mut one = Sandpile::new(61, &MOORE, Boundary::Sink);
        let (cx, cy) = all.center();
        all.add(cx, cy, n);
        all.relax_fully();
        for _ in 0..n {
            one.add(cx, cy, 1);
            one.relax_fully();
        }
        assert_eq!(all.lost, 0, "grid too small for the test to mean anything");
        assert_eq!(all.grid(), one.grid());
    }

    /// The panel warns about drip points that lie off a sink grid; `holds_relative` decides
    /// which. The centre is held, the frame cell at the edge and anything beyond are not.
    #[test]
    fn holds_relative_marks_cells_off_the_pile() {
        let pile = Sandpile::new(11, &ORTHOGONAL, Boundary::Sink);
        assert!(pile.holds_relative(0, 0));
        assert!(pile.holds_relative(4, -4), "the last cell inside the frame");
        assert!(!pile.holds_relative(5, 0), "the frame itself");
        assert!(!pile.holds_relative(100, 0), "off the grid");
    }

    /// Not a check: prints how long big piles take, to compare relaxation strategies.
    #[test]
    #[ignore = "timing: run with --release -- --ignored --nocapture"]
    fn timing() {
        for n in [100_000, 1_000_000] {
            let start = std::time::Instant::now();
            let p = centred_pile(n, Boundary::Grow, 101);
            eprintln!(
                "{n} grains: {:?}, side {}, {} sweeps",
                start.elapsed(),
                p.grid().width(),
                p.sweeps
            );
        }
    }
}
