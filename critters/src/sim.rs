//! Block cellular automata on the Margolus neighbourhood, kept free of UI code.
//!
//! The convention throughout is Norman Margolus's in *Crystalline Computation* (in A. J. G.
//! Hey (ed.), *Feynman and Computation*, Perseus Books, 1999, pp. 267–305;
//! arXiv:comp-gas/9811002), section 1.4: the grid is cut into 2 x 2 blocks, "the solid
//! lines show the grouping of the bits into 2×2 blocks that is used on the even
//! time-steps, the dotted lines show the odd-time grouping", and "this same rule is used
//! for both the even-time grouping and the odd-time grouping". Here the even blocking has
//! its corners at even coordinates, and the odd blocking is the same shifted one cell right
//! and one down. The world wraps round at its edges (periodic boundaries, as in Margolus's
//! Critters run of his figure 1.6), so the odd blocks along the edges are whole, and its
//! width and height must be even.
//!
//! Other sources state Critters differently: Wikipedia's article also describes a version
//! that "flips the states only in blocks with exactly two live cells, and in alternating
//! time steps rotates either the blocks with three live cells or the blocks with one live
//! cell", equivalent "up to flipping all states every other generation". Its table changes
//! from step to step, and is not used here.
//!
//! Each rule is written as its source's figure draws it: one block of each kind and what it
//! becomes, the other rotations following by symmetry ("each of the four discrete
//! rotations of a block that is shown on the left turns into the same rotation of the
//! corresponding result-block shown on the right", Margolus 1999, p. 12). The full table of
//! 16 is generated from that, and its inverse is computed, never written by hand.

use ca_core::{Grid, Rng};

/// A 2 x 2 block as four bits: bit 0 the top-left cell, bit 1 top-right, bit 2 bottom-left,
/// bit 3 bottom-right.
pub type Block = u8;

const TL: Block = 1;
const TR: Block = 2;
const BL: Block = 4;
const BR: Block = 8;

/// A block drawn as two rows, "#" for a 1 and "." for a 0: `"#./.#"` is the top-left and
/// bottom-right cells.
pub fn block(picture: &str) -> Block {
    let cells: Vec<bool> = picture
        .chars()
        .filter(|c| *c != '/')
        .map(|c| match c {
            '#' => true,
            '.' => false,
            _ => panic!("bad block picture {picture:?}"),
        })
        .collect();
    assert_eq!(cells.len(), 4, "bad block picture {picture:?}");
    [TL, TR, BL, BR]
        .iter()
        .zip(cells)
        .filter(|(_, on)| *on)
        .map(|(bit, _)| bit)
        .sum()
}

/// The block turned a quarter turn clockwise: top-left goes to top-right, and so on round.
pub fn rotate(b: Block) -> Block {
    let to = |from: Block, to: Block| if b & from != 0 { to } else { 0 };
    to(TL, TR) | to(TR, BR) | to(BR, BL) | to(BL, TL)
}

/// A block rule as its source draws it, and the opening it starts from.
#[derive(Debug, PartialEq, Eq)]
pub struct Rule {
    pub name: &'static str,
    /// One case of each kind, `(before, after)`, as [`block`] pictures. The other rotations
    /// of each follow by symmetry.
    pub figure: &'static [(&'static str, &'static str)],
    pub opening: Opening,
    /// The smallest world the opening shows well in.
    pub min_size: (usize, usize),
    /// Shown under the chips.
    pub note: &'static str,
}

/// What Restart puts in the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opening {
    /// An empty world with a square of random cells in the middle, a quarter of the world's
    /// height across (Margolus's figure 1.6 put a random 512 x 512 square in an empty
    /// 2,048 x 2,048 world).
    RandomSquare,
    /// A random gas filling the world, with a solid square in the middle (Margolus's figure
    /// 1.11b and c: a round pressure wave spreads out from the square).
    GasWithSquare,
    /// Billiard balls, each a pair of particles, inside a box of mirror walls.
    BallsInABox,
}

/// Critters: Margolus 1999, figure 1.5b (p. 12). Toffoli and Margolus introduced it in
/// *Cellular Automata Machines* (MIT Press, 1987), section 12.8.2, pp. 132–134, where p. 133
/// states it in words: "a block is complemented unless it contains exactly two [1]'s, in
/// which case it is left unchanged. A block containing three [1]'s is rotated through 180°
/// in addition to being complemented."
pub const CRITTERS: Rule = Rule {
    name: "Critters",
    figure: &[
        ("../..", "##/##"),
        ("#./..", ".#/##"),
        ("#./.#", "#./.#"),
        ("#./#.", "#./#."),
        (".#/##", "../.#"),
        ("##/##", "../.."),
    ],
    opening: Opening::RandomSquare,
    min_size: (0, 0),
    note: "Toffoli and Margolus's reversible rule. Gliders escape from the random square and bounce around the world, changing whatever they hit.",
};

/// The billiard-ball model cellular automaton (BBMCA): Margolus 1999, figure 1.8a (p. 16),
/// the same rule as figure 4 of N. Margolus, "Physics-like models of computation", Physica
/// D 10, 81–95 (1984), p. 86. Only two cases change: a lone 1 moves to the opposite corner,
/// and two 1s on a diagonal switch to the other diagonal.
pub const BILLIARD_BALLS: Rule = Rule {
    name: "Billiard balls",
    figure: &[
        ("../..", "../.."),
        ("#./..", "../.#"),
        ("#./.#", ".#/#."),
        ("#./#.", "#./#."),
        (".#/##", ".#/##"),
        ("##/##", "##/##"),
    ],
    opening: Opening::BallsInABox,
    min_size: (0, 0),
    note: "Margolus's billiard-ball model, 1984. Each ball is two particles, one behind the other, and the walls are mirrors. Balls that meet squarely knock each other off their paths; dropped at random, most meet askew and break up.",
};

/// The HPP lattice gas in block form: Margolus 1999, figure 1.11a (p. 19). Every particle
/// moves to the opposite corner of its block, except that "when exactly two molecules
/// appear on one diagonal of a block, they come out on the other diagonal".
pub const HPP_GAS: Rule = Rule {
    name: "HPP gas",
    figure: &[
        ("../..", "../.."),
        ("#./..", "../.#"),
        ("#./.#", ".#/#."),
        ("#./#.", ".#/.#"),
        (".#/##", "##/#."),
        ("##/##", "##/##"),
    ],
    opening: Opening::GasWithSquare,
    // Room for the wave to grow round before it wraps, as in Margolus's 512 x 512 figure.
    min_size: (640, 400),
    note: "A gas of particles moving diagonally; two meeting head-on leave at right angles. A round wave spreads from the square of particles.",
};

pub const RULES: [&Rule; 3] = [&CRITTERS, &BILLIARD_BALLS, &HPP_GAS];

impl Rule {
    /// The full table of 16: the figure's cases and every rotation of each. Panics if the
    /// figure gives one block two different results, or leaves a block out.
    pub fn table(&self) -> [Block; 16] {
        let mut table: [Option<Block>; 16] = [None; 16];
        for &(before, after) in self.figure {
            let (mut b, mut a) = (block(before), block(after));
            for _ in 0..4 {
                match table[b as usize] {
                    Some(old) if old != a => {
                        panic!("{}: block {b} gives both {old} and {a}", self.name)
                    }
                    _ => table[b as usize] = Some(a),
                }
                b = rotate(b);
                a = rotate(a);
            }
        }
        table.map(|t| t.unwrap_or_else(|| panic!("{}: a block is missing", self.name)))
    }
}

/// The table that undoes `table`, or `None` if two blocks share a result (the rule is then
/// not reversible).
pub fn inverse(table: &[Block; 16]) -> Option<[Block; 16]> {
    let mut inv: [Option<Block>; 16] = [None; 16];
    for (b, &a) in table.iter().enumerate() {
        if inv[a as usize].replace(b as Block).is_some() {
            return None;
        }
    }
    Some(inv.map(|b| b.expect("16 distinct results cover all 16 blocks")))
}

/// Whether every block with k 1s becomes one with 4 - k: then, as Margolus puts it for
/// Critters, "if we call 1's particles on even steps, and call 0's particles on odd steps,
/// then particles are conserved".
pub fn swaps_ones_and_zeros(table: &[Block; 16]) -> bool {
    (0..16u8).all(|b| table[b as usize].count_ones() == 4 - b.count_ones())
}

/// A world of 0s and 1s run by a block rule, forwards or backwards.
pub struct Blocks {
    grid: Grid<u8>,
    pub rule: &'static Rule,
    forward: [Block; 16],
    backward: [Block; 16],
    /// Whether particles are the 0s on odd generations (see [`swaps_ones_and_zeros`]).
    alternates: bool,
    /// Steps from the start: negative after running backwards past it. Its parity says
    /// which blocking the next step uses.
    pub generation: i64,
}

impl Blocks {
    pub fn new(width: usize, height: usize, rule: &'static Rule) -> Self {
        assert!(
            width >= 2 && height >= 2 && width.is_multiple_of(2) && height.is_multiple_of(2),
            "a wrapped Margolus world needs an even width and height, not {width} x {height}"
        );
        let forward = rule.table();
        let backward =
            inverse(&forward).unwrap_or_else(|| panic!("{} is not reversible", rule.name));
        Self {
            grid: Grid::new(width, height, 0),
            rule,
            forward,
            backward,
            alternates: swaps_ones_and_zeros(&forward),
            generation: 0,
        }
    }

    pub fn width(&self) -> usize {
        self.grid.width()
    }

    pub fn height(&self) -> usize {
        self.grid.height()
    }

    /// Which cell value is a particle now: 1, except on odd generations of a rule that
    /// swaps 1s and 0s, where it is 0.
    fn particle_value(&self) -> u8 {
        u8::from(!(self.alternates && self.generation.rem_euclid(2) == 1))
    }

    pub fn particle(&self, x: usize, y: usize) -> bool {
        self.grid.get(x, y) == self.particle_value()
    }

    pub fn set_particle(&mut self, x: usize, y: usize, on: bool) {
        let p = self.particle_value();
        self.grid.set(x, y, if on { p } else { 1 - p });
    }

    /// Every cell as particle (1) or empty (0), for drawing.
    pub fn particles(&self) -> Grid<u8> {
        let mut out = self.grid.clone();
        if self.particle_value() == 0 {
            for c in out.cells_mut() {
                *c = 1 - *c;
            }
        }
        out
    }

    pub fn particle_count(&self) -> usize {
        let p = self.particle_value();
        self.grid.cells().iter().filter(|&&c| c == p).count()
    }

    /// Empty the world and go back to generation 0.
    pub fn clear(&mut self) {
        self.grid = Grid::new(self.width(), self.height(), 0);
        self.generation = 0;
    }

    /// One step forwards: the rule on the blocking this generation uses.
    pub fn step(&mut self) {
        let table = self.forward;
        self.apply(&table, self.generation.rem_euclid(2) as usize);
        self.generation += 1;
    }

    /// One step backwards, undoing the step that led to this generation exactly: the
    /// inverse rule on the blocking that step used.
    pub fn step_back(&mut self) {
        self.generation -= 1;
        let table = self.backward;
        self.apply(&table, self.generation.rem_euclid(2) as usize);
    }

    /// Replace every block of the blocking shifted by `offset` (0 or 1) by its entry in
    /// `table`.
    fn apply(&mut self, table: &[Block; 16], offset: usize) {
        let (w, h) = (self.width(), self.height());
        let cells = self.grid.cells_mut();
        for y0 in (offset..h + offset).step_by(2) {
            let (top, bottom) = (y0 % h * w, (y0 + 1) % h * w);
            for x0 in (offset..w + offset).step_by(2) {
                let (left, right) = (x0 % w, (x0 + 1) % w);
                let b = cells[top + left]
                    | cells[top + right] << 1
                    | cells[bottom + left] << 2
                    | cells[bottom + right] << 3;
                let a = table[b as usize];
                cells[top + left] = a & 1;
                cells[top + right] = (a >> 1) & 1;
                cells[bottom + left] = (a >> 2) & 1;
                cells[bottom + right] = (a >> 3) & 1;
            }
        }
    }

    /// The rule's opening, drawn afresh from `rng`, at generation 0.
    pub fn restart(&mut self, rng: &mut Rng) {
        self.clear();
        let (w, h) = (self.width(), self.height());
        match self.rule.opening {
            Opening::RandomSquare => {
                let side = (h / 4).max(2);
                let (x0, y0) = ((w - side) / 2, (h - side) / 2);
                for y in y0..y0 + side {
                    for x in x0..x0 + side {
                        self.grid.set(x, y, u8::from(rng.chance(0.5)));
                    }
                }
            }
            Opening::GasWithSquare => {
                for c in self.grid.cells_mut() {
                    *c = u8::from(rng.chance(0.5));
                }
                let side = (h / 5).max(2);
                let (x0, y0) = ((w - side) / 2, (h - side) / 2);
                for y in y0..y0 + side {
                    for x in x0..x0 + side {
                        self.grid.set(x, y, 1);
                    }
                }
            }
            Opening::BallsInABox => {
                self.walls();
                let balls = (w * h / 1_500).max(1);
                for _ in 0..balls * 20 {
                    if self.ball_count_guess() >= balls {
                        break;
                    }
                    let x = 2 * (rng.next_u64() as usize % (w / 2));
                    let y = 2 * (rng.next_u64() as usize % (h / 2));
                    let corner = (rng.next_u64() % 4) as usize;
                    self.place_ball(x, y, corner);
                }
            }
        }
    }

    /// Walls two cells thick round the edge of the world. Every block of them is full or
    /// holds a pair of side-by-side particles, in both blockings, and neither changes under
    /// the billiard-ball rule: they are Margolus's mirrors ("square patterns of four
    /// particles straddling two adjacent blocks", which stay put "even if you put them right
    /// next to each other").
    fn walls(&mut self) {
        let (w, h) = (self.width(), self.height());
        for y in 0..h {
            for x in 0..w {
                if x < 2 || y < 2 || x >= w - 2 || y >= h - 2 {
                    self.grid.set(x, y, 1);
                }
            }
        }
    }

    fn ball_count_guess(&self) -> usize {
        let (w, h) = (self.width(), self.height());
        let walls = w * h - (w - 4) * (h - 4);
        (self.particle_count() - walls) / 2
    }

    /// A ball moving diagonally away from `corner` (0 top-left, 1 top-right, 2 bottom-left,
    /// 3 bottom-right) of the even block at `(x, y)`: a particle there and another two cells
    /// ahead of it, with one empty cell between, as in Margolus's figure 1.9. Placed only
    /// where the 6 x 6 cells around it are empty, so it starts clear of walls and balls.
    fn place_ball(&mut self, x: usize, y: usize, corner: usize) {
        let (dx, dy) = [(1, 1), (-1, 1), (1, -1), (-1, -1)][corner];
        let (cx, cy) = (x + corner % 2, y + corner / 2);
        let (w, h) = (self.width() as isize, self.height() as isize);
        let at = |i: isize, j: isize| (cx as isize + i, cy as isize + j);
        for j in -2..4 {
            for i in -2..4 {
                let (px, py) = at(i * dx, j * dy);
                if px < 0 || py < 0 || px >= w || py >= h {
                    return;
                }
                if self.grid.get(px as usize, py as usize) != 0 {
                    return;
                }
            }
        }
        for k in [0, 2] {
            let (px, py) = at(k * dx, k * dy);
            self.grid.set(px as usize, py as usize, 1);
        }
    }

    /// Fill a square of side `2 * radius + 1` around `(x, y)`, clipped to the world, with
    /// particles, empty cells, or (with `rng`) a random mix.
    pub fn paint(&mut self, x: usize, y: usize, radius: usize, fill: Fill, rng: &mut Rng) {
        let (w, h) = (self.width(), self.height());
        for py in y.saturating_sub(radius)..(y + radius + 1).min(h) {
            for px in x.saturating_sub(radius)..(x + radius + 1).min(w) {
                let on = match fill {
                    Fill::Particles => true,
                    Fill::Empty => false,
                    Fill::Random => rng.chance(0.5),
                };
                self.set_particle(px, py, on);
            }
        }
    }

    /// Cells whose particle-or-empty differs from `start` (taken at generation 0 with
    /// [`Blocks::particles`]).
    pub fn differences(&self, start: &Grid<u8>) -> usize {
        let p = self.particle_value();
        self.grid
            .cells()
            .iter()
            .zip(start.cells())
            .filter(|&(&c, &s)| (c == p) != (s == 1))
            .count()
    }
}

/// What a brush stroke puts down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    Particles,
    Empty,
    Random,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A world built from rows of "#" and ".", at generation 0.
    fn world(rule: &'static Rule, rows: &[&str]) -> Blocks {
        let mut b = Blocks::new(rows[0].len(), rows.len(), rule);
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                b.grid.set(x, y, u8::from(c == '#'));
            }
        }
        b
    }

    /// The particles' positions, sorted.
    fn positions(b: &Blocks) -> Vec<(usize, usize)> {
        let mut v = Vec::new();
        for y in 0..b.height() {
            for x in 0..b.width() {
                if b.particle(x, y) {
                    v.push((x, y));
                }
            }
        }
        v.sort();
        v
    }

    fn random_world(rule: &'static Rule, w: usize, h: usize, seed: u64) -> Blocks {
        let mut b = Blocks::new(w, h, rule);
        let mut rng = Rng::new(seed);
        for c in b.grid.cells_mut() {
            *c = u8::from(rng.chance(0.5));
        }
        b
    }

    /// The pictures read as drawn, and a quarter turn moves each corner on clockwise.
    #[test]
    fn block_pictures_and_rotation() {
        assert_eq!(block("#./.."), TL);
        assert_eq!(block(".#/.."), TR);
        assert_eq!(block("../#."), BL);
        assert_eq!(block("../.#"), BR);
        assert_eq!(rotate(TL), TR);
        assert_eq!(rotate(TR), BR);
        assert_eq!(rotate(BR), BL);
        assert_eq!(rotate(BL), TL);
        for b in 0..16 {
            assert_eq!(rotate(rotate(rotate(rotate(b)))), b);
        }
    }

    /// Each figure gives every one of the 16 blocks exactly one result (the table would
    /// panic otherwise), and the sources say each rule is reversible: "each of the 16
    /// possible initial states of a block is turned into a distinct result state. Thus the
    /// Critters rule is invertible" (Margolus 1999, p. 12); the billiard-ball rule "is its
    /// own inverse" (p. 16), and "like the BBMCA, [the HPP gas's] rule is its own inverse"
    /// (p. 20).
    #[test]
    fn every_table_is_a_permutation() {
        for rule in RULES {
            let t = rule.table();
            let inv = inverse(&t).unwrap_or_else(|| panic!("{} is not reversible", rule.name));
            for b in 0..16u8 {
                assert_eq!(inv[t[b as usize] as usize], b, "{}", rule.name);
            }
        }
    }

    /// The whole Critters table, by hand from the book's wording of the rule (Toffoli and
    /// Margolus 1987, p. 133): complement the block unless it holds exactly two 1s; a block
    /// of three is also "rotated through 180°". This checks the transcription of Margolus's
    /// 1999 figure against an independent statement of it.
    #[test]
    fn critters_is_complement_unless_two_and_three_turn_half_way() {
        let t = CRITTERS.table();
        for b in 0..16u8 {
            let expected = match b.count_ones() {
                2 => b,
                3 => rotate(rotate(!b & 15)),
                _ => !b & 15,
            };
            assert_eq!(t[b as usize], expected, "block {b:04b}");
        }
    }

    /// Margolus 1999, p. 12: "the number of 1's in the initial state of each block is, in
    /// all cases, equal to the number of 0's in the result", and "the Critters rule also
    /// conserves the parity (sum mod 2) along each diagonal of each block".
    #[test]
    fn critters_swaps_ones_and_zeros_and_keeps_diagonal_parity() {
        let t = CRITTERS.table();
        assert!(swaps_ones_and_zeros(&t));
        let parity = |b: Block, mask: Block| (b & mask).count_ones() % 2;
        for b in 0..16u8 {
            let a = t[b as usize];
            assert_eq!(parity(a, TL | BR), parity(b, TL | BR), "block {b:04b}");
            assert_eq!(parity(a, TR | BL), parity(b, TR | BL), "block {b:04b}");
        }
    }

    /// Toffoli and Margolus 1987, p. 133: "the block transformation performed by CRITTERS is
    /// not its own inverse", but "this rule is time-reversal invariant ... under the
    /// operation of complementation": its inverse is complement, rule, complement. And "the
    /// inverse rule must rotate through −180° and complement a block with one [1] in it".
    #[test]
    fn critters_is_not_its_own_inverse_but_complementing_reverses_it() {
        let t = CRITTERS.table();
        let inv = inverse(&t).unwrap();
        assert_ne!(t, inv);
        for b in 0..16u8 {
            assert_eq!(
                inv[b as usize],
                !t[(!b & 15) as usize] & 15,
                "block {b:04b}"
            );
        }
        for one in [TL, TR, BL, BR] {
            assert_eq!(inv[one as usize], rotate(rotate(!one & 15)));
        }
    }

    /// Margolus 1999, p. 16: the billiard-ball rule "conserves 1's (particles), and ... only
    /// two cases change", and it "is its own inverse". The same page's HPP rule is its own
    /// inverse too (p. 20), and its particles are conserved.
    #[test]
    fn billiard_balls_and_hpp_conserve_particles_and_undo_themselves() {
        for rule in [&BILLIARD_BALLS, &HPP_GAS] {
            let t = rule.table();
            assert!(!swaps_ones_and_zeros(&t));
            for b in 0..16u8 {
                assert_eq!(t[b as usize].count_ones(), b.count_ones(), "{}", rule.name);
                assert_eq!(t[t[b as usize] as usize], b, "{}", rule.name);
            }
        }
        let bbm = BILLIARD_BALLS.table();
        let changed = (0..16u8).filter(|&b| bbm[b as usize] != b).count();
        // A lone particle in any of 4 corners, and either diagonal pair.
        assert_eq!(changed, 4 + 2);
    }

    /// The HPP gas in Margolus's words (p. 18–19): each particle "simply moves to the
    /// opposite corner of its block", except "when exactly two molecules appear on one
    /// diagonal of a block, they come out on the other diagonal".
    #[test]
    fn hpp_moves_each_particle_to_the_opposite_corner_except_head_on() {
        let t = HPP_GAS.table();
        let opposite = |b: Block| rotate(rotate(b));
        for b in 0..16u8 {
            let expected = if b == TL | BR || b == TR | BL {
                b ^ 15
            } else {
                opposite(b)
            };
            assert_eq!(t[b as usize], expected, "block {b:04b}");
        }
    }

    /// The core promise: N steps forwards then N back give the exact starting grid, from
    /// random worlds, for many N, starting on either blocking. This tests the stepping
    /// (which blocking each step undoes); the tables' correctness rests on the tests above.
    #[test]
    fn forwards_then_backwards_restores_the_start_exactly() {
        for rule in RULES {
            for (seed, n) in [(1, 1), (2, 2), (3, 7), (4, 50), (5, 333)] {
                for start_generation in [0, 1, -3] {
                    let mut b = random_world(rule, 24, 16, seed);
                    b.generation = start_generation;
                    let start = b.grid.clone();
                    for _ in 0..n {
                        b.step();
                    }
                    if n > 2 {
                        assert_ne!(b.grid, start, "{} n={n}: nothing moved", rule.name);
                    }
                    for _ in 0..n {
                        b.step_back();
                    }
                    assert_eq!(b.generation, start_generation);
                    assert_eq!(b.grid, start, "{} n={n} from {start_generation}", rule.name);
                }
            }
        }
    }

    /// Turning round part-way, at an odd or an even generation, retraces the steps exactly:
    /// the off-by-one this guards against shows only when the direction flips mid-run.
    #[test]
    fn turning_round_mid_run_retraces_every_step() {
        for rule in RULES {
            for turn_at in [5, 6] {
                let mut b = random_world(rule, 20, 12, 9);
                let mut history = vec![b.grid.clone()];
                for _ in 0..turn_at {
                    b.step();
                    history.push(b.grid.clone());
                }
                // Back past the start, then forwards again to where we turned.
                for g in (0..turn_at).rev() {
                    b.step_back();
                    assert_eq!(b.grid, history[g], "{} at {g}", rule.name);
                }
                b.step_back();
                b.step_back();
                b.step();
                b.step();
                assert_eq!(b.grid, history[0]);
                for (g, then) in history.iter().enumerate().skip(1) {
                    b.step();
                    assert_eq!(&b.grid, then, "{} at {g}", rule.name);
                }
            }
        }
    }

    /// Critters as Margolus counts it: 1s are particles on even steps and 0s on odd ones, and
    /// so counted, particles are conserved (Margolus 1999, p. 12). The background, which
    /// flips every step, never shows as particles.
    #[test]
    fn critters_particles_are_conserved_and_the_background_stays_empty() {
        let mut b = Blocks::new(40, 30, &CRITTERS);
        b.restart(&mut Rng::new(3));
        let n = b.particle_count();
        let far = (1, 1);
        for _ in 0..200 {
            b.step();
            assert_eq!(b.particle_count(), n, "generation {}", b.generation);
            if b.generation < 10 {
                assert!(!b.particle(far.0, far.1));
            }
        }
        for _ in 0..400 {
            b.step_back();
            assert_eq!(b.particle_count(), n, "generation {}", b.generation);
        }
    }

    /// Margolus 1984, figure 10 (and 1999, p. 17): "a single 1 in one corner of a block moves
    /// to the opposite corner. When we change the blocking, the particle again finds itself
    /// alone in a block in the same corner it started in, and again moves in the same
    /// direction." So a lone particle runs along a diagonal, one cell a step, in the
    /// billiard-ball rule and the HPP gas alike.
    #[test]
    fn a_lone_particle_runs_diagonally() {
        for rule in [&BILLIARD_BALLS, &HPP_GAS] {
            // The particle's corner in its even block, (0, 0) the top-left: it heads for
            // the opposite corner, direction (1 - 2 ox, 1 - 2 oy).
            for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let mut b = Blocks::new(20, 20, rule);
                let (x0, y0) = (10 + ox, 10 + oy);
                b.set_particle(x0, y0, true);
                let dir = (1 - 2 * ox as isize, 1 - 2 * oy as isize);
                for k in 1..=6isize {
                    b.step();
                    let want = (
                        (x0 as isize + k * dir.0) as usize,
                        (y0 as isize + k * dir.1) as usize,
                    );
                    assert_eq!(positions(&b), vec![want], "{} step {k}", rule.name);
                }
            }
        }
    }

    /// Margolus 1984, figure 11 (p. 89): "a square of four ones straddling the boundary of
    /// two adjacent blocks will be stable". Here two such squares side by side (corners at
    /// odd x, even y), one the other way round (even x, odd y), and two more side by side
    /// that make a short wall, under the billiard-ball rule.
    #[test]
    fn mirror_squares_stay_put() {
        let mut b = world(
            &BILLIARD_BALLS,
            &[
                "............",
                "........##..",
                ".####...##..",
                ".####.......",
                ".......####.",
                ".......####.",
                "............",
                "............",
            ],
        );
        let start = b.grid.clone();
        for _ in 0..10 {
            b.step();
            assert_eq!(b.grid, start);
        }
    }

    /// Margolus 1999, figure 1.9 (p. 17): a collision of two minimum-size balls, six
    /// successive snapshots of an 8 x 8 area, transcribed cell by cell. "In the first image,
    /// the solid-blocks are about to be updated", so it is an even generation here. Each ball
    /// is two particles with one empty cell between; they meet, are held a step, and leave
    /// displaced from their paths.
    #[test]
    fn a_billiard_ball_collision_matches_margolus_figure() {
        let frames: [[(usize, usize); 4]; 6] = [
            [(0, 0), (2, 2), (2, 5), (0, 7)],
            [(1, 1), (3, 3), (3, 4), (1, 6)],
            [(2, 2), (3, 3), (3, 4), (2, 5)],
            [(3, 2), (2, 3), (2, 4), (3, 5)],
            [(4, 1), (2, 3), (2, 4), (4, 6)],
            [(5, 0), (3, 2), (3, 5), (5, 7)],
        ];
        // The 8 x 8 area sits at (4, 4) in a bigger world, on the same blocking.
        let mut b = Blocks::new(16, 16, &BILLIARD_BALLS);
        for &(x, y) in &frames[0] {
            b.set_particle(x + 4, y + 4, true);
        }
        for (i, frame) in frames.iter().enumerate() {
            let mut want: Vec<_> = frame.iter().map(|&(x, y)| (x + 4, y + 4)).collect();
            want.sort();
            assert_eq!(positions(&b), want, "frame {}", i + 1);
            b.step();
        }
    }

    /// Margolus 1999, p. 17: "Single particles just bounce back from such mirrors": a lone
    /// particle meeting a wall is held one step, then retraces its path exactly.
    #[test]
    fn a_particle_bounces_straight_back_off_a_wall() {
        let mut rows = vec!["..........".to_string(); 10];
        rows[8] = "##########".into();
        rows[9] = "##########".into();
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        let mut b = world(&BILLIARD_BALLS, &rows);
        b.set_particle(2, 2, true);
        let walls = b.particle_count() - 1;
        let mover = |b: &Blocks| {
            positions(b)
                .into_iter()
                .find(|&(_, y)| y < 8)
                .expect("the particle is still there")
        };
        let mut path = vec![mover(&b)];
        for _ in 0..14 {
            b.step();
            assert_eq!(b.particle_count(), walls + 1);
            path.push(mover(&b));
        }
        // Down-right to the row above the wall, held for a step, then back.
        let held = path.windows(2).position(|w| w[0] == w[1]).expect("held");
        assert_eq!(path[held], (7, 7));
        for k in 0..=held.min(path.len() - held - 2) {
            assert_eq!(path[held + 1 + k], path[held - k], "k={k}");
        }
    }

    /// The HPP gas's collisions conserve momentum (Toffoli and Margolus, "Invertible
    /// cellular automata: a review", Physica D 45, 1990, p. 243: "besides conserving energy
    /// and momentum ..."). A particle's direction is fixed by its corner in its block (it
    /// moves to the opposite corner), so the sum of directions over all particles must not
    /// change, step after step.
    #[test]
    fn the_hpp_gas_conserves_momentum() {
        let momentum = |b: &Blocks| {
            let off = b.generation.rem_euclid(2) as usize;
            let (mut px, mut py) = (0i64, 0i64);
            for y in 0..b.height() {
                for x in 0..b.width() {
                    if b.particle(x, y) {
                        let (ox, oy) = ((x + b.width() - off) % 2, (y + b.height() - off) % 2);
                        px += 1 - 2 * ox as i64;
                        py += 1 - 2 * oy as i64;
                    }
                }
            }
            (px, py)
        };
        let mut b = random_world(&HPP_GAS, 30, 20, 11);
        // Bias it so the momentum isn't zero.
        for y in 0..20 {
            for x in (0..30).step_by(2) {
                b.grid.set(x, y - y % 2, 1);
            }
        }
        let p = momentum(&b);
        assert_ne!(p, (0, 0));
        for _ in 0..100 {
            b.step();
            assert_eq!(momentum(&b), p, "generation {}", b.generation);
        }
    }

    /// Every opening is drawn afresh from the seed and starts at generation 0, and the
    /// billiard-ball opening's balls stay balls: particles are conserved and nothing
    /// escapes the box.
    #[test]
    fn openings() {
        for rule in RULES {
            let mut a = Blocks::new(160, 100, rule);
            let mut b = Blocks::new(160, 100, rule);
            a.restart(&mut Rng::new(1));
            b.restart(&mut Rng::new(2));
            assert_eq!(a.generation, 0);
            assert!(a.particle_count() > 0, "{}", rule.name);
            assert_ne!(a.grid, b.grid, "{}", rule.name);
        }
        let mut b = Blocks::new(160, 100, &BILLIARD_BALLS);
        b.restart(&mut Rng::new(5));
        let n = b.particle_count();
        let balls = b.ball_count_guess();
        assert!(balls >= 8, "{balls} balls");
        for _ in 0..500 {
            b.step();
            assert_eq!(b.particle_count(), n);
            for x in 0..160 {
                assert!(b.particle(x, 0) && b.particle(x, 1) && b.particle(x, 99));
            }
        }
    }

    /// Painting and the count go by particles, not raw cells, so on an odd Critters
    /// generation a painted particle is a stored 0.
    #[test]
    fn painting_follows_the_particle_view() {
        let mut b = Blocks::new(10, 10, &CRITTERS);
        b.step();
        assert_eq!(b.particle_count(), 0);
        b.paint(5, 5, 0, Fill::Particles, &mut Rng::new(0));
        assert_eq!(b.particle_count(), 1);
        assert_eq!(b.grid.get(5, 5), 0);
        let start = b.particles();
        assert_eq!(b.differences(&start), 0);
        b.paint(5, 5, 1, Fill::Empty, &mut Rng::new(0));
        assert_eq!(b.differences(&start), 1);
    }

    /// The four particles of a Critters glider: two side by side, two a little apart, as
    /// Margolus describes them ("compact symmetrical structures composed of four particles,
    /// with two particles adjacent to each other, and two slightly separated"). Found by a
    /// search of every four particles in a 5 x 5 square, not taken from a figure.
    const GLIDER: [(usize, usize); 4] = [(1, 0), (0, 1), (0, 2), (1, 3)];

    /// Margolus 1999, p. 13: "a glider goes through a cycle of four configurations each time
    /// it shifts by two positions", and his figure 1.10 shows gliders heading left, right,
    /// up and down. Each quarter turn of the glider, put down on the right blocking, moves
    /// two cells every four steps along one axis, through four different shapes, and keeps
    /// going; between them, the four go all four ways.
    #[test]
    fn critters_gliders_move_two_cells_every_four_steps() {
        let mut directions = Vec::new();
        let mut shape: Vec<(usize, usize)> = GLIDER.to_vec();
        for _ in 0..4 {
            // A quarter turn clockwise within a 4 x 4 box: (x, y) -> (3 - y, x).
            shape = shape.iter().map(|&(x, y)| (3 - y, x)).collect();
            let mut glides = None;
            for (px, py) in [(10, 10), (11, 10), (10, 11), (11, 11)] {
                let mut b = Blocks::new(40, 40, &CRITTERS);
                let start: Vec<_> = shape.iter().map(|&(x, y)| (x + px, y + py)).collect();
                for &(x, y) in &start {
                    b.set_particle(x, y, true);
                }
                let mut seen = vec![positions(&b)];
                for _ in 0..4 {
                    b.step();
                    seen.push(positions(&b));
                }
                let shifted = |dx: isize, dy: isize, k: isize| {
                    let mut v: Vec<_> = start
                        .iter()
                        .map(|&(x, y)| {
                            (
                                (x as isize + k * dx) as usize,
                                (y as isize + k * dy) as usize,
                            )
                        })
                        .collect();
                    v.sort();
                    v
                };
                for (dx, dy) in [(2, 0), (-2, 0), (0, 2), (0, -2)] {
                    if seen[4] == shifted(dx, dy, 1) {
                        // Shapes, not places: each frame moved to touch the origin.
                        let mut distinct: Vec<_> = seen[..4]
                            .iter()
                            .map(|frame| {
                                let x0 = frame.iter().map(|p| p.0).min().unwrap();
                                let y0 = frame.iter().map(|p| p.1).min().unwrap();
                                let mut f: Vec<_> =
                                    frame.iter().map(|&(x, y)| (x - x0, y - y0)).collect();
                                f.sort();
                                f
                            })
                            .collect();
                        distinct.sort();
                        distinct.dedup();
                        assert_eq!(distinct.len(), 4, "four different shapes on the way");
                        for k in 2..=5 {
                            for _ in 0..4 {
                                b.step();
                            }
                            assert_eq!(positions(&b), shifted(dx, dy, k), "lap {k}");
                        }
                        glides = Some((dx, dy));
                    }
                }
            }
            directions.push(glides.expect("each quarter turn glides from some blocking"));
        }
        directions.sort();
        assert_eq!(directions, vec![(-2, 0), (0, -2), (0, 2), (2, 0)]);
    }
}
