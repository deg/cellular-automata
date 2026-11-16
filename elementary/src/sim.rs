//! Elementary cellular automata: one row of cells, each 0 or 1, where every cell's next
//! value depends on itself and its two neighbours.
//!
//! Three cells have 8 possible patterns (111, 110, ..., 000). A rule says which of the 8 give
//! a 1. Read those 8 answers, from 111 down to 000, as the bits of a binary number and you
//! get the rule's number, from 0 to 255 (Wolfram's numbering). Rule 30 = 00011110: patterns
//! 100, 011, 010 and 001 give 1, the rest 0.

use ca_core::Grid;

/// One row evolving under a rule, with its recent history kept as a space-time diagram:
/// row 0 at the top is the oldest kept, the last filled row is the current one.
pub struct Elementary {
    pub rule: u8,
    /// Whether the row's ends are neighbours (a ring) or have permanent 0s beyond them.
    pub wrap: bool,
    row: Vec<bool>,
    history: Grid<bool>,
    /// Rows of `history` in use; once full, the diagram scrolls up.
    filled: usize,
    pub generation: u64,
}

/// How a new row starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// One 1 in the middle: shows each rule's character most clearly.
    SingleCell,
    /// Each cell 1 with probability one half, from this seed.
    Random(u64),
}

impl Elementary {
    pub fn new(width: usize, rows_kept: usize, rule: u8, wrap: bool, start: Start) -> Self {
        let row = match start {
            Start::SingleCell => {
                let mut row = vec![false; width];
                row[width / 2] = true;
                row
            }
            Start::Random(seed) => {
                let mut rng = ca_core::Rng::new(seed);
                (0..width).map(|_| rng.chance(0.5)).collect()
            }
        };
        let mut history = Grid::new(width, rows_kept, false);
        history.cells_mut()[..width].copy_from_slice(&row);
        Self {
            rule,
            wrap,
            row,
            history,
            filled: 1,
            generation: 0,
        }
    }

    /// The space-time diagram: time runs down the screen.
    pub fn history(&self) -> &Grid<bool> {
        &self.history
    }

    pub fn row(&self) -> &[bool] {
        &self.row
    }

    /// The generation drawn in row `y` of [`Self::history`], or `None` for a row not yet
    /// reached. The last filled row is the current generation, and each row above it one
    /// generation earlier.
    pub fn generation_of_row(&self, y: usize) -> Option<u64> {
        (y < self.filled).then(|| self.generation + 1 + y as u64 - self.filled as u64)
    }

    /// The rule's answer for the 3-cell pattern `left, centre, right`.
    pub fn apply(rule: u8, left: bool, centre: bool, right: bool) -> bool {
        let pattern = (left as u8) << 2 | (centre as u8) << 1 | right as u8;
        rule >> pattern & 1 == 1
    }

    /// Compute the next row and add it to the diagram.
    pub fn step(&mut self) {
        let n = self.row.len();
        let cell = |i: isize| -> bool {
            if (0..n as isize).contains(&i) {
                self.row[i as usize]
            } else {
                self.wrap && self.row[i.rem_euclid(n as isize) as usize]
            }
        };
        let next: Vec<bool> = (0..n as isize)
            .map(|i| Self::apply(self.rule, cell(i - 1), cell(i), cell(i + 1)))
            .collect();
        self.row = next;
        self.generation += 1;

        let (width, height) = (self.history.width(), self.history.height());
        let cells = self.history.cells_mut();
        if self.filled < height {
            self.filled += 1;
        } else {
            cells.copy_within(width.., 0); // scroll everything up one row
        }
        let last = (self.filled - 1) * width;
        cells[last..last + width].copy_from_slice(&self.row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(rule: u8, width: usize, steps: usize, wrap: bool, start: Start) -> Elementary {
        let mut e = Elementary::new(width, steps + 1, rule, wrap, start);
        for _ in 0..steps {
            e.step();
        }
        e
    }

    fn bits(row: &[bool]) -> String {
        row.iter().map(|&b| if b { '1' } else { '0' }).collect()
    }

    /// The rule number's bits must be read with 111 as the highest bit and 000 the lowest,
    /// or every rule would be a different (mirrored or inverted) rule. Rule 30 worked by hand:
    /// a single 1 becomes 111, then 11001.
    #[test]
    fn rule_30_matches_a_hand_computation() {
        let e = run(30, 9, 2, false, Start::SingleCell);
        let h = e.history();
        let row = |y: usize| bits(&h.cells()[y * 9..(y + 1) * 9]);
        assert_eq!(row(0), "000010000");
        assert_eq!(row(1), "000111000");
        assert_eq!(row(2), "001100100");
    }

    /// Rule 204 copies each cell's own value (bit set exactly where the centre is 1), rule 0
    /// clears everything and rule 255 fills everything: the numbering's fixed points.
    #[test]
    fn identity_and_constant_rules() {
        let start = Start::Random(5);
        let original = Elementary::new(64, 2, 204, true, start).row().to_vec();
        assert_eq!(run(204, 64, 1, true, start).row(), original.as_slice());
        assert!(run(0, 64, 1, true, start).row().iter().all(|&b| !b));
        assert!(run(255, 64, 1, true, start).row().iter().all(|&b| b));
    }

    /// Rule 90 sets each cell to the XOR of its neighbours, so from a single cell, row t is
    /// Pascal's triangle mod 2: the Sierpinski triangle. Checked against binomial parity for
    /// 32 rows, which also tests that the diagram stores rows in order.
    #[test]
    fn rule_90_draws_pascals_triangle_mod_2() {
        let (w, t_max) = (129, 32);
        let e = run(90, w, t_max, false, Start::SingleCell);
        let c = w / 2;
        for t in 0..=t_max {
            for x in 0..w {
                let d = x as isize - c as isize;
                // Row t has a 1 at offset d iff t+d is even, |d| <= t, and C(t, (t+d)/2) is odd,
                // which by Lucas's theorem means k & (t - k) == 0, with k = (t+d)/2.
                let expected = d.abs() <= t as isize && (t as isize + d) % 2 == 0 && {
                    let k = ((t as isize + d) / 2) as usize;
                    k & (t - k) == 0
                };
                assert_eq!(e.history().get(x, t), expected, "row {t}, offset {d}");
            }
        }
    }

    /// Rule 184 is the traffic rule: a 1 (a car) moves right only into a 0 (a gap). Cars are
    /// never created or destroyed, so on a ring the count of 1s never changes.
    #[test]
    fn rule_184_conserves_cars_on_a_ring() {
        let mut e = Elementary::new(200, 2, 184, true, Start::Random(42));
        let cars = e.row().iter().filter(|&&b| b).count();
        for _ in 0..500 {
            e.step();
            assert_eq!(e.row().iter().filter(|&&b| b).count(), cars);
        }
    }

    /// Rule 184's traffic jams move backwards: a queue of stopped cars clears from the front,
    /// one car per generation, so the queue's front edge moves back one cell per generation
    /// even though every car only ever moves forwards. The README says so; this checks it.
    #[test]
    fn rule_184_jams_clear_from_the_front() {
        let mut e = Elementary::new(100, 2, 184, true, Start::SingleCell);
        let mut row = vec![false; 100];
        row[50..60].fill(true); // ten cars, bumper to bumper
        e.row = row;
        for t in 1..=9 {
            e.step();
            let front = (50..100).take_while(|&i| e.row()[i]).last().unwrap();
            assert_eq!(front, 59 - t, "generation {t}");
        }
    }

    /// The pointer readout names the generation in each row of the diagram: rows count up
    /// from 0 while the diagram fills, rows below the current one have none yet, and once it
    /// scrolls the top row is the oldest generation kept.
    #[test]
    fn each_row_knows_its_generation() {
        let mut e = Elementary::new(9, 5, 30, true, Start::SingleCell);
        e.step();
        e.step();
        let gens: Vec<_> = (0..5).map(|y| e.generation_of_row(y)).collect();
        assert_eq!(gens, [Some(0), Some(1), Some(2), None, None]);
        for _ in 0..10 {
            e.step();
        }
        let gens: Vec<_> = (0..5).map(|y| e.generation_of_row(y)).collect();
        assert_eq!(gens, [Some(8), Some(9), Some(10), Some(11), Some(12)]);
        assert_eq!(e.generation_of_row(4), Some(e.generation));
    }

    /// Once the diagram is full it scrolls: the newest row stays at the bottom and the
    /// height never changes. An off-by-one here would show the current row twice or not at all.
    #[test]
    fn full_diagram_scrolls_with_the_newest_row_last() {
        let e = run(30, 41, 100, true, Start::SingleCell);
        let mut small = Elementary::new(41, 10, 30, true, Start::SingleCell);
        for _ in 0..100 {
            small.step();
        }
        let h = small.history();
        assert_eq!(h.height(), 10);
        assert_eq!(&h.cells()[9 * 41..], small.row());
        // Row 0 of the scrolled diagram is generation 91 of the full one.
        assert_eq!(&h.cells()[..41], &e.history().cells()[91 * 41..92 * 41]);
    }
}
