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
