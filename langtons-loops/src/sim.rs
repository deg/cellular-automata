//! Langton's self-reproducing loop: an 8-state automaton on the von Neumann neighbourhood.
//!
//! Each cell looks at itself and its four orthogonal neighbours (north, east, south, west)
//! and looks the five states up in a transition table. The table is the whole automaton:
//! Christopher Langton designed it so that one particular loop of cells, a "sheath" (state 2)
//! around a "core" (state 1) carrying a train of signals, builds a copy of itself.
//!
//! # Source of the table and the loop
//!
//! - The 219 transitions are Table I of C. G. Langton, "Self-reproduction in cellular
//!   automata", Physica D 10 (1984) 135-144, doi:10.1016/0167-2789(84)90256-2, p. 140,
//!   written `CTRBL->I` (centre, top, right, bottom, left, then the next state).
//! - They were checked entry for entry against the machine-readable transcription in Golly,
//!   `Rules/Langtons-Loops.rule` (format `CNESWC'`, `symmetries:rotate4`), at Golly's
//!   SourceForge git commit 53bdb74e48cd0792f87a2e3e6a5bb0c5d7d5af0e
//!   (<https://sourceforge.net/p/golly/code/ci/master/tree/Rules/Langtons-Loops.rule>), and
//!   both list exactly the same 219 entries. Golly took them from Eli Bachmutsky's Java applet
//!   `loops.java` (1999), which has the same list.
//! - The starting loop is the paper's fig. 6, the same as Golly's
//!   `Patterns/Loops/Langtons-Loops.rle` and the applet's.
//!
//! The table is a list of published facts (the paper's), transcribed here, not code copied
//! from Golly: Golly's own files (GPL) are used only to check the transcription.
//!
//! # Rotations
//!
//! The paper lists each neighbourhood once: "rotations are not listed". So every entry also
//! stands for its three quarter-turns, the neighbours (T, R, B, L) cycled round (Golly's
//! `rotate4`, the applet's four assignments). Reflections are not included: the loop is
//! handed (its signals circulate one way round), and its mirror image is not a loop of this
//! rule.
//!
//! # Neighbourhoods missing from the table
//!
//! The paper doesn't say what they do. Golly's rule-table format says "for those not listed,
//! the central cell remains unchanged", and the applet does the same (`clearundefrules`). So
//! here an unlisted neighbourhood leaves the cell as it is. (Sayama's SDSR loop later gave
//! these cases a meaning of their own: a new state that dissolves structures.)
//!
//! # Edges
//!
//! Beyond the edge of the world every cell is 0 (empty), always: the world does not wrap.

use ca_core::Grid;

/// The number of states.
pub const STATES: usize = 8;

/// Langton's Table I: each token is `CTRBL` then `I`, the next state, where C is the cell,
/// and T, R, B, L the neighbours above, to the right, below and to the left. In the paper's
/// order.
pub const LANGTON_TABLE: &[&str] = &[
    "000000 000012 000020 000030 000050 000063 000071 000112 000122 000132 000212",
    "000220 000230 000262 000272 000320 000525 000622 000722 001022 001120 002020",
    "002030 002050 002125 002220 002322 005222 012321 012421 012525 012621 012721",
    "012751 014221 014321 014421 014721 016251 017221 017255 017521 017621 017721",
    "025271 100011 100061 100077 100111 100121 100211 100244 100277 100511 101011",
    "101111 101244 101277 102026 102121 102211 102244 102263 102277 102327 102424",
    "102626 102644 102677 102710 102727 105427 111121 111221 111244 111251 111261",
    "111277 111522 112121 112221 112244 112251 112277 112321 112424 112621 112727",
    "113221 122244 122277 122434 122547 123244 123277 124255 124267 125275 200012",
    "200022 200042 200071 200122 200152 200212 200222 200232 200242 200250 200262",
    "200272 200326 200423 200517 200522 200575 200722 201022 201122 201222 201422",
    "201722 202022 202032 202052 202073 202122 202152 202212 202222 202272 202321",
    "202422 202452 202520 202552 202622 202722 203122 203216 203226 203422 204222",
    "205122 205212 205222 205521 205725 206222 206722 207122 207222 207422 207722",
    "211222 211261 212222 212242 212262 212272 214222 215222 216222 217222 222272",
    "222442 222462 222762 222772 300013 300022 300041 300076 300123 300421 300622",
    "301021 301220 302511 401120 401220 401250 402120 402221 402326 402520 403221",
    "500022 500215 500225 500232 500272 500520 502022 502122 502152 502220 502244",
    "502722 512122 512220 512422 512722 600011 600021 602120 612125 612131 612225",
    "700077 701120 701220 701250 702120 702221 702251 702321 702525 702720",
];

/// The starting loop, fig. 6 of the paper: 10 rows by 15 columns, a space for 0. The core
/// holds the genome, six "7 0" signals (each extends the construction arm by one cell) and
/// two "4 0" signals (together they turn it left); the arm leaves at the bottom right.
pub const LANGTON_LOOP: [&str; 10] = [
    " 22222222",
    "2170140142",
    "2022222202",
    "272    212",
    "212    212",
    "202    212",
    "272    212",
    "21222222122222",
    "207107107111112",
    " 2222222222222",
];

/// The pattern's cells, as rows of states (padded with 0 to the longest row).
pub fn pattern(rows: &[&str]) -> Vec<Vec<u8>> {
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    rows.iter()
        .map(|r| {
            let mut row: Vec<u8> = r.bytes().map(|b| b.saturating_sub(b'0')).collect();
            row.resize(width, 0);
            row
        })
        .collect()
}

/// Two entries of a table, once rotated, that give the same neighbourhood different results.
#[derive(Debug, PartialEq, Eq)]
pub struct Conflict {
    /// The neighbourhood, `[C, T, R, B, L]`.
    pub neighbourhood: [u8; 5],
    pub results: (u8, u8),
}

/// A complete transition function: the next state for every one of the 8^5 neighbourhoods.
pub struct Rule {
    next: Vec<u8>,
}

/// Where neighbourhood `[C, T, R, B, L]` lives in the lookup table: 3 bits per state.
fn key(c: u8, t: u8, r: u8, b: u8, l: u8) -> usize {
    (c as usize) << 12 | (t as usize) << 9 | (r as usize) << 6 | (b as usize) << 3 | l as usize
}

impl Rule {
    /// Build the rule from a table of `CTRBLI` tokens (whitespace-separated, across any number
    /// of strings), each standing for itself and its three rotations. Neighbourhoods the table
    /// doesn't list keep their centre state. Fails if two entries disagree.
    pub fn from_table(table: &[&str]) -> Result<Self, Conflict> {
        let mut next: Vec<u8> = (0..STATES.pow(5)).map(|k| (k >> 12) as u8).collect();
        let mut set = vec![false; next.len()];
        for token in table.iter().flat_map(|s| s.split_whitespace()) {
            let d: Vec<u8> = token.bytes().map(|b| b - b'0').collect();
            assert!(
                d.len() == 6 && d.iter().all(|&s| (s as usize) < STATES),
                "bad table entry {token:?}"
            );
            let (c, mut n, result) = (d[0], [d[1], d[2], d[3], d[4]], d[5]);
            for _ in 0..4 {
                let k = key(c, n[0], n[1], n[2], n[3]);
                if set[k] && next[k] != result {
                    return Err(Conflict {
                        neighbourhood: [c, n[0], n[1], n[2], n[3]],
                        results: (next[k], result),
                    });
                }
                set[k] = true;
                next[k] = result;
                n.rotate_right(1); // (T, R, B, L) -> (L, T, R, B): a quarter-turn
            }
        }
        Ok(Self { next })
    }

    pub fn langton() -> Self {
        Self::from_table(LANGTON_TABLE).expect("Langton's table is consistent (tested)")
    }

    /// The next state of a cell in state `c` whose neighbours above, right, below and left are
    /// `t`, `r`, `b`, `l`.
    pub fn next(&self, c: u8, t: u8, r: u8, b: u8, l: u8) -> u8 {
        self.next[key(c, t, r, b, l)]
    }
}

/// The smallest rectangle `[x0, x1) x [y0, y1)` holding every non-empty cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

impl Bounds {
    pub fn width(&self) -> usize {
        self.x1 - self.x0
    }

    pub fn height(&self) -> usize {
        self.y1 - self.y0
    }

    fn include(this: Option<Self>, x: usize, y: usize) -> Option<Self> {
        Some(match this {
            None => Self {
                x0: x,
                y0: y,
                x1: x + 1,
                y1: y + 1,
            },
            Some(b) => Self {
                x0: b.x0.min(x),
                y0: b.y0.min(y),
                x1: b.x1.max(x + 1),
                y1: b.y1.max(y + 1),
            },
        })
    }
}

pub struct Loops {
    grid: Grid<u8>,
    rule: Rule,
    /// Where the non-empty cells are; `None` if there are none.
    bounds: Option<Bounds>,
    /// The new states of the cells being updated, reused across steps.
    scratch: Vec<u8>,
    pub generation: u64,
}

impl Loops {
    pub fn new(width: usize, height: usize, rule: Rule) -> Self {
        Self {
            grid: Grid::new(width, height, 0),
            rule,
            bounds: None,
            scratch: Vec::new(),
            generation: 0,
        }
    }

    pub fn grid(&self) -> &Grid<u8> {
        &self.grid
    }

    pub fn bounds(&self) -> Option<Bounds> {
        self.bounds
    }

    pub fn set(&mut self, x: usize, y: usize, state: u8) {
        self.grid.set(x, y, state);
        if state != 0 {
            self.bounds = Bounds::include(self.bounds, x, y);
        }
    }

    /// Place `rows` (see [`pattern`]) with its top left corner at `(x, y)`. Cells off the
    /// world are dropped; the pattern's 0s are written too.
    pub fn place(&mut self, x: usize, y: usize, rows: &[Vec<u8>]) {
        for (dy, row) in rows.iter().enumerate() {
            for (dx, &s) in row.iter().enumerate() {
                if x + dx < self.grid.width() && y + dy < self.grid.height() {
                    self.set(x + dx, y + dy, s);
                }
            }
        }
    }

    /// How many cells the next step will update: the occupied rectangle and a one-cell margin.
    /// Everything else is empty with empty neighbours, and stays empty (the table's first
    /// entry, 00000 -> 0), so it is skipped.
    pub fn work(&self) -> usize {
        self.active().map_or(0, |b| b.width() * b.height())
    }

    fn active(&self) -> Option<Bounds> {
        let b = self.bounds?;
        Some(Bounds {
            x0: b.x0.saturating_sub(1),
            y0: b.y0.saturating_sub(1),
            x1: (b.x1 + 1).min(self.grid.width()),
            y1: (b.y1 + 1).min(self.grid.height()),
        })
    }

    pub fn step(&mut self) {
        self.generation += 1;
        let Some(a) = self.active() else {
            return;
        };
        let (w, h) = (self.grid.width(), self.grid.height());
        let cells = self.grid.cells();
        self.scratch.clear();
        let mut bounds = None;
        for y in a.y0..a.y1 {
            let row = &cells[y * w..(y + 1) * w];
            let above = (y > 0).then(|| &cells[(y - 1) * w..y * w]);
            let below = (y + 1 < h).then(|| &cells[(y + 1) * w..(y + 2) * w]);
            for x in a.x0..a.x1 {
                let t = above.map_or(0, |r| r[x]);
                let b = below.map_or(0, |r| r[x]);
                let l = if x > 0 { row[x - 1] } else { 0 };
                let r = if x + 1 < w { row[x + 1] } else { 0 };
                let next = self.rule.next(row[x], t, r, b, l);
                if next != 0 {
                    bounds = Bounds::include(bounds, x, y);
                }
                self.scratch.push(next);
            }
        }
        let aw = a.width();
        let cells = self.grid.cells_mut();
        for (dy, new) in self.scratch.chunks_exact(aw).enumerate() {
            let start = (a.y0 + dy) * w + a.x0;
            cells[start..start + aw].copy_from_slice(new);
        }
        self.bounds = bounds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pattern turned a quarter-turn clockwise.
    pub fn rotate(rows: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let (h, w) = (rows.len(), rows.first().map_or(0, |r| r.len()));
        (0..w)
            .map(|x| (0..h).rev().map(|y| rows[y][x]).collect())
            .collect()
    }

    /// Every place `(x, y, quarter_turns)` where the grid holds `rows` exactly, turned clockwise
    /// by `quarter_turns`, including its 0s. Only the occupied rectangle is searched, so a pattern
    /// lying partly in empty space beyond it is not found; for a loop, whose sheath reaches the
    /// edges of its rectangle, that can't happen.
    pub fn find(
        grid: &Grid<u8>,
        bounds: Option<Bounds>,
        rows: &[Vec<u8>],
    ) -> Vec<(usize, usize, u8)> {
        let Some(b) = bounds else {
            return Vec::new();
        };
        let mut found = Vec::new();
        let mut p = rows.to_vec();
        for turns in 0..4 {
            let (ph, pw) = (p.len(), p[0].len());
            for y in b.y0..b.y1.saturating_sub(ph - 1) {
                for x in b.x0..b.x1.saturating_sub(pw - 1) {
                    let matches = p.iter().enumerate().all(|(dy, row)| {
                        row.iter()
                            .enumerate()
                            .all(|(dx, &s)| grid.get(x + dx, y + dy) == s)
                    });
                    if matches {
                        found.push((x, y, turns));
                    }
                }
            }
            p = rotate(&p);
        }
        found
    }

    fn langton(width: usize, height: usize, x: usize, y: usize) -> Loops {
        let mut l = Loops::new(width, height, Rule::langton());
        l.place(x, y, &pattern(&LANGTON_LOOP));
        l
    }

    /// The paper's Table I has 219 entries, and so does Golly's transcription (its header says
    /// `# rules: 219`). A dropped or doubled token would change the count.
    #[test]
    fn the_table_has_219_entries() {
        let tokens: Vec<&str> = LANGTON_TABLE
            .iter()
            .flat_map(|s| s.split_whitespace())
            .collect();
        assert_eq!(tokens.len(), 219);
        let mut unique = tokens.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 219, "no entry listed twice");
    }

    /// The table lists each neighbourhood once, and its rotations stand for themselves too, so
    /// no two entries, rotated, may claim the same neighbourhood with different results.
    /// (Golly applies the first match and the Java applet the last, so a conflict would also
    /// make the two disagree.) Also check the check: a table that does conflict is refused.
    #[test]
    fn rotations_never_conflict() {
        assert!(Rule::from_table(LANGTON_TABLE).is_ok());
        // 01234 -> 5 and its rotation 04123 -> 6 claim the same neighbourhood.
        let bad = Rule::from_table(&["012345 041236"]).err();
        assert_eq!(
            bad,
            Some(Conflict {
                neighbourhood: [0, 4, 1, 2, 3],
                results: (5, 6)
            })
        );
    }

    /// Every listed entry, in all four rotations, gives its listed result; an unlisted
    /// neighbourhood leaves the cell as it is (see the module notes).
    #[test]
    fn the_rule_follows_the_table_and_keeps_unlisted_cells() {
        let rule = Rule::langton();
        for token in LANGTON_TABLE.iter().flat_map(|s| s.split_whitespace()) {
            let d: Vec<u8> = token.bytes().map(|b| b - b'0').collect();
            let (c, i) = (d[0], d[5]);
            let (t, r, b, l) = (d[1], d[2], d[3], d[4]);
            assert_eq!(rule.next(c, t, r, b, l), i, "{token}");
            assert_eq!(rule.next(c, l, t, r, b), i, "{token} turned once");
            assert_eq!(rule.next(c, b, l, t, r), i, "{token} turned twice");
            assert_eq!(rule.next(c, r, b, l, t), i, "{token} turned three times");
        }
        assert_eq!(rule.next(0, 0, 0, 0, 0), 0, "empty space stays empty");
        // No entry starts 3333 (nor 7777), in any rotation.
        assert_eq!(rule.next(3, 3, 3, 3, 3), 3);
        assert_eq!(rule.next(7, 7, 7, 7, 7), 7);
        // Reflections are not implied: 01432 -> 1 is listed, but its mirror image 01234
        // (right and left swapped) is not, in any rotation, so that cell keeps its state, 0.
        assert_eq!(rule.next(0, 1, 4, 3, 2), 1);
        assert_eq!(rule.next(0, 1, 2, 3, 4), 0);
    }

    /// Fig. 1 of the paper: a "7 0" signal travels along a data path (core 1 between two rows
    /// of sheath 2) one cell per step, the 7 leading, the path behind it restored to core.
    /// The path here is long enough that its uncapped ends don't reach the signal in time.
    #[test]
    fn a_signal_moves_one_cell_per_step_along_a_path() {
        let mut l = Loops::new(40, 5, Rule::langton());
        for x in 0..40 {
            l.set(x, 1, 2);
            l.set(x, 2, 1);
            l.set(x, 3, 2);
        }
        l.set(15, 2, 0);
        l.set(16, 2, 7);
        for t in 0..8 {
            let core: Vec<u8> = (10..30).map(|x| l.grid().get(x, 2)).collect();
            let mut want = vec![1; 20];
            want[5 + t] = 0;
            want[6 + t] = 7;
            assert_eq!(core, want, "step {t}");
            for x in 10..30 {
                assert_eq!((l.grid().get(x, 1), l.grid().get(x, 3)), (2, 2));
            }
            l.step();
        }
    }

    /// The paper's loop fits in "a rectangular area of just 10 by 15 cells" (p. 144), and
    /// Wikipedia's table of loops gives it 86 cells (non-empty ones: Langton's Loops, "Langton's
    /// loops", the comparison table, retrieved 2026-11-16).
    #[test]
    fn the_starting_loop_is_10_by_15_with_86_cells() {
        let l = langton(30, 30, 5, 5);
        let b = l.bounds().unwrap();
        assert_eq!((b.height(), b.width()), (10, 15));
        assert_eq!(l.grid().cells().iter().filter(|&&s| s != 0).count(), 86);
    }

    /// The loops in the world that are, right now, exact copies of the starting loop in some
    /// orientation, each given by the top left corner of its 10 x 10 body (the loop without
    /// its construction arm). The body is the pattern's left 10 columns; turned clockwise by
    /// 1, 2 or 3 quarter-turns, it sits at (0, 0), (5, 0) or (0, 5) within the turned pattern.
    fn loop_bodies(l: &Loops) -> Vec<((usize, usize), u8)> {
        find(l.grid(), l.bounds(), &pattern(&LANGTON_LOOP))
            .into_iter()
            .map(|(x, y, turns)| {
                let (dx, dy) = [(0, 0), (0, 0), (5, 0), (0, 5)][turns as usize];
                ((x + dx, y + dy), turns)
            })
            .collect()
    }

    /// The paper's central result (p. 141 and fig. 9): "after 151 time steps, we are left with
    /// an offspring loop which is an exact copy of its parent at time 0". So at generation
    /// 151, and not before, a second loop exists that matches the starting loop cell for
    /// cell, its empty cells and the position of its genome included, the same way up and
    /// beside the parent. (The parent itself, with its new arm, matches the starting loop
    /// turned a quarter, a few steps earlier; that is the parent, not a copy.) This checks the
    /// whole table at once: one wrong entry, or a mirrored neighbourhood, derails the
    /// construction.
    #[test]
    fn the_loop_copies_itself_in_151_steps() {
        let mut l = langton(80, 80, 30, 40);
        assert_eq!(loop_bodies(&l), vec![((30, 40), 0)]);
        for _ in 0..150 {
            l.step();
            let copies: Vec<_> = loop_bodies(&l)
                .into_iter()
                .filter(|&(body, _)| body != (30, 40))
                .collect();
            assert!(copies.is_empty(), "a copy at generation {}", l.generation);
        }
        l.step();
        assert_eq!(l.generation, 151);
        let copies = loop_bodies(&l);
        assert_eq!(copies.len(), 1, "one copy at 151: {copies:?}");
        let ((x, y), turns) = copies[0];
        assert_eq!(
            (y, turns),
            (40, 0),
            "level with the parent, the same way up"
        );
        assert!(x >= 40, "beside the parent's body, not over it: x = {x}");
    }

    /// "Since the offspring loop is an exact duplicate of the parent loop, it will behave in
    /// exactly the same manner as its parent and construct an offspring loop to its 'right'"
    /// (p. 141). So 151 steps after the first copy appeared, the copy has made its own, at
    /// the same offset from it as it is from the parent.
    #[test]
    fn the_copy_copies_itself_151_steps_later() {
        let mut l = langton(120, 80, 20, 40);
        for _ in 0..151 {
            l.step();
        }
        let ((x1, y1), _) = loop_bodies(&l)[0];
        let offset = x1 - 20;
        for _ in 0..151 {
            l.step();
        }
        assert!(
            loop_bodies(&l).contains(&((x1 + offset, y1), 0)),
            "at generation 302: {:?}",
            loop_bodies(&l)
        );
    }

    /// The colony (p. 143): "the original loop (the 'Adam' loop) will reproduce four times",
    /// each offspring built after the construction arm has "moved 90 degrees
    /// counterclockwise" (p. 141), so one on each side of it. And once a loop finds every
    /// site taken, "its cycling instructions are slowly erased. The final result is an empty
    /// or 'dead' loop": only core (1) and sheath (2) are left. Meanwhile the colony keeps
    /// growing: "an expanding colony of loops growing out into the array".
    #[test]
    fn adam_has_four_children_then_dies_and_the_colony_grows() {
        let (ax, ay) = (200, 200);
        let mut l = langton(400, 400, ax, ay);
        let mut loops = std::collections::BTreeSet::new();
        let mut seen_at = Vec::new();
        for _ in 0..1000 {
            l.step();
            loops.extend(loop_bodies(&l).into_iter().map(|(body, _)| body));
            if l.generation.is_multiple_of(250) {
                seen_at.push(loops.len());
            }
        }
        let (x, _) = loops
            .iter()
            .find(|&&(x, y)| y == ay && x > ax)
            .copied()
            .expect("a first child to the right");
        let d = x - ax;
        for (side, child) in [
            ("right", (ax + d, ay)),
            ("above", (ax, ay - d)),
            ("left", (ax - d, ay)),
            ("below", (ax, ay + d)),
        ] {
            assert!(loops.contains(&child), "no child {side} of Adam: {loops:?}");
        }
        for y in ay..ay + 10 {
            for x in ax..ax + 10 {
                let s = l.grid().get(x, y);
                assert!(
                    s == 1 || s == 2 || s == 0,
                    "Adam still holds a {s} at ({x}, {y})"
                );
            }
        }
        assert!(
            seen_at.windows(2).all(|w| w[1] > w[0]),
            "loops seen by generation 250, 500, ..: {seen_at:?}"
        );
    }
}
