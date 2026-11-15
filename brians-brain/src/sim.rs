//! Brian's Brain: a three-state automaton modelled loosely on neurons.
//!
//! Each cell is **ready** (off), **firing** (on) or **refractory** (recovering). Every
//! generation, all cells update at once:
//! - a ready cell fires if exactly 2 of its 8 neighbours are firing;
//! - a firing cell becomes refractory;
//! - a refractory cell becomes ready again.
//!
//! The refractory step is the difference from Life-like rules: a cell that just fired can't
//! fire again at once, which pushes activity onward, so most patterns move.

use ca_core::{Grid, moore_counts};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Ready,
    Firing,
    Refractory,
}

/// Which way a glider travels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heading {
    Up,
    Down,
    Left,
    Right,
}

pub struct Brain {
    grid: Grid<Cell>,
    counts: Vec<u8>,
    pub wrap: bool,
    pub generation: u64,
}

impl Brain {
    pub fn new(width: usize, height: usize, wrap: bool) -> Self {
        Self {
            grid: Grid::new(width, height, Cell::Ready),
            counts: Vec::new(),
            wrap,
            generation: 0,
        }
    }

    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.grid.set(x, y, cell);
    }

    pub fn count(&self, state: Cell) -> usize {
        self.grid.cells().iter().filter(|&&c| c == state).count()
    }

    /// Place the basic glider, travelling `heading`: two firing cells side by side at the
    /// front, `(x, y)` and the next cell across, with two recovering cells right behind them.
    /// The recovering pair is what makes it move cleanly (see the tests).
    pub fn add_glider(&mut self, x: usize, y: usize, heading: Heading) {
        let (across, behind): ((isize, isize), (isize, isize)) = match heading {
            Heading::Up => ((1, 0), (0, 1)),
            Heading::Down => ((1, 0), (0, -1)),
            Heading::Left => ((0, 1), (1, 0)),
            Heading::Right => ((0, 1), (-1, 0)),
        };
        let at = |dx: isize, dy: isize| ((x as isize + dx) as usize, (y as isize + dy) as usize);
        for (fx, fy) in [at(0, 0), at(across.0, across.1)] {
            self.set(fx, fy, Cell::Firing);
        }
        for (rx, ry) in [
            at(behind.0, behind.1),
            at(across.0 + behind.0, across.1 + behind.1),
        ] {
            self.set(rx, ry, Cell::Refractory);
        }
    }

    pub fn step(&mut self) {
        moore_counts(
            &self.grid,
            self.wrap,
            |c| c == Cell::Firing,
            &mut self.counts,
        );
        for (i, cell) in self.grid.cells_mut().iter_mut().enumerate() {
            *cell = match *cell {
                Cell::Ready if self.counts[i] == 2 => Cell::Firing,
                Cell::Ready => Cell::Ready,
                Cell::Firing => Cell::Refractory,
                Cell::Refractory => Cell::Ready,
            };
        }
        self.generation += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn firing_cells(b: &Brain) -> Vec<(usize, usize)> {
        let g = b.grid();
        (0..g.height())
            .flat_map(|y| (0..g.width()).map(move |x| (x, y)))
            .filter(|&(x, y)| g.get(x, y) == Cell::Firing)
            .collect()
    }

    /// The three-step cycle: a firing cell is refractory next, then ready. A lone firing
    /// cell has no neighbours firing, so nothing else happens around it.
    #[test]
    fn firing_then_refractory_then_ready() {
        let mut b = Brain::new(5, 5, false);
        b.set(2, 2, Cell::Firing);
        b.step();
        assert_eq!(b.grid().get(2, 2), Cell::Refractory);
        assert_eq!(b.count(Cell::Firing), 0);
        b.step();
        assert_eq!(b.grid().get(2, 2), Cell::Ready);
    }

    /// The basic glider, worked by hand: a firing pair with a refractory pair behind it.
    /// Only the two cells directly ahead touch both firing cells, so they fire; the
    /// refractory cells can't, and nothing at the sides touches two. So the pair moves one
    /// row per generation, forever. (A firing pair *without* the refractory pair behind it
    /// also sprouts cells at its sides, which is why the trailing pair matters.) This checks
    /// the "exactly 2" birth rule and the refractory state together.
    #[test]
    fn a_glider_moves_one_row_per_generation() {
        let mut b = Brain::new(20, 40, false);
        for x in [9, 10] {
            b.set(x, 30, Cell::Firing);
            b.set(x, 31, Cell::Refractory);
        }
        for t in 1..=20 {
            b.step();
            assert_eq!(
                firing_cells(&b),
                vec![(9, 30 - t), (10, 30 - t)],
                "generation {t}"
            );
            assert_eq!(b.count(Cell::Refractory), 2, "generation {t}");
        }
    }

    /// "Add four gliders" relies on `add_glider` getting the orientation right in all four
    /// directions: each glider must move one cell per generation the way it heads, intact.
    #[test]
    fn add_glider_moves_the_way_it_heads() {
        for (heading, (dx, dy)) in [
            (Heading::Up, (0, -1)),
            (Heading::Down, (0, 1)),
            (Heading::Left, (-1, 0)),
            (Heading::Right, (1, 0)),
        ] {
            let mut b = Brain::new(41, 41, false);
            b.add_glider(20, 20, heading);
            let start = firing_cells(&b);
            for _ in 0..10 {
                b.step();
            }
            let moved: Vec<_> = start
                .iter()
                .map(|&(x, y)| {
                    (
                        (x as isize + 10 * dx) as usize,
                        (y as isize + 10 * dy) as usize,
                    )
                })
                .collect();
            assert_eq!(firing_cells(&b), moved, "{heading:?}");
        }
    }

    /// A cell with 3 firing neighbours must not fire: the rule is exactly 2, not at least 2.
    #[test]
    fn three_firing_neighbours_do_not_fire_a_cell() {
        let mut b = Brain::new(5, 5, false);
        for x in 1..4 {
            b.set(x, 1, Cell::Firing);
        }
        b.step();
        assert_ne!(
            b.grid().get(2, 2),
            Cell::Firing,
            "(2,2) touched 3 firing cells"
        );
        assert_eq!(b.grid().get(1, 2), Cell::Firing, "(1,2) touched exactly 2");
    }
}
