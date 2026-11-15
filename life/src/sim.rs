//! Conway's Game of Life, and the other "Life-like" rules written as `B.../S...`.
//!
//! Each cell is alive or dead. Every generation, all cells update at once from the count of
//! live cells among their eight neighbours: a dead cell is **born** if the count is in the
//! rule's birth set, and a live cell **survives** if it is in the survival set. Conway's rule
//! is B3/S23: born with exactly 3 neighbours, survives with 2 or 3.

use crate::rle::Shape;
use ca_core::{Grid, moore_counts};

/// A Life-like rule: which neighbour counts (0 to 8) cause birth, and which survival.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    birth: [bool; 9],
    survive: [bool; 9],
}

impl Rule {
    /// Conway's Game of Life, B3/S23.
    pub fn life() -> Self {
        Self::parse("B3/S23").expect("valid rule")
    }

    /// Parse a rule string such as `B3/S23` or `b36/s23`. Either part may be empty, as in
    /// `B2/S` (Seeds, where nothing survives).
    pub fn parse(text: &str) -> Result<Self, String> {
        let text = text.trim().to_ascii_uppercase();
        let (b, s) = text
            .split_once('/')
            .ok_or_else(|| format!("'{text}' has no '/': write it like B3/S23"))?;
        let digits = |part: &str, letter: char| -> Result<[bool; 9], String> {
            let rest = part
                .strip_prefix(letter)
                .ok_or_else(|| format!("'{part}' should start with {letter}"))?;
            let mut set = [false; 9];
            for c in rest.chars() {
                match c.to_digit(10) {
                    Some(d) if d <= 8 => set[d as usize] = true,
                    _ => return Err(format!("'{c}' is not a neighbour count from 0 to 8")),
                }
            }
            Ok(set)
        };
        Ok(Self {
            birth: digits(b, 'B')?,
            survive: digits(s, 'S')?,
        })
    }

    /// Whether a cell that is `alive` now, with `neighbours` live neighbours, lives next.
    fn next(&self, alive: bool, neighbours: usize) -> bool {
        if alive {
            self.survive[neighbours]
        } else {
            self.birth[neighbours]
        }
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let set =
            |s: &[bool; 9]| -> String { (0..9).filter(|&n| s[n]).map(|n| n.to_string()).collect() };
        write!(f, "B{}/S{}", set(&self.birth), set(&self.survive))
    }
}

/// A finite Life world. Its edges either wrap around (a torus, so a glider leaving one side
/// comes back on the other) or are surrounded by permanently dead cells.
pub struct Life {
    grid: Grid<bool>,
    /// Each cell's live-neighbour count, reused every generation.
    counts: Vec<u8>,
    pub rule: Rule,
    pub wrap: bool,
    pub generation: u64,
}

impl Life {
    pub fn new(width: usize, height: usize, rule: Rule, wrap: bool) -> Self {
        Self {
            grid: Grid::new(width, height, false),
            counts: Vec::new(),
            rule,
            wrap,
            generation: 0,
        }
    }

    pub fn grid(&self) -> &Grid<bool> {
        &self.grid
    }

    pub fn population(&self) -> usize {
        self.grid.cells().iter().filter(|&&alive| alive).count()
    }

    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        self.grid.set(x, y, alive);
    }

    pub fn clear(&mut self) {
        self.grid.cells_mut().fill(false);
        self.generation = 0;
    }

    /// Bring cells to life at `cells`, offset so the pattern's centre lands on the grid's.
    /// Cells that fall off the grid are dropped.
    pub fn place_centred(&mut self, shape: &Shape) {
        let (cells, size) = (&shape.cells, shape.size);
        let ox = self.grid.width() as isize / 2 - size.0 as isize / 2;
        let oy = self.grid.height() as isize / 2 - size.1 as isize / 2;
        for &(x, y) in cells {
            let (gx, gy) = (x as isize + ox, y as isize + oy);
            if gx >= 0 && gy >= 0 {
                let (gx, gy) = (gx as usize, gy as usize);
                if gx < self.grid.width() && gy < self.grid.height() {
                    self.grid.set(gx, gy, true);
                }
            }
        }
    }

    /// Advance one generation: every cell updates at once from the previous generation.
    pub fn step(&mut self) {
        moore_counts(&self.grid, self.wrap, |alive| alive, &mut self.counts);
        for (i, cell) in self.grid.cells_mut().iter_mut().enumerate() {
            *cell = self.rule.next(*cell, self.counts[i] as usize);
        }
        self.generation += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rle;

    fn world_with(name: &str, side: usize, wrap: bool) -> Life {
        let pattern = rle::PATTERNS.iter().find(|p| p.name == name).unwrap();
        let shape = rle::parse(pattern.rle).unwrap();
        let mut life = Life::new(side, side, Rule::life(), wrap);
        life.place_centred(&shape);
        life
    }

    fn live_cells(life: &Life) -> Vec<(usize, usize)> {
        let g = life.grid();
        (0..g.height())
            .flat_map(|y| (0..g.width()).map(move |x| (x, y)))
            .filter(|&(x, y)| g.get(x, y))
            .collect()
    }

    /// Rule strings round-trip, and case and empty parts are accepted, so the presets and
    /// whatever the user types both work. Seeds (B2/S) has an empty survival set.
    #[test]
    fn rules_parse_and_print() {
        assert_eq!(Rule::parse("b36/s23").unwrap().to_string(), "B36/S23");
        assert_eq!(Rule::parse("B2/S").unwrap().to_string(), "B2/S");
        assert!(Rule::parse("B9/S23").is_err(), "9 neighbours is impossible");
        assert!(Rule::parse("3/23").is_err(), "the letters are required");
    }

    /// The blinker is Life's simplest oscillator: three cells in a row turn to a column and
    /// back, period 2. Wrong neighbour counting breaks it at once.
    #[test]
    fn blinker_has_period_two() {
        let mut life = Life::new(5, 5, Rule::life(), false);
        for x in 1..4 {
            life.set(x, 2, true);
        }
        let start = live_cells(&life);
        life.step();
        assert_eq!(live_cells(&life), vec![(2, 1), (2, 2), (2, 3)]);
        life.step();
        assert_eq!(live_cells(&life), start);
    }

    /// A glider moves one cell diagonally every 4 generations. On a wrapping world it must
    /// cross the edge and come back whole, which checks the wrap-around neighbours.
    #[test]
    fn glider_crosses_a_wrapping_edge_intact() {
        let mut life = world_with("Glider", 8, true);
        let shift = |cells: &[(usize, usize)], d: usize| -> Vec<(usize, usize)> {
            let mut out: Vec<_> = cells
                .iter()
                .map(|&(x, y)| ((x + d) % 8, (y + d) % 8))
                .collect();
            out.sort_by_key(|&(x, y)| (y, x));
            out
        };
        let start = live_cells(&life);
        for _ in 0..4 * 8 {
            life.step();
        }
        assert_eq!(live_cells(&life), shift(&start, 8), "back where it started");
        for _ in 0..4 * 3 {
            life.step();
        }
        assert_eq!(
            live_cells(&life),
            shift(&start, 3),
            "3 cells along, across the edge"
        );
    }

    /// The diehard is famous for vanishing completely after exactly 130 generations. That
    /// checks the RLE parser, the pattern's data and the engine together.
    #[test]
    fn diehard_dies_at_generation_130() {
        let mut life = world_with("Diehard", 64, false);
        for _ in 0..129 {
            life.step();
        }
        assert!(life.population() > 0, "still alive at 129");
        life.step();
        assert_eq!(life.population(), 0, "gone at 130");
    }

    /// The Gosper glider gun has period 30 and fires one 5-cell glider each period, so after
    /// it warms up the population rises by exactly 5 every 30 generations (until the gliders
    /// reach the edge).
    #[test]
    fn glider_gun_adds_one_glider_every_30_generations() {
        let mut life = world_with("Gosper glider gun", 200, false);
        let mut counts = Vec::new();
        for g in 0..=150 {
            if g % 30 == 0 {
                counts.push(life.population());
            }
            life.step();
        }
        let steps: Vec<_> = counts
            .windows(2)
            .map(|w| w[1] as isize - w[0] as isize)
            .collect();
        assert!(
            steps[1..].iter().all(|&d| d == 5),
            "population steps {steps:?}"
        );
    }

    /// The first generation (a multiple of 30, up to `limit`) at which the glider gun on the
    /// default 320 x 200 world stops repeating: its own 36 x 9 box differs from 30
    /// generations before. `None` if it is still intact at `limit`.
    fn gun_breaks_at(limit: u64, wrap: bool) -> Option<u64> {
        let pattern = rle::PATTERNS
            .iter()
            .find(|p| p.name == "Gosper glider gun")
            .unwrap();
        let mut life = Life::new(320, 200, Rule::life(), wrap);
        life.place_centred(&rle::parse(pattern.rle).unwrap());
        let (x0, y0) = (160 - 18, 100 - 4);
        let snapshot = |life: &Life| -> Vec<bool> {
            (y0..y0 + 9)
                .flat_map(|y| (x0..x0 + 36).map(move |x| (x, y)))
                .map(|(x, y)| life.grid().get(x, y))
                .collect()
        };
        let mut previous = snapshot(&life);
        while life.generation < limit {
            for _ in 0..30 {
                life.step();
            }
            let now = snapshot(&life);
            if now != previous {
                return Some(life.generation);
            }
            previous = now;
        }
        None
    }

    /// The README's first tip: on the default world, with wrap on, the gun's own gliders
    /// come round and wreck it (at generation 6,390); with wrap off they die at the edge and
    /// the gun runs on (checked to 20,000). Slow, so ignored by default: run `make slow`.
    /// If this number changes, update `life/README.md`, which quotes it.
    #[test]
    #[ignore = "slow: run with --release -- --ignored"]
    fn wrapping_wrecks_the_glider_gun_and_a_dead_edge_does_not() {
        assert_eq!(gun_breaks_at(20_000, true), Some(6_390));
        assert_eq!(gun_breaks_at(20_000, false), None);
    }

    /// The pulsar is a period-3 oscillator: it repeats after 3 generations and not before.
    #[test]
    fn pulsar_has_period_three() {
        let mut life = world_with("Pulsar", 25, false);
        let start = live_cells(&life);
        life.step();
        assert_ne!(live_cells(&life), start);
        life.step();
        life.step();
        assert_eq!(live_cells(&life), start);
    }

    /// A pasted pattern can be bigger than the world. Placing it must keep the part that
    /// fits and drop the rest, not panic on an out-of-range cell.
    #[test]
    fn a_pattern_bigger_than_the_world_is_clipped() {
        let mut life = Life::new(4, 3, Rule::life(), false);
        let shape = rle::parse("10o$10o$10o$10o$10o!").unwrap();
        life.place_centred(&shape);
        assert_eq!(life.population(), 12, "every cell of the 4 x 3 world");
    }

    /// Every built-in pattern must parse, and fit the default world when placed.
    #[test]
    fn every_built_in_pattern_parses() {
        for p in rle::PATTERNS {
            let rle::Shape { cells, size } =
                rle::parse(p.rle).unwrap_or_else(|e| panic!("{}: {e}", p.name));
            assert!(!cells.is_empty(), "{} is empty", p.name);
            assert!(size.0 <= 160 && size.1 <= 100, "{} is too big", p.name);
        }
    }

    /// Not a check: how long one frame's worth of work (50 generations, the per-frame cap)
    /// takes on the largest world, to see whether the cap keeps frames short.
    #[test]
    #[ignore = "timing: run with --release -- --ignored --nocapture"]
    fn timing_50_generations_on_the_largest_world() {
        let mut life = Life::new(640, 400, Rule::life(), true);
        let mut rng = ca_core::Rng::new(1);
        for y in 0..400 {
            for x in 0..640 {
                life.set(x, y, rng.chance(0.3));
            }
        }
        let start = std::time::Instant::now();
        for _ in 0..50 {
            life.step();
        }
        eprintln!("50 generations of 640 x 400: {:?}", start.elapsed());
    }
}
