//! Wireworld: electrons running along wires, built for drawing circuits.
//!
//! Each cell is **empty**, **conductor** (wire), an electron **head**, or an electron
//! **tail**. Every generation, all cells update at once:
//! - empty stays empty;
//! - a head becomes a tail;
//! - a tail becomes conductor;
//! - a conductor becomes a head if exactly 1 or 2 of its 8 neighbours are heads.
//!
//! The tail is what gives an electron a direction: the wire behind it is still a tail when
//! the head moves on, so the head can only advance away from it.

use ca_core::{Grid, moore_counts};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Head,
    Tail,
    Conductor,
}

pub struct Wireworld {
    grid: Grid<Cell>,
    counts: Vec<u8>,
    pub generation: u64,
}

impl Wireworld {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid::new(width, height, Cell::Empty),
            counts: Vec::new(),
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
        // Circuits are drawn on a finite board; nothing wraps.
        moore_counts(&self.grid, false, |c| c == Cell::Head, &mut self.counts);
        for (i, cell) in self.grid.cells_mut().iter_mut().enumerate() {
            *cell = match *cell {
                Cell::Empty => Cell::Empty,
                Cell::Head => Cell::Tail,
                Cell::Tail => Cell::Conductor,
                Cell::Conductor if matches!(self.counts[i], 1 | 2) => Cell::Head,
                Cell::Conductor => Cell::Conductor,
            };
        }
        self.generation += 1;
    }

    /// Turn every electron back into wire, leaving the circuit itself unchanged.
    pub fn remove_electrons(&mut self) {
        for cell in self.grid.cells_mut() {
            if matches!(cell, Cell::Head | Cell::Tail) {
                *cell = Cell::Conductor;
            }
        }
    }

    /// Draw `pattern` with its top-left corner at `(x0, y0)`; cells off the board are skipped.
    pub fn place(&mut self, pattern: &Pattern, x0: usize, y0: usize) {
        for (y, row) in pattern.rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let cell = match ch {
                    '#' => Cell::Conductor,
                    '@' => Cell::Head,
                    '~' => Cell::Tail,
                    _ => continue,
                };
                let (px, py) = (x0 + x, y0 + y);
                if px < self.grid.width() && py < self.grid.height() {
                    self.grid.set(px, py, cell);
                }
            }
        }
    }
}

/// A circuit drawn as text: `#` wire, `@` electron head, `~` electron tail, space empty.
pub struct Pattern {
    pub name: &'static str,
    pub note: &'static str,
    pub rows: &'static [&'static str],
}

impl Pattern {
    pub fn size(&self) -> (usize, usize) {
        let w = self
            .rows
            .iter()
            .map(|r| r.chars().count())
            .max()
            .unwrap_or(0);
        (w, self.rows.len())
    }
}

/// Built-in circuits. Each one's behaviour is checked by a test below.
pub const PATTERNS: &[Pattern] = &[
    Pattern {
        name: "Clock",
        note: "An electron circling a 12-cell loop sends a copy down the wire every 12 generations.",
        rows: &[" #@~", "#   #", "#   ####################", "#   #", " ###"],
    },
    Pattern {
        name: "Diodes",
        note: "Two clocks. Top: the diode lets electrons through. Bottom: the same diode, reversed, blocks them.",
        rows: &[
            " #@~",
            "#   #      ##",
            "#   ######## ############",
            "#   #      ##",
            " ###",
            "",
            " #@~",
            "#   #         ##",
            "#   ########## ##########",
            "#   #         ##",
            " ###",
        ],
    },
    Pattern {
        name: "OR gate",
        note: "Clocks of period 12 (top) and 8 (bottom) feed one output through diodes; the output carries both.",
        rows: &[
            " #@~",
            "#   #      ##",
            "#   ######## ##########",
            "#   #      ##         #",
            " ###                  #",
            "                      ###########",
            " @~                   #",
            "#  #       ##         #",
            "#  ######### ##########",
            " ##        ##",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// An electron (a head with a tail behind it) runs along a straight wire one cell per
    /// generation, in the direction away from its tail.
    #[test]
    fn an_electron_runs_along_a_wire() {
        let mut w = Wireworld::new(20, 3);
        for x in 0..20 {
            w.set(x, 1, Cell::Conductor);
        }
        w.set(3, 1, Cell::Tail);
        w.set(4, 1, Cell::Head);
        for t in 1..=10 {
            w.step();
            assert_eq!(w.grid().get(4 + t, 1), Cell::Head, "generation {t}");
            assert_eq!(w.grid().get(3 + t, 1), Cell::Tail);
            assert_eq!(w.count(Cell::Head), 1);
        }
    }

    /// A conductor touching 3 heads stays conductor: the rule is 1 or 2, and that limit is
    /// what lets circuits block and combine signals.
    #[test]
    fn three_heads_do_not_trigger_a_conductor() {
        let mut w = Wireworld::new(3, 3);
        w.set(1, 1, Cell::Conductor);
        for x in 0..3 {
            w.set(x, 0, Cell::Head);
        }
        w.step();
        assert_eq!(w.grid().get(1, 1), Cell::Conductor);
    }

    fn board_with(name: &str) -> Wireworld {
        let p = PATTERNS.iter().find(|p| p.name == name).unwrap();
        let (w, h) = p.size();
        let mut world = Wireworld::new(w, h);
        world.place(p, 0, 0);
        world
    }

    /// Generations (within the first 300) at which `(x, y)` holds an electron head.
    fn pulses_at(world: &mut Wireworld, x: usize, y: usize) -> Vec<u64> {
        let mut out = vec![];
        for _ in 0..300 {
            world.step();
            if world.grid().get(x, y) == Cell::Head {
                out.push(world.generation);
            }
        }
        out
    }

    fn gaps(pulses: &[u64]) -> Vec<u64> {
        pulses.windows(2).map(|w| w[1] - w[0]).collect()
    }

    /// The clock's loop is 12 cells, so its output must carry an electron exactly every 12
    /// generations, indefinitely: the tap must copy the electron without killing it.
    #[test]
    fn clock_pulses_every_12_generations() {
        let mut w = board_with("Clock");
        let p = pulses_at(&mut w, 23, 2);
        assert!(p.len() > 20, "{p:?}");
        assert!(gaps(&p).iter().all(|&g| g == 12), "{:?}", gaps(&p));
    }

    /// The point of the diode pattern: the forward diode's far end sees every pulse, the
    /// reversed diode's far end sees none.
    #[test]
    fn diodes_pass_one_way_only() {
        let mut w = board_with("Diodes");
        let mut top = vec![];
        let mut bottom = vec![];
        for _ in 0..300 {
            w.step();
            if w.grid().get(24, 2) == Cell::Head {
                top.push(w.generation);
            }
            if w.grid().get(24, 8) == Cell::Head {
                bottom.push(w.generation);
            }
        }
        assert!(
            gaps(&top).iter().all(|&g| g == 12) && top.len() > 20,
            "{top:?}"
        );
        assert!(bottom.is_empty(), "the reversed diode leaked: {bottom:?}");
    }

    /// An OR gate's output carries electrons from either input. With inputs of period 12 and
    /// 8, it must see pulses 8 apart and pulses 12 apart (or closer, where they interleave),
    /// and more pulses than either input alone. The diodes stop each clock's electrons from
    /// running back up the other input and wrecking its clock.
    #[test]
    fn or_gate_carries_both_inputs() {
        let mut w = board_with("OR gate");
        let p = pulses_at(&mut w, 32, 5);
        let alone = 300 / 12;
        assert!(p.len() > alone + 5, "{} pulses: {p:?}", p.len());
        // Both clocks must still be running at the end: neither was wrecked by the other.
        let mut later = vec![];
        for _ in 0..96 {
            w.step();
            if w.grid().get(32, 5) == Cell::Head {
                later.push(w.generation);
            }
        }
        assert!(later.len() >= 96 / 12 + 96 / 8 - 3, "{later:?}");
    }

    /// "Remove all electrons" must stop all activity and keep every wire cell, so a circuit
    /// can be reset and re-fed by hand.
    #[test]
    fn removing_electrons_keeps_the_wires() {
        let mut w = board_with("Clock");
        let wire_before = w.count(Cell::Conductor) + w.count(Cell::Head) + w.count(Cell::Tail);
        w.remove_electrons();
        assert_eq!(w.count(Cell::Head) + w.count(Cell::Tail), 0);
        assert_eq!(w.count(Cell::Conductor), wire_before);
        w.step();
        assert_eq!(w.count(Cell::Head), 0, "nothing should move afterwards");
    }
}
