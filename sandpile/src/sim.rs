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
//!
//! Once every height is small, a [`crate::jacobi`] stage takes over, sweeping every cell
//! simultaneously, in 16 bits, far faster: over a rectangle holding every grain (a
//! [`Patch`]), or, for a pile whose every grain went to the centre, over one quarter of it
//! (a [`Quarter`]). That pile has the square's 8-fold symmetry, so its in-place sweeps
//! cover one eighth of it too. 1,000,000 grains on the centre take 3.3 s natively (from
//! 102 s), 100,000 take 0.07 s. Tried and dropped, measured no better: skipping inactive
//! tiles, and a branch-free in-place loop.

use crate::jacobi::{self, Patch, Quarter};
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

/// The six neighbours of a hex cell in axial coordinates (see `ca_core::Layout::Hex`): the
/// two beside it in its row, and two each in the rows above and below.
pub const HEX: [(isize, isize); 6] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, -1), (-1, 1)];

/// By default grids never grow past this many cells on a side: bigger textures fail in many
/// browsers. A layout that draws cells wider than one texel needs a smaller limit, set with
/// [`Sandpile::with_max_side`].
pub const MAX_SIDE: usize = 4001;

/// What happens to grains that topple off the edge of the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary {
    /// The grid grows (up to its maximum side), so no grain is ever lost.
    Grow,
    /// The grid keeps its size and grains falling off the edge are lost.
    Sink,
}

/// An inclusive rectangle of cells, `x0..=x1` by `y0..=y1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bounds {
    pub(crate) x0: usize,
    pub(crate) y0: usize,
    pub(crate) x1: usize,
    pub(crate) y1: usize,
}

impl Bounds {
    pub(crate) fn area(self) -> usize {
        (self.x1 + 1 - self.x0) * (self.y1 + 1 - self.y0)
    }

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
    /// The grid never grows past this side; see [`Sandpile::with_max_side`].
    max_side: usize,
    boundary: Boundary,
    /// Every grain lies inside this box, or nothing does when it is `None`.
    occupied: Option<Bounds>,
    settled: bool,
    /// While true, the pile has the square's 8-fold symmetry (every grain so far went to the
    /// centre, the edge grows, and the neighbourhood is symmetric), so sweeps work on one
    /// eighth of it, the octant `0 <= j <= i` of offsets from the centre, and the rest of the
    /// grid is filled in by mirroring. See [`Sandpile::sweep_octant`].
    symmetric: bool,
    /// Whether `grid` is out of date: outside the octant in symmetric mode, or after a Jacobi
    /// sweep not yet written back. [`Sandpile::sync_grid`] brings it up to date. A Jacobi
    /// stage goes on running after a sync, with the grid up to date until its next sweep,
    /// which [`Sandpile::add_to_jacobi`] relies on.
    grid_stale: bool,
    /// In symmetric mode, once every height is small, the pile's quarter lives here and is
    /// swept by the much faster Jacobi stage (see [`jacobi`]); `grid` is then a copy.
    quarter: Option<Quarter>,
    /// The same for any other pile, once every height is small: a rectangle of the grid
    /// holding every grain.
    patch: Option<Patch>,
    /// The neighbour count, if a [`Patch`] can sweep this neighbourhood.
    patch_kernel: Option<usize>,
    /// False only in tests, to compare the fast paths against the plain sweep.
    fast: bool,
    /// Sweeps done by a Jacobi stage, so tests can tell it was used.
    jacobi_sweeps: u64,

    /// Grains currently on the grid.
    pub grains: u64,
    /// Grains that fell off a sink edge (or off the edge of a grid at its maximum side).
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
            max_side: MAX_SIDE,
            boundary,
            occupied: None,
            settled: true,
            symmetric: boundary == Boundary::Grow && is_square_neighbourhood(offsets),
            grid_stale: false,
            quarter: None,
            patch: None,
            patch_kernel: patch_kernel(offsets),
            fast: true,
            jacobi_sweeps: 0,
            grains: 0,
            lost: 0,
            topples: 0,
            sweeps: 0,
        };
        pile.compute_deltas();
        pile
    }

    /// Cap the grid's growth at `max_side` (odd, like every side here) instead of
    /// [`MAX_SIDE`]. The hex layout draws a pile 3 times as wide as its side, so it needs one.
    pub fn with_max_side(mut self, max_side: usize) -> Self {
        assert!(
            max_side % 2 == 1 && max_side >= self.grid.width(),
            "bad max side {max_side}"
        );
        self.max_side = max_side;
        self
    }

    pub fn max_side(&self) -> usize {
        self.max_side
    }

    pub fn grid(&self) -> &Grid<u32> {
        debug_assert!(!self.grid_stale, "read a pile before syncing its grid");
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

    /// Cells one sweep visits: the box that holds every grain, or one eighth of it while
    /// the pile is symmetric; a Jacobi stage's cells count for less, being cheaper.
    pub fn sweep_area(&self) -> usize {
        if let Some(p) = &self.patch {
            return p.area() / patch_cells_per_cell(self.offsets.len());
        }
        self.occupied.map_or(0, |b| {
            let b = self.clamp_inside_frame(b);
            if let Some(q) = &self.quarter {
                let n = q.extent() + 2;
                n * n / JACOBI_CELLS_PER_CELL
            } else if self.symmetric {
                let n = self.octant_extent(b) + 1;
                n * (n + 1) / 2
            } else {
                (b.x1 + 1 - b.x0) * (b.y1 + 1 - b.y0)
            }
        })
    }

    /// Whether `n` more grains fit: the pile holds at most `u32::MAX` grains in all.
    ///
    /// Bounding the total, not each cell, is what keeps the counts safe: no cell can ever
    /// hold more than the whole pile, so no toppling in [`Sandpile::relax`] can overflow.
    pub fn can_add(&self, n: u32) -> bool {
        self.grains + u64::from(n) <= u64::from(u32::MAX)
    }

    /// Drop `n` grains on cell `(x, y)`. They topple only when [`Sandpile::relax`] runs.
    /// A drop on the frame grows the grid first, or is lost over a sink edge.
    /// Does nothing, and returns false, if the grains don't fit (see [`Sandpile::can_add`]).
    pub fn add(&mut self, mut x: usize, mut y: usize, n: u32) -> bool {
        if !self.can_add(n) {
            return false;
        }
        if self.add_to_jacobi(x, y, n) {
            return true;
        }
        // A Jacobi stage holds the heights while it runs; put them back first.
        self.leave_jacobi();
        if (x, y) != self.center() {
            self.break_symmetry();
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
    /// The app uses [`Sandpile::relax`], which mirrors a symmetric pile once per call.
    #[cfg(test)]
    pub fn sweep(&mut self) -> bool {
        let toppled = self.sweep_once();
        self.sync_grid();
        toppled
    }

    /// One sweep, possibly leaving `grid` stale (see [`Sandpile::sync_grid`]).
    fn sweep_once(&mut self) -> bool {
        if self.settled {
            return false;
        }
        if self.patch.as_ref().is_some_and(Patch::needs_room) {
            // Grains are near the patch's edge: below, the grid grows if it must, and a
            // bigger patch starts.
            self.leave_jacobi();
        }
        if self.patch.is_some() {
            return self.sweep_patch();
        }
        let Some(mut b) = self.occupied else {
            self.settled = true;
            return false;
        };
        if self.boundary == Boundary::Grow && self.touches_frame(b) {
            match self.grow() {
                Some(_) => b = self.occupied.expect("growing keeps the occupied box"),
                // At the largest size grains start falling off the edge; counting those in
                // one eighth of the frame isn't worth the bookkeeping, so go back to full.
                None => self.break_symmetry(),
            }
        }
        let b = self.clamp_inside_frame(b);
        if self.symmetric {
            if self.quarter.is_none() {
                self.try_enter_jacobi(b);
            }
            if self.quarter.is_some() {
                return self.sweep_jacobi();
            }
            return self.sweep_octant(b);
        }
        self.try_enter_patch(b);
        if self.patch.is_some() {
            return self.sweep_patch();
        }

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
        let draining = self.boundary == Boundary::Sink || self.grid.width() >= self.max_side;
        if draining && self.touches_frame(t) {
            self.drain_frame();
        }
        true
    }

    /// Sweep until stable or until `max_sweeps` have run. Returns the sweeps that toppled.
    pub fn relax(&mut self, max_sweeps: usize) -> usize {
        let mut done = 0;
        while done < max_sweeps && self.sweep_once() {
            done += 1;
        }
        self.sync_grid();
        done
    }

    /// Sweep until stable or until the sweeps have visited about `budget` cells, counting
    /// each sweep at [`Sandpile::sweep_area`] as it stands before that sweep, so the budget
    /// keeps holding as the pile grows. Always does at least one sweep. Returns the sweeps
    /// that toppled. This is what keeps a frame short: fixing a sweep count up front, from
    /// the area of a fresh one-cell pile, let a whole big drop run inside one frame.
    pub fn relax_within(&mut self, budget: usize) -> usize {
        let (mut spent, mut done) = (0, 0);
        loop {
            let cost = self.sweep_area().max(1);
            if done > 0 && spent + cost > budget {
                break;
            }
            if !self.sweep_once() {
                break;
            }
            spent += cost;
            done += 1;
        }
        self.sync_grid();
        done
    }

    /// Relax until stable. Only for tests and small piles: it can take a long time.
    #[cfg(test)]
    pub fn relax_fully(&mut self) {
        while self.sweep_once() {}
        self.sync_grid();
    }

    /// One sweep of a symmetric pile, over the octant `0 <= j <= i` of offsets from the
    /// centre only. Every cell there stands for its up to 8 mirror images, which topple
    /// alongside it; so a grain any image sends into the octant is added to the octant cell
    /// it lands on, and grains landing outside it are ignored, being accounted for by the
    /// images of their receivers. Away from the axes and the diagonal no image's grains can
    /// reach the octant and the plain offsets apply; near them, the images are enumerated.
    fn sweep_octant(&mut self, b: Bounds) -> bool {
        let c = self.grid.width() / 2;
        let octant = Octant {
            c,
            width: self.grid.width(),
            n: self.octant_extent(b) as isize,
            reach: self.reach as isize,
            offsets: &self.offsets,
        };
        // Only the square neighbourhoods have the symmetry; fixing the neighbour count at
        // compile time turns the division into a shift and unrolls the neighbour loop.
        let cells = self.grid.cells_mut();
        let (reached, topples, any) = match self.deltas.len() {
            4 => octant.pass::<4>(cells, self.deltas[..].try_into().expect("4 deltas")),
            8 => octant.pass::<8>(cells, self.deltas[..].try_into().expect("8 deltas")),
            n => unreachable!("a {n}-neighbour pile can't be symmetric"),
        };
        self.topples += topples;
        self.sweeps += 1;
        self.grid_stale = true;
        if !any {
            self.settled = true;
            return false;
        }
        // The pile is symmetric, so it reaches as far in every direction as along +x.
        let edge = reached as usize + self.reach;
        let mut occupied = self.occupied.expect("a sweep needs grains");
        occupied.include(c - edge, c - edge);
        occupied.include(c + edge, c + edge);
        self.occupied = Some(occupied);
        true
    }

    /// Start the Jacobi stage if every height in the octant is below [`jacobi::LIMIT`] and
    /// the grains are far enough from the edge that it can run until the grid must grow.
    fn try_enter_jacobi(&mut self, b: Bounds) {
        let n = self.octant_extent(b);
        let c = self.grid.width() / 2;
        if n + 2 * self.reach + 1 >= c {
            return;
        }
        let tallest = (0..=n)
            .flat_map(|j| (j..=n).map(move |i| (i, j)))
            .map(|(i, j)| self.grid.get(c + i, c + j))
            .max()
            .unwrap_or(0);
        if tallest >= jacobi::LIMIT {
            return;
        }
        self.sync_grid(); // the quarter includes cells below the diagonal
        self.quarter = Some(Quarter::from_grid(&self.grid, n, self.offsets.len()));
    }

    /// Start a [`Patch`] if this neighbourhood has one, every height is below
    /// [`jacobi::LIMIT`], and no grain is about to need more room than it would give.
    /// `b` is the box holding every grain, inside the frame.
    fn try_enter_patch(&mut self, b: Bounds) {
        let Some(neighbours) = self.patch_kernel.filter(|_| self.fast) else {
            return;
        };
        // No cell can hold more than the whole pile, so a small pile needs no scan.
        if self.grains >= u64::from(jacobi::LIMIT) && self.tallest(b) >= jacobi::LIMIT {
            return;
        }
        // A margin in proportion to the pile, so a spreading pile changes patch only a few
        // times; each change costs a copy out and back in.
        let side = self.grid.width();
        let margin = ((b.x1 - b.x0).max(b.y1 - b.y0) / 2).max(16);
        let at = Bounds {
            x0: b.x0.saturating_sub(margin),
            y0: b.y0.saturating_sub(margin),
            x1: (b.x1 + margin).min(side - 1),
            y1: (b.y1 + margin).min(side - 1),
        };
        // The patch's ring can lie on the grid's frame. Grains sent there are lost, which is
        // right only while the pile drains; otherwise the grid must grow first.
        let draining = self.boundary == Boundary::Sink || side >= self.max_side;
        let edge = |at_edge: bool| draining && at_edge;
        let sinks = [
            edge(at.x0 == 0),
            edge(at.x1 == side - 1),
            edge(at.y0 == 0),
            edge(at.y1 == side - 1),
        ];
        let patch = Patch::from_grid(&self.grid, at, b, neighbours, sinks);
        if !patch.needs_room() {
            self.patch = Some(patch);
        }
    }

    /// The tallest cell in the box `b`.
    fn tallest(&self, b: Bounds) -> u32 {
        (b.y0..=b.y1)
            .filter_map(|y| {
                let row = self.grid.index_of(b.x0, y)..=self.grid.index_of(b.x1, y);
                self.grid.cells()[row].iter().copied().max()
            })
            .max()
            .unwrap_or(0)
    }

    /// One sweep of the patch.
    fn sweep_patch(&mut self) -> bool {
        let p = self.patch.as_mut().expect("in the patch stage");
        let toppled = p.sweep();
        self.topples += toppled;
        self.sweeps += 1;
        self.jacobi_sweeps += 1;
        self.grid_stale = true;
        if toppled == 0 {
            self.settled = true;
            return false;
        }
        true
    }

    /// Add grains straight into a running Jacobi stage, if the cell lies in it and stays
    /// below [`jacobi::LIMIT`]; otherwise do nothing and return false. Leaving the stage
    /// and starting it again for every drip would copy the whole pile out and back each time.
    fn add_to_jacobi(&mut self, x: usize, y: usize, n: u32) -> bool {
        let centre = self.center();
        let height = if let Some(p) = &mut self.patch {
            p.add(x, y, n)
        } else if let Some(q) = &mut self.quarter {
            ((x, y) == centre).then(|| q.add_to_centre(n)).flatten()
        } else {
            None
        };
        let Some(height) = height else {
            return false;
        };
        // Keep an up-to-date grid up to date (the centre is its own mirror image).
        if !self.grid_stale {
            self.grid.set(x, y, height);
        }
        if let Some(b) = &mut self.occupied {
            b.include(x, y);
        }
        self.grains += u64::from(n);
        self.settled = false;
        true
    }

    /// One Jacobi sweep. When grains come near the edge, hand back to the octant sweep,
    /// which grows the grid; the Jacobi stage restarts on the bigger grid.
    fn sweep_jacobi(&mut self) -> bool {
        let q = self.quarter.as_mut().expect("in the Jacobi stage");
        let toppled = q.sweep();
        let extent = q.extent();
        self.topples += toppled;
        self.sweeps += 1;
        self.jacobi_sweeps += 1;
        self.grid_stale = true;
        if toppled == 0 {
            self.settled = true;
            return false;
        }
        if extent + 2 * self.reach + 1 >= self.grid.width() / 2 {
            self.leave_jacobi();
        }
        true
    }

    /// End a Jacobi stage, if one is running, writing its heights back into the grid.
    fn leave_jacobi(&mut self) {
        if self.quarter.is_some() || self.patch.is_some() {
            self.sync_grid();
            self.quarter = None;
            self.patch = None;
        }
    }

    /// The largest offset from the centre, along either axis, of any cell in the box `b`.
    fn octant_extent(&self, b: Bounds) -> usize {
        let c = self.grid.width() / 2;
        (b.x1 - c).max(c - b.x0).max(b.y1 - c).max(c - b.y0)
    }

    /// Bring the whole grid up to date so it can be read: from a Jacobi stage while one
    /// runs, otherwise, in symmetric mode, by mirroring the octant.
    fn sync_grid(&mut self) {
        if !self.grid_stale {
            return;
        }
        self.grid_stale = false;
        if let Some(p) = &self.patch {
            // Every grain is in the patch; any missing went over a sink edge.
            let on_patch = p.write_back(&mut self.grid);
            self.lost += self.grains - on_patch;
            self.grains = on_patch;
            let t = p.occupied();
            let occupied = self.occupied.get_or_insert(t);
            occupied.include(t.x0, t.y0);
            occupied.include(t.x1, t.y1);
            return;
        }
        if let Some(q) = &self.quarter {
            q.write_back(&mut self.grid);
            let (c, edge) = (self.grid.width() / 2, q.extent() + self.reach);
            let occupied = self.occupied.get_or_insert(Bounds::cell(c, c));
            occupied.include(c - edge, c - edge);
            occupied.include(c + edge, c + edge);
            return;
        }
        let Some(b) = self.occupied else { return };
        let c = self.grid.width() / 2;
        for y in b.y0..=b.y1 {
            for x in b.x0..=b.x1 {
                let (a, d) = (x.abs_diff(c), y.abs_diff(c));
                let (i, j) = (a.max(d), a.min(d));
                let v = self.grid.get(c + i, c + j);
                self.grid.set(x, y, v);
            }
        }
    }

    /// Leave symmetric mode for good, after filling in the whole grid. Called before any
    /// grain lands off the centre, which would break the symmetry.
    fn break_symmetry(&mut self) {
        self.leave_jacobi();
        self.sync_grid();
        self.symmetric = false;
    }

    /// Test hook: never use symmetric mode, so a centred pile takes the [`Patch`] path.
    #[cfg(test)]
    fn without_symmetry(mut self) -> Self {
        self.symmetric = false;
        self
    }

    /// Test hook: use only the plain in-place sweep, the reference every fast path must
    /// match exactly.
    #[cfg(test)]
    fn plain(mut self) -> Self {
        self.symmetric = false;
        self.fast = false;
        self
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

    /// Pad the grid on every side, keeping it odd-sized and no larger than its maximum side.
    /// Returns the margin added to every side, or `None` if the grid is at its largest.
    fn grow(&mut self) -> Option<usize> {
        // A Jacobi stage's cells are placed by grid coordinates, which growing shifts.
        self.leave_jacobi();
        let side = self.grid.width();
        let margin = (side / 2)
            .max(8)
            .min(self.max_side.saturating_sub(side) / 2);
        if margin == 0 {
            return None;
        }
        self.grid = self.grid.padded(margin, 0);
        self.compute_deltas();
        self.occupied = self.occupied.map(|b| b.shifted(margin));
        Some(margin)
    }
}

/// The geometry of one octant sweep (see [`Sandpile::sweep_octant`]).
struct Octant<'a> {
    /// The centre's index along each axis, and the grid's width.
    c: usize,
    width: usize,
    /// The largest offset from the centre swept.
    n: isize,
    reach: isize,
    offsets: &'a [(isize, isize)],
}

impl Octant<'_> {
    fn at(&self, i: isize, j: isize) -> usize {
        (self.c as isize + j) as usize * self.width + (self.c as isize + i) as usize
    }

    /// Topple every unstable cell of the octant once, for a pile with `N` neighbours (and
    /// so a threshold of `N`). Returns the furthest offset that toppled, the topplings
    /// counted over all mirror images, and whether anything toppled.
    fn pass<const N: usize>(&self, cells: &mut [u32], deltas: [isize; N]) -> (isize, u64, bool) {
        let threshold = N as u32;
        let (n, r) = (self.n, self.reach);
        let (mut reached, mut topples, mut any) = (0isize, 0u64, false);
        for j in 0..=n {
            let row = self.at(0, j);
            for i in j..=n {
                let idx = row + i as usize;
                let height = cells[idx];
                if height < threshold {
                    continue;
                }
                let k = height / threshold;
                cells[idx] = height % threshold;
                any = true;
                reached = reached.max(i);
                if j > r && i - j > 2 * r {
                    for d in deltas {
                        cells[idx.wrapping_add_signed(d)] += k;
                    }
                    topples += 8;
                    continue;
                }
                let (images, count) = mirror_images(i, j);
                for &(pi, pj) in &images[..count] {
                    for &(dx, dy) in self.offsets {
                        let (ti, tj) = (pi + dx, pj + dy);
                        if tj >= 0 && ti >= tj {
                            cells[self.at(ti, tj)] += k;
                        }
                    }
                }
                topples += count as u64;
            }
        }
        (reached, topples, any)
    }
}

/// How many cells of the quarter cost as much as one in-place cell, for the per-frame
/// budget. Measured natively: 0.10 ns against 2.5 ns, so 25; 16 leaves room for the
/// browser build.
const JACOBI_CELLS_PER_CELL: usize = 16;

/// The same for a [`Patch`], by neighbour count. Measured natively, 1,000,000 grains, as the
/// time per sweep (which counts the cells the same way in both) with the plain sweep against
/// the patch: 10.5 off the centre (12.7 over a sink edge), 7.3 on hex. Eight neighbours,
/// not measured, get the cautious 7.
fn patch_cells_per_cell(neighbours: usize) -> usize {
    if neighbours == 4 { 10 } else { 7 }
}

/// Whether `offsets` are exactly the 4 or the 8 square neighbours, the shapes the
/// symmetric fast paths handle. Both look the same mirrored across either axis or the
/// diagonal, which is what keeps a centred pile 8-fold symmetric.
fn is_square_neighbourhood(offsets: &[(isize, isize)]) -> bool {
    // Square symmetry alone isn't enough: the Jacobi stage hard-codes these two shapes
    // (the four diagonals alone, say, are symmetric too, but would come out wrong).
    matching_shape(offsets, &[&ORTHOGONAL, &MOORE]).is_some()
}

/// The neighbour count, if `offsets` are exactly a neighbourhood a [`Patch`] can sweep:
/// the 4 or 8 square neighbours, or the 6 hex ones.
fn patch_kernel(offsets: &[(isize, isize)]) -> Option<usize> {
    matching_shape(offsets, &[&ORTHOGONAL, &MOORE, &HEX])
}

/// The neighbour count, if `offsets` are exactly one of `shapes`, in any order. A repeated
/// offset never matches: the threshold is the number of offsets, and the plain sweep sends a
/// repeated neighbour two grains, where a fast path built for the shape would send one.
fn matching_shape(offsets: &[(isize, isize)], shapes: &[&[(isize, isize)]]) -> Option<usize> {
    let set: std::collections::HashSet<_> = offsets.iter().copied().collect();
    if set.len() != offsets.len() {
        return None;
    }
    shapes
        .iter()
        .find(|shape| shape.len() == set.len() && shape.iter().all(|o| set.contains(o)))
        .map(|shape| shape.len())
}

/// The distinct mirror images of offset `(i, j)` under the square's 8 symmetries: the first
/// `count` of `images`. A fixed array, not a `Vec`: the octant sweep asks for these for
/// every toppling cell near the axes, and allocating each time made the first sweeps of a
/// small pile several times slower than the rest.
fn mirror_images(i: isize, j: isize) -> ([(isize, isize); 8], usize) {
    let mut images = [(0, 0); 8];
    let mut count = 0;
    for (a, b) in [(i, j), (j, i)] {
        for (sa, sb) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
            let p = (sa * a, sb * b);
            if !images[..count].contains(&p) {
                images[count] = p;
                count += 1;
            }
        }
    }
    (images, count)
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
    /// and grow to cover the cells the pile spreads into; a Jacobi stage's cells, being
    /// cheaper, must count for less.
    #[test]
    fn sweep_area_tracks_the_occupied_box() {
        let mut plain = Sandpile::new(21, &ORTHOGONAL, Boundary::Sink).plain();
        let mut fast = Sandpile::new(21, &ORTHOGONAL, Boundary::Sink);
        assert_eq!((plain.sweep_area(), fast.sweep_area()), (0, 0));
        for pile in [&mut plain, &mut fast] {
            let (cx, cy) = pile.center();
            pile.add(cx, cy, 4);
        }
        assert_eq!(plain.sweep_area(), 1);
        plain.relax_fully();
        fast.relax_fully();
        assert_eq!(plain.sweep_area(), 9, "one toppling reaches a 3 x 3 box");
        assert!(fast.patch.is_some(), "a small pile should use the patch");
        assert!((1..9).contains(&fast.sweep_area()), "{}", fast.sweep_area());
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

    /// On a hex grid a centred pile must have six-fold symmetry. In axial coordinates a
    /// 60-degree turn about the centre maps the offset (dq, dr) to (-dr, dq + dr). This also
    /// checks that the six offsets really form a hexagon: with a wrong one the pile would
    /// lose the symmetry. And no grain may be lost, with every cell left below 6.
    #[test]
    fn hex_pile_has_six_fold_symmetry() {
        let n = 20_000;
        let mut pile = Sandpile::new(3, &HEX, Boundary::Grow);
        assert_eq!(pile.threshold(), 6);
        let (cx, cy) = pile.center();
        pile.add(cx, cy, n);
        pile.relax_fully();
        assert_eq!((sum(&pile), pile.lost), (u64::from(n), 0));
        assert!(pile.grid().cells().iter().all(|&h| h < 6));
        let g = pile.grid();
        let (c, side) = (g.width() as isize / 2, g.width() as isize);
        let at = |dq: isize, dr: isize| {
            let (q, r) = (c + dq, c + dr);
            ((0..side).contains(&q) && (0..side).contains(&r))
                .then(|| g.get(q as usize, r as usize))
        };
        let mut compared = 0;
        for dr in -c..=c {
            for dq in -c..=c {
                if let (Some(a), Some(b)) = (at(dq, dr), at(-dr, dq + dr)) {
                    assert_eq!(a, b, "turning ({dq}, {dr})");
                    compared += 1;
                }
            }
        }
        assert!(compared > 1_000);
    }

    /// A smaller maximum side must stop growth there, after which grains fall off the edge
    /// and are counted as lost, as with a sink, instead of piling up unseen in the frame.
    #[test]
    fn growth_stops_at_a_custom_max_side() {
        let mut pile = Sandpile::new(3, &HEX, Boundary::Grow).with_max_side(21);
        let (cx, cy) = pile.center();
        pile.add(cx, cy, 5_000);
        pile.relax_fully();
        assert_eq!(pile.grid().width(), 21);
        assert!(pile.lost > 0);
        assert_eq!(pile.lost + sum(&pile), 5_000);
    }

    /// The octant sweep must give exactly the pile the plain sweep gives, for both square
    /// neighbourhoods, including while the grid grows. Any slip in the mirror rules near the
    /// axes or the diagonal (a doubled or missing grain) would show up as a difference.
    #[test]
    fn symmetric_sweep_matches_the_plain_sweep() {
        for offsets in [&ORTHOGONAL[..], &MOORE[..]] {
            let run = |pile: Sandpile| {
                let mut pile = pile;
                let (cx, cy) = pile.center();
                pile.add(cx, cy, 30_000);
                pile.relax_fully();
                pile
            };
            let fast = run(Sandpile::new(3, offsets, Boundary::Grow));
            let plain = run(Sandpile::new(3, offsets, Boundary::Grow).plain());
            assert!(
                fast.symmetric,
                "the centred pile should have used the octant"
            );
            assert!(fast.jacobi_sweeps > 0, "the Jacobi stage should have run");
            assert_eq!(fast.grid(), plain.grid(), "{} neighbours", offsets.len());
            assert_eq!((fast.grains, fast.lost), (30_000, 0));
        }
    }

    /// A grain off the centre in the middle of an avalanche must end symmetric mode cleanly:
    /// the mirror is filled in first, and the result matches a plain run of the same steps.
    #[test]
    fn breaking_symmetry_midway_matches_the_plain_sweep() {
        let run = |pile: Sandpile| {
            let mut pile = pile;
            let (cx, cy) = pile.center();
            pile.add(cx, cy, 8_000);
            pile.relax(15); // part way: the avalanche is still running
            let (cx, cy) = pile.center();
            pile.add(cx + 7, cy - 3, 500);
            pile.relax_fully();
            pile
        };
        let fast = run(Sandpile::new(3, &ORTHOGONAL, Boundary::Grow));
        let plain = run(Sandpile::new(3, &ORTHOGONAL, Boundary::Grow).plain());
        assert!(!fast.symmetric);
        assert_eq!(fast.grid(), plain.grid());
    }

    /// Relax a centred pile until the Jacobi stage is running, part way through.
    fn mid_jacobi(pile: Sandpile) -> Sandpile {
        let mut pile = pile;
        let (cx, cy) = pile.center();
        pile.add(cx, cy, 20_000);
        while pile.quarter.is_none() || pile.jacobi_sweeps < 100 {
            assert!(
                pile.relax(1) > 0,
                "settled before the Jacobi stage got going"
            );
        }
        pile
    }

    /// Dropping more grains at the centre while the Jacobi stage runs must go straight into
    /// its quarter (leaving and restarting the stage for every drip would copy the pile out
    /// and back each time), with nothing lost or double-counted: the result matches a plain
    /// run.
    #[test]
    fn a_centre_drop_during_the_jacobi_stage_matches_the_plain_sweep() {
        let mut fast = mid_jacobi(Sandpile::new(3, &ORTHOGONAL, Boundary::Grow));
        let (cx, cy) = fast.center();
        fast.add(cx, cy, 3_000);
        assert!(
            fast.quarter.is_some(),
            "the drop should go into the quarter"
        );
        fast.relax_fully();
        let mut plain = Sandpile::new(3, &ORTHOGONAL, Boundary::Grow).plain();
        let (cx, cy) = plain.center();
        plain.add(cx, cy, 20_000);
        plain.relax_fully();
        let (cx, cy) = plain.center();
        plain.add(cx, cy, 3_000);
        plain.relax_fully();
        // Abelian: when the second drop lands doesn't change the final pile.
        assert_eq!(fast.grid().width(), plain.grid().width());
        assert_eq!(fast.grid(), plain.grid());
        assert_eq!(fast.grains, 23_000);
    }

    /// A grain off the centre while the Jacobi stage runs must end it and symmetric mode
    /// cleanly; the result matches a plain run of the same drops.
    #[test]
    fn breaking_symmetry_during_the_jacobi_stage_matches_the_plain_sweep() {
        let mut fast = mid_jacobi(Sandpile::new(3, &MOORE, Boundary::Grow));
        let (dx, dy) = (5, -9);
        fast.add_relative(dx, dy, 2_000);
        fast.relax_fully();
        let mut plain = Sandpile::new(3, &MOORE, Boundary::Grow).plain();
        let (cx, cy) = plain.center();
        plain.add(cx, cy, 20_000);
        plain.relax_fully();
        plain.add_relative(dx, dy, 2_000);
        plain.relax_fully();
        assert!(!fast.symmetric);
        assert_eq!(fast.grid().width(), plain.grid().width());
        assert_eq!(fast.grid(), plain.grid());
    }

    /// A drip point beyond the grid while the Jacobi stage runs: `add_relative` grows the
    /// grid before the drop reaches `add`, so the quarter must survive the growth and be
    /// written back at the new centre. The result must match a plain run.
    #[test]
    fn a_drop_beyond_the_grid_during_the_jacobi_stage_matches_the_plain_sweep() {
        let mut fast = mid_jacobi(Sandpile::new(3, &ORTHOGONAL, Boundary::Grow));
        let far = fast.grid().width() as isize + 50;
        fast.add_relative(far, 0, 500);
        fast.relax_fully();
        let mut plain = Sandpile::new(3, &ORTHOGONAL, Boundary::Grow).plain();
        let (cx, cy) = plain.center();
        plain.add(cx, cy, 20_000);
        plain.relax_fully();
        plain.add_relative(far, 0, 500);
        plain.relax_fully();
        assert_eq!(fast.grid().width(), plain.grid().width());
        assert_eq!(fast.grid(), plain.grid());
        assert_eq!((fast.grains, fast.lost), (20_500, 0));
    }

    /// When a symmetric pile reaches its largest size, grains must start falling off the edge
    /// and be counted, as in the plain sweep: symmetric mode hands over to it.
    #[test]
    fn symmetric_pile_at_its_largest_size_matches_the_plain_sweep() {
        let run = |pile: Sandpile| {
            let mut pile = pile.with_max_side(21);
            let (cx, cy) = pile.center();
            pile.add(cx, cy, 5_000);
            pile.relax_fully();
            pile
        };
        let fast = run(Sandpile::new(3, &MOORE, Boundary::Grow));
        let plain = run(Sandpile::new(3, &MOORE, Boundary::Grow).plain());
        assert_eq!(fast.grid(), plain.grid());
        assert_eq!((fast.lost, fast.grains), (plain.lost, plain.grains));
        assert_eq!(fast.lost + sum(&fast), 5_000);
    }

    /// A neighbourhood listing a neighbour twice has a threshold one higher than its shape
    /// and sends that neighbour two grains, so the fast paths built for the shape would give
    /// a different pile. It must get none, in any order of offsets, and still match the plain
    /// sweep. (The app never builds one; this guards the public constructor.)
    #[test]
    fn a_repeated_neighbour_gets_no_fast_path() {
        let doubled = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 0)];
        assert_eq!(patch_kernel(&doubled), None);
        assert!(!is_square_neighbourhood(&doubled));
        assert_eq!(patch_kernel(&[(0, 1), (1, 0), (0, -1), (-1, 0)]), Some(4));
        let run = |pile: Sandpile| {
            let mut pile = pile;
            pile.add_relative(2, 0, 2_000);
            pile.relax_fully();
            pile
        };
        let fast = run(Sandpile::new(3, &doubled, Boundary::Grow));
        let plain = run(Sandpile::new(3, &doubled, Boundary::Grow).plain());
        assert_same_pile(&fast, &plain, "doubled right neighbour");
    }

    /// Symmetric mode is only safe for the two neighbourhoods its fast paths know: the four
    /// diagonals alone are just as symmetric but came out wrong in the Jacobi stage. And the
    /// octant rules rely on counting each cell's distinct mirror images correctly: 1 for the
    /// centre, 4 on an axis or the diagonal, 8 elsewhere.
    #[test]
    fn square_symmetry_and_mirror_images() {
        assert!(is_square_neighbourhood(&ORTHOGONAL));
        assert!(is_square_neighbourhood(&MOORE));
        assert!(!is_square_neighbourhood(&HEX));
        let diagonals = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        assert!(
            !is_square_neighbourhood(&diagonals),
            "symmetric, but not a shape the fast paths know"
        );
        let counts: Vec<_> = [(0, 0), (3, 0), (2, 2), (3, 1)]
            .iter()
            .map(|&(i, j)| mirror_images(i, j).1)
            .collect();
        assert_eq!(counts, vec![1, 4, 4, 8]);
    }

    /// The UI spends a fixed budget of cell visits per frame. Turning that into a sweep count
    /// once, from the area before any sweep, fails on a fresh pile: its area is one cell, so
    /// the whole relaxation would run in one frame and freeze the app. The budget must hold
    /// as the pile grows, sweep by sweep.
    #[test]
    fn a_frame_budget_holds_while_the_pile_grows() {
        let budget = 2_000_000;
        let mut pile = Sandpile::new(101, &ORTHOGONAL, Boundary::Grow);
        let (cx, cy) = pile.center();
        pile.add(cx, cy, 100_000);
        let naive = budget / pile.sweep_area().max(1);
        assert!(naive >= 1_000_000, "the old way allowed {naive} sweeps");
        pile.relax_within(budget);
        assert!(
            !pile.is_stable(),
            "one frame's budget must not settle 100,000 grains"
        );
        let mut frames = 1;
        while !pile.is_stable() {
            pile.relax_within(budget);
            frames += 1;
        }
        assert!(frames > 5, "settled in only {frames} frames");
    }

    /// Whether two piles are the same: the same grains on and off the grid, and the same
    /// height at every offset from the centre. Their grids may differ in size: a fast path
    /// may grow the grid at another moment than the plain sweep, which changes nothing else
    /// (growth loses no grain, and once at its largest side a grid stops growing).
    fn assert_same_pile(fast: &Sandpile, plain: &Sandpile, what: &str) {
        assert_eq!(
            (fast.grains, fast.lost),
            (plain.grains, plain.lost),
            "{what}: grains, lost"
        );
        let (big, small) = if fast.grid().width() >= plain.grid().width() {
            (fast, plain)
        } else {
            (plain, fast)
        };
        let (c, s) = (big.center().0 as isize, small.center().0 as isize);
        let g = big.grid();
        for y in 0..g.height() as isize {
            for x in 0..g.width() as isize {
                let (sx, sy) = (x - c + s, y - c + s);
                let on_small = (0..=2 * s).contains(&sx) && (0..=2 * s).contains(&sy);
                let other = if on_small {
                    small.grid().get(sx as usize, sy as usize)
                } else {
                    0
                };
                let h = g.get(x as usize, y as usize);
                assert_eq!(h, other, "{what}: offset ({}, {})", x - c, y - c);
            }
        }
    }

    /// Off the centre, on a hex grid, over a sink edge and at a grid's largest side, the
    /// patch must run and give exactly the pile the plain sweep gives.
    #[test]
    fn patch_matches_the_plain_sweep() {
        for offsets in [&ORTHOGONAL[..], &MOORE[..], &HEX[..]] {
            for (boundary, side, max_side) in [
                (Boundary::Grow, 3, MAX_SIDE),
                (Boundary::Grow, 3, 41),
                (Boundary::Sink, 61, 61),
            ] {
                let run = |pile: Sandpile| {
                    let mut pile = pile.with_max_side(max_side);
                    pile.add_relative(3, -2, 20_000);
                    pile.relax_fully();
                    pile
                };
                let fast = run(Sandpile::new(side, offsets, boundary));
                let plain = run(Sandpile::new(side, offsets, boundary).plain());
                let what = format!("{} neighbours, {boundary:?}, max {max_side}", offsets.len());
                assert!(fast.jacobi_sweeps > 0, "{what}: the patch should have run");
                assert_same_pile(&fast, &plain, &what);
            }
        }
    }

    /// Relax an off-centre pile until the patch is running, part way through.
    fn mid_patch(pile: Sandpile) -> Sandpile {
        let mut pile = pile;
        pile.add_relative(4, 1, 20_000);
        while pile.patch.is_none() || pile.jacobi_sweeps < 100 {
            assert!(pile.relax(1) > 0, "settled before the patch got going");
        }
        pile
    }

    /// A drip into a running patch must go straight into it, without leaving the stage
    /// (which would copy the whole pile out and back on every drip), and change nothing
    /// else: the result matches a plain run of the same drops.
    #[test]
    fn a_drip_into_a_running_patch_stays_in_it() {
        let mut fast = mid_patch(Sandpile::new(3, &HEX, Boundary::Grow));
        let mut plain = Sandpile::new(3, &HEX, Boundary::Grow).plain();
        plain.add_relative(4, 1, 20_000);
        for pile in [&mut fast, &mut plain] {
            pile.add_relative(0, 0, 7);
            pile.add_relative(-6, 5, 300);
        }
        assert!(
            fast.patch.is_some(),
            "the drips should have gone into the patch"
        );
        fast.relax_fully();
        plain.relax_fully();
        assert_same_pile(&fast, &plain, "drips");
    }

    /// A drip into the patch straight after a sync, just beyond the pile, then at once a drop
    /// outside the patch, which ends the stage before any sweep. The grid was up to date, so
    /// ending the stage writes nothing back: the drip must already be in the grid and in its
    /// occupied box, or the plain sweep never topples it, or it is dropped from the next
    /// patch, while still counted in `grains`.
    #[test]
    fn a_drip_then_a_drop_outside_the_patch_keeps_the_drip() {
        let mut fast = mid_patch(Sandpile::new(3, &ORTHOGONAL, Boundary::Grow));
        assert!(fast.patch.is_some() && !fast.grid_stale);
        let c = fast.center().0;
        let beyond_pile = (fast.occupied.unwrap().x1 + 3 - c) as isize;
        let mut plain = Sandpile::new(3, &ORTHOGONAL, Boundary::Grow).plain();
        plain.add_relative(4, 1, 20_000);
        for pile in [&mut fast, &mut plain] {
            pile.add_relative(beyond_pile, 0, 5);
        }
        assert!(fast.patch.is_some(), "the drip should go into the patch");
        for pile in [&mut fast, &mut plain] {
            pile.add_relative(-(c as isize) - 1, 0, 9);
        }
        assert!(fast.patch.is_none(), "the far drop should end the stage");
        fast.relax_fully();
        plain.relax_fully();
        assert_same_pile(&fast, &plain, "drip, then far drop");
    }

    /// A drop just beyond the grid while the patch runs: the grid grows before the drop
    /// lands, shifting every cell by the margin, which is half the old side. So the drop's
    /// new position is, in the old coordinates, right beside the pile, inside the patch: a
    /// patch kept across the growth would take the drop there, and go on sweeping a pile
    /// shifted from the grid's. The result must match a plain run.
    #[test]
    fn a_drop_beyond_the_grid_during_the_patch_matches_the_plain_sweep() {
        let mut fast = mid_patch(Sandpile::new(3, &ORTHOGONAL, Boundary::Grow));
        let beyond = -(fast.center().0 as isize + 1);
        let mut plain = Sandpile::new(3, &ORTHOGONAL, Boundary::Grow).plain();
        plain.add_relative(4, 1, 20_000);
        for pile in [&mut fast, &mut plain] {
            pile.add_relative(beyond, beyond, 500);
            pile.relax_fully();
        }
        assert_same_pile(&fast, &plain, "drop beyond the edge");
        assert_eq!((fast.grains, fast.lost), (20_500, 0));
    }

    /// Random piles, each built twice, with every fast path and with only the plain sweep:
    /// any neighbourhood, edge and size, several drops anywhere (on, near and beyond the
    /// grid), some in the middle of an avalanche and so of a Jacobi stage. Every pair must
    /// end the same. This found nothing the tests above miss when written, and is here to
    /// catch what a later change to the fast paths breaks.
    #[test]
    fn random_piles_match_the_plain_sweep() {
        let mut rng = ca_core::Rng::new(20261116);
        let mut below = |n: usize| (rng.next_u64() % n as u64) as usize;
        let (mut used_jacobi, mut added_to_jacobi) = (0, 0);
        for case in 0..150 {
            let offsets = [&ORTHOGONAL[..], &MOORE[..], &HEX[..]][below(3)];
            let side = 2 * below(20) + 3;
            let (boundary, max_side) = match below(3) {
                0 => (Boundary::Grow, MAX_SIDE),
                1 => (Boundary::Grow, side + 2 * below(30)),
                _ => (Boundary::Sink, side),
            };
            let symmetric = below(2) == 0;
            // Centred piles can take the patch too, if symmetric mode is off.
            let use_symmetry = below(2) == 0;
            let drops: Vec<_> = (0..1 + below(4))
                .map(|_| {
                    let r = side as isize;
                    let at = |rng: usize| if symmetric { 0 } else { rng as isize - r };
                    let grains = [1 + below(10), 1 + below(3_000), 1 + below(20_000)][below(3)];
                    let pause = below(60);
                    (
                        (at(below(2 * side + 1)), at(below(2 * side + 1))),
                        grains,
                        pause,
                    )
                })
                .collect();
            let mut run = |pile: Sandpile| {
                let mut pile = pile.with_max_side(max_side);
                for &((dx, dy), grains, pause) in &drops {
                    added_to_jacobi += usize::from(pile.patch.is_some() || pile.quarter.is_some());
                    pile.add_relative(dx, dy, grains as u32);
                    pile.relax(pause);
                }
                pile.relax_fully();
                pile
            };
            let fast = Sandpile::new(side, offsets, boundary);
            let fast = run(if use_symmetry {
                fast
            } else {
                fast.without_symmetry()
            });
            let plain = run(Sandpile::new(side, offsets, boundary).plain());
            used_jacobi += usize::from(fast.jacobi_sweeps > 0);
            let what = format!(
                "case {case}: {} neighbours, {boundary:?}, side {side}, max {max_side}, {drops:?}",
                offsets.len()
            );
            assert_same_pile(&fast, &plain, &what);
        }
        // Or the test would prove little about the fast paths.
        eprintln!("Jacobi stage in {used_jacobi} of 150 piles; {added_to_jacobi} drops into one");
        assert!(used_jacobi > 100 && added_to_jacobi > 30);
    }

    /// Not a check: prints how long big piles take, to compare relaxation strategies.
    #[test]
    #[ignore = "timing: run with --release -- --ignored --nocapture"]
    fn timing() {
        // (what, neighbours, boundary, starting side, drop offset from the centre)
        let cases = [
            ("centred", &ORTHOGONAL[..], Boundary::Grow, 101, (0, 0)),
            ("off-centre", &ORTHOGONAL[..], Boundary::Grow, 101, (1, 0)),
            ("hex", &HEX[..], Boundary::Grow, 101, (0, 0)),
            ("sink 501", &ORTHOGONAL[..], Boundary::Sink, 501, (0, 0)),
        ];
        let only = std::env::var("SANDPILE_TIMING").unwrap_or_default();
        for (what, offsets, boundary, side, (dx, dy)) in cases {
            if !only.is_empty() && !what.starts_with(only.as_str()) {
                continue;
            }
            for n in [100_000, 1_000_000] {
                let start = std::time::Instant::now();
                let mut p = Sandpile::new(side, offsets, boundary);
                if offsets == &HEX[..] {
                    p = p.with_max_side(1365);
                }
                p.add_relative(dx, dy, n);
                p.relax_fully();
                eprintln!(
                    "{what}, {n} grains: {:?}, side {}, {} sweeps",
                    start.elapsed(),
                    p.grid().width(),
                    p.sweeps
                );
            }
        }
    }
}
