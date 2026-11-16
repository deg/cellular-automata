//! The cyclic cellular automaton (CCA), with range, threshold, colours and neighbourhood.
//!
//! Each cell holds one of `colours` colours, 0 to `colours - 1`, arranged in a wheel: the
//! successor of colour k is k + 1, and the successor of the last colour is 0. Every
//! generation, all cells update at once:
//! - a cell of colour k advances to k + 1 (mod `colours`) if at least `threshold` cells
//!   within `range` of it already have colour k + 1;
//! - otherwise it keeps its colour.
//!
//! This is the rule (CCA) of Fisch, Gravner and Griffeath, "Threshold-range scaling of
//! excitable cellular automata", Statistics and Computing 1 (1991) 23-39, section 1.
//! "Within range" is the box (Moore) or the diamond (von Neumann) of that radius around the
//! cell, as in their paper and in MCell's Cyclic CA family. With range 1, the von Neumann
//! neighbourhood and threshold 1, it is the basic CCA of their earlier paper, "Cyclic
//! cellular automata in two dimensions" (1991).

use ca_core::{Grid, Rng};

/// The largest range offered. The paper's EXCITE! program and MCell both stop at 10.
pub const MAX_RANGE: usize = 10;
/// The most colours offered.
pub const MAX_COLOURS: u8 = 32;

/// Padding beyond an edge that doesn't wrap: no colour, so it never counts.
const NOTHING: u8 = u8::MAX;

/// Which cells are "within range".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Neighbourhood {
    /// The box: every cell at most `range` steps away across, down or diagonally.
    Moore,
    /// The diamond: every cell reached in at most `range` steps north, south, east or west.
    VonNeumann,
}

impl Neighbourhood {
    /// How far the neighbourhood reaches across, in the row `dy` rows above or below the cell.
    fn half_width(self, range: usize, dy: usize) -> usize {
        match self {
            Self::Moore => range,
            Self::VonNeumann => range - dy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    pub range: usize,
    pub threshold: usize,
    pub colours: u8,
    pub neighbourhood: Neighbourhood,
}

impl Rule {
    pub const fn new(
        range: usize,
        threshold: usize,
        colours: u8,
        neighbourhood: Neighbourhood,
    ) -> Self {
        Self {
            range,
            threshold,
            colours,
            neighbourhood,
        }
    }

    /// How many cells a cell's neighbourhood holds, not counting the cell itself: the
    /// largest threshold that can ever be met.
    pub fn neighbours(self) -> usize {
        let r = self.range;
        match self.neighbourhood {
            Neighbourhood::Moore => (2 * r + 1) * (2 * r + 1) - 1,
            Neighbourhood::VonNeumann => 2 * r * (r + 1),
        }
    }

    /// The rule in MCell's notation, such as `R1/T3/C4/NM`.
    pub fn notation(self) -> String {
        let n = match self.neighbourhood {
            Neighbourhood::Moore => "NM",
            Neighbourhood::VonNeumann => "NN",
        };
        format!("R{}/T{}/C{}/{n}", self.range, self.threshold, self.colours)
    }
}

/// A named rule from the sources.
pub struct Preset {
    pub name: &'static str,
    pub rule: Rule,
    /// Where the rule was published first, if before MCell, and whom MCell's list credits
    /// it to ("A rule by X"), which is not always who found it.
    pub credit: &'static str,
    /// What to look for.
    pub note: &'static str,
}

use Neighbourhood::{Moore, VonNeumann};

/// Named rules. Every one is in MCell's rule list (<https://mcell.ca/app/data/rules.txt>),
/// whose names they mostly use. "FGG 1991b" is Fisch, Gravner and Griffeath,
/// "Threshold-range scaling of excitable cellular automata", where several appeared first.
pub const PRESETS: &[Preset] = &[
    Preset {
        name: "CCA",
        rule: Rule::new(1, 1, 14, VonNeumann),
        credit: "Fisch, Gravner and Griffeath (1989-91). MCell's \"CCA\", credited to David Griffeath.",
        note: "The basic rule. Droplets of waves eat the noise, then spirals appear and slowly take over.",
    },
    Preset {
        name: "Perfect",
        rule: Rule::new(1, 3, 4, Moore),
        credit: "FGG 1991b, Plate H. MCell's \"Perfect\", credited to David Griffeath.",
        note: "A long, churning soup first; then perfect spiral cores appear and take over. Use a big world.",
    },
    Preset {
        name: "Imperfect",
        rule: Rule::new(1, 2, 4, Moore),
        credit: "FGG 1991b, section 10.3. MCell's \"Imperfect\", credited to John Elliott.",
        note: "Spirals form quickly. Later, a small structure that cycles on its own may appear and suddenly take over.",
    },
    Preset {
        name: "313",
        rule: Rule::new(1, 3, 3, Moore),
        credit: "FGG 1991b, section 7, as a rule that makes spirals. MCell's \"313\", credited to David Griffeath.",
        note: "Only three colours, and still spirals.",
    },
    Preset {
        name: "Cyclic spirals",
        rule: Rule::new(3, 5, 8, Moore),
        credit: "MCell's \"Cyclic spirals\", credited to David Griffeath.",
        note: "A larger range: broad, smooth spirals.",
    },
    Preset {
        name: "Turbulent phase",
        rule: Rule::new(2, 5, 8, Moore),
        credit: "FGG 1991b, Plate F. MCell's \"Turbulent phase\", credited to David Griffeath.",
        note: "Would-be spirals keep unravelling. On small worlds it often freezes instead.",
    },
    Preset {
        name: "Lava lamp",
        rule: Rule::new(2, 10, 3, Moore),
        credit: "FGG 1991b, section 10.2, the \"lava light\". MCell's \"LavaLamp\", credited to Jason Rampe.",
        note: "Blobs that separate and join, like a lava lamp.",
    },
    Preset {
        name: "Stripes",
        rule: Rule::new(3, 4, 5, VonNeumann),
        credit: "MCell's \"Stripes\", credited to Mirek Wójtowicz.",
        note: "As the name says: the waves line up in stripes.",
    },
    Preset {
        name: "Fossil debris",
        rule: Rule::new(2, 9, 4, Moore),
        credit: "MCell's \"Fossil debris\", credited to David Griffeath.",
        note: "Range 2, threshold 9, four colours.",
    },
    Preset {
        name: "Bootstrap fixation",
        rule: Rule::new(2, 5, 3, VonNeumann),
        credit: "FGG 1991b, Plate G. MCell's \"Cubism\", credited to Jason Rampe.",
        note: "The threshold is too high to keep going: the world freezes.",
    },
];

pub struct Cyclic {
    grid: Grid<u8>,
    pub rule: Rule,
    pub wrap: bool,
    pub generation: u64,
    changed: usize,
    // Scratch space, reused every generation.
    padded: Vec<u8>,
    prefix: Vec<u32>,
    order: Vec<u32>,
    starts: Vec<usize>,
}

impl Cyclic {
    /// A world of colour 0 everywhere, which never changes until something is painted.
    pub fn new(width: usize, height: usize, rule: Rule, wrap: bool) -> Self {
        Self {
            grid: Grid::new(width, height, 0),
            rule,
            wrap,
            generation: 0,
            changed: 0,
            padded: Vec::new(),
            prefix: Vec::new(),
            order: Vec::new(),
            starts: Vec::new(),
        }
    }

    pub fn grid(&self) -> &Grid<u8> {
        &self.grid
    }

    #[cfg(test)]
    pub fn set(&mut self, x: usize, y: usize, colour: u8) {
        self.grid.set(x, y, colour);
    }

    /// Cells that changed colour in the last generation.
    pub fn changed(&self) -> usize {
        self.changed
    }

    /// Paint every cell a colour chosen uniformly at random: the "primordial soup" the
    /// sources start from.
    pub fn fill_random(&mut self, rng: &mut Rng) {
        let n = u64::from(self.rule.colours);
        for c in self.grid.cells_mut() {
            *c = (rng.next_u64() % n) as u8;
        }
    }

    /// Paint random colours in a disc of `radius` around `(x, y)`.
    pub fn paint_noise(&mut self, x: usize, y: usize, radius: usize, rng: &mut Rng) {
        let n = u64::from(self.rule.colours);
        self.paint_disc(x, y, radius, || (rng.next_u64() % n) as u8);
    }

    /// Paint one colour in a disc of `radius` around `(x, y)`.
    pub fn paint_solid(&mut self, x: usize, y: usize, radius: usize, colour: u8) {
        self.paint_disc(x, y, radius, || colour);
    }

    fn paint_disc(&mut self, x: usize, y: usize, radius: usize, mut colour: impl FnMut() -> u8) {
        let r = radius as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                if let Some((px, py)) = self.grid.offset(x, y, dx, dy) {
                    let c = colour();
                    self.grid.set(px, py, c);
                }
            }
        }
    }

    /// Roughly how much work a generation takes per cell, in units of one Life cell update
    /// (about 11 ns, as `ca_core` measured for its frame budget), for
    /// [`ca_core::generations_per_frame`]. Measured natively with the `timing` test on
    /// 2026-11-16: about 10 ns a cell, plus 0.5 ns per colour (a pass over the world each)
    /// and 0.85 ns per row of the neighbourhood (two lookups each). The formula gives 1.8
    /// Life cells for the basic CCA (measured 1.5 to 1.7) and 4.3 at range 10 with 32
    /// colours (measured 4.2), rounded up to whole units.
    pub fn work_per_cell(&self) -> usize {
        let rows = 2 * self.rule.range + 1;
        (20 + usize::from(self.rule.colours) + 2 * rows).div_ceil(22)
    }

    /// One generation. Returns how many cells changed.
    ///
    /// Counting each cell's successor-coloured neighbours one by one costs the neighbourhood's
    /// size, up to 440 cells at range 10. Instead, for each colour c in turn, this builds the
    /// running count of c along every row, then every cell whose successor is c reads off its
    /// count with two lookups per row of its neighbourhood. The edges are handled once, by
    /// padding the world with `range` cells on every side: copies from the far side when it
    /// wraps, or [`NOTHING`] when it doesn't.
    pub fn step(&mut self) -> usize {
        let (w, h) = (self.grid.width(), self.grid.height());
        let r = self.rule.range;
        let n = usize::from(self.rule.colours.max(1));
        let (pw, ph) = (w + 2 * r, h + 2 * r);

        // The padded copy of the world.
        let source = |p: usize, len: usize| -> Option<usize> {
            let i = p as isize - r as isize;
            if (0..len as isize).contains(&i) {
                Some(i as usize)
            } else {
                self.wrap.then(|| i.rem_euclid(len as isize) as usize)
            }
        };
        let columns: Vec<Option<usize>> = (0..pw).map(|px| source(px, w)).collect();
        self.padded.clear();
        self.padded.resize(pw * ph, NOTHING);
        for py in 0..ph {
            let Some(sy) = source(py, h) else { continue };
            let row = &self.grid.cells()[sy * w..(sy + 1) * w];
            let out = &mut self.padded[py * pw..(py + 1) * pw];
            for (o, col) in out.iter_mut().zip(&columns) {
                if let Some(sx) = *col {
                    *o = row[sx];
                }
            }
        }

        // The cells grouped by the colour they would advance to (a counting sort).
        let next = |c: u8| (usize::from(c) + 1) % n;
        self.starts.clear();
        self.starts.resize(n + 1, 0);
        for &c in self.grid.cells() {
            self.starts[next(c) + 1] += 1;
        }
        for c in 0..n {
            self.starts[c + 1] += self.starts[c];
        }
        self.order.clear();
        self.order.resize(w * h, 0);
        let mut fill = self.starts.clone();
        for (i, &c) in self.grid.cells().iter().enumerate() {
            let k = next(c);
            self.order[fill[k]] = i as u32;
            fill[k] += 1;
        }

        let stride = pw + 1;
        self.prefix.resize(ph * stride, 0);
        let mut changed = 0;
        for c in 0..n {
            let bucket = &self.order[self.starts[c]..self.starts[c + 1]];
            if bucket.is_empty() {
                continue;
            }
            // prefix[row][x] = cells of colour c among the row's first x padded cells.
            for py in 0..ph {
                let row = &self.padded[py * pw..(py + 1) * pw];
                let out = &mut self.prefix[py * stride..(py + 1) * stride];
                let mut sum = 0;
                out[0] = 0;
                for (o, &v) in out[1..].iter_mut().zip(row) {
                    sum += u32::from(v == c as u8);
                    *o = sum;
                }
            }
            for &i in bucket {
                let i = i as usize;
                let (x, y) = (i % w, i / w);
                let mut count = 0;
                // Padded row y + dy is dy - r rows from the cell; its column x + r is the cell's.
                for dy in 0..=2 * r {
                    let half = self.rule.neighbourhood.half_width(r, dy.abs_diff(r));
                    let base = (y + dy) * stride + x + r;
                    count += self.prefix[base + half + 1] - self.prefix[base - half];
                }
                if count as usize >= self.rule.threshold {
                    self.grid[i] = c as u8;
                    changed += 1;
                }
            }
        }
        self.changed = changed;
        self.generation += 1;
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rule written the slow, obvious way, straight from the definition: count every
    /// cell within range, one at a time.
    fn naive_step(grid: &Grid<u8>, rule: Rule, wrap: bool) -> Grid<u8> {
        let (w, h) = (grid.width() as isize, grid.height() as isize);
        let r = rule.range as isize;
        let mut out = grid.clone();
        for y in 0..h {
            for x in 0..w {
                let c = grid.get(x as usize, y as usize);
                let succ = (c + 1) % rule.colours;
                let mut count = 0;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let inside = match rule.neighbourhood {
                            Moore => true,
                            VonNeumann => dx.abs() + dy.abs() <= r,
                        };
                        if !inside || (dx, dy) == (0, 0) {
                            continue;
                        }
                        let (nx, ny) = (x + dx, y + dy);
                        let (nx, ny) = if wrap {
                            (nx.rem_euclid(w), ny.rem_euclid(h))
                        } else if (0..w).contains(&nx) && (0..h).contains(&ny) {
                            (nx, ny)
                        } else {
                            continue;
                        };
                        count += usize::from(grid.get(nx as usize, ny as usize) == succ);
                    }
                }
                if count >= rule.threshold {
                    out.set(x as usize, y as usize, succ);
                }
            }
        }
        out
    }

    fn soup(w: usize, h: usize, rule: Rule, wrap: bool, seed: u64) -> Cyclic {
        let mut ca = Cyclic::new(w, h, rule, wrap);
        ca.fill_random(&mut Rng::new(seed));
        ca
    }

    /// Runs until a generation changes nothing, up to `limit`; returns that generation.
    fn fixation_time(ca: &mut Cyclic, limit: u64) -> Option<u64> {
        while ca.generation < limit {
            if ca.step() == 0 {
                return Some(ca.generation);
            }
        }
        None
    }

    /// The threshold slider's top end: the box of radius r holds (2r+1)^2 - 1 other cells,
    /// the diamond 2r(r+1). Checked against a direct count of the offsets.
    #[test]
    fn neighbourhood_sizes() {
        for r in 1..=MAX_RANGE {
            let ri = r as isize;
            let offsets = (-ri..=ri).flat_map(|dy| (-ri..=ri).map(move |dx| (dx, dy)));
            let diamond = offsets
                .clone()
                .filter(|&(dx, dy)| (dx, dy) != (0, 0) && dx.abs() + dy.abs() <= ri)
                .count();
            let box_ = offsets.count() - 1;
            assert_eq!(Rule::new(r, 1, 3, Moore).neighbours(), box_);
            assert_eq!(Rule::new(r, 1, 3, VonNeumann).neighbours(), diamond);
        }
        assert_eq!(Rule::new(1, 1, 3, Moore).neighbours(), 8);
        assert_eq!(Rule::new(1, 1, 3, VonNeumann).neighbours(), 4);
    }

    /// The threshold, by hand: a cell of colour 0 with threshold 2 stays put with one
    /// neighbour of colour 1, and advances with two. Neighbours of other colours (here 2,
    /// the cell's predecessor) never count.
    #[test]
    fn a_cell_advances_at_the_threshold() {
        let rule = Rule::new(1, 2, 3, Moore);
        let mut ca = Cyclic::new(5, 5, rule, false);
        for (x, y) in [(1, 1), (3, 1), (1, 3), (3, 3)] {
            ca.set(x, y, 2);
        }
        ca.set(2, 1, 1);
        ca.step();
        assert_eq!(ca.grid().get(2, 2), 0, "one successor neighbour is too few");

        let mut ca = Cyclic::new(5, 5, rule, false);
        ca.set(2, 1, 1);
        ca.set(3, 3, 1);
        ca.step();
        assert_eq!(
            ca.grid().get(2, 2),
            1,
            "two successor neighbours advance it"
        );
    }

    /// Moore includes the diagonals, von Neumann doesn't: one diagonal neighbour of the
    /// successor colour, threshold 1.
    #[test]
    fn diagonals_count_only_in_the_moore_neighbourhood() {
        for (nb, expected) in [(Moore, 1), (VonNeumann, 0)] {
            let mut ca = Cyclic::new(3, 3, Rule::new(1, 1, 5, nb), false);
            ca.set(2, 2, 1);
            ca.step();
            assert_eq!(ca.grid().get(1, 1), expected, "{nb:?}");
        }
    }

    /// At range 2, the corner (2, 2) away is in the box but not the diamond; (2, 0) and
    /// (1, 1) are in both; (3, 0) is in neither.
    #[test]
    fn range_reaches_the_box_and_the_diamond() {
        for (offset, in_box, in_diamond) in [
            ((2, 2), true, false),
            ((2, 0), true, true),
            ((1, 1), true, true),
            ((3, 0), false, false),
        ] {
            for (nb, inside) in [(Moore, in_box), (VonNeumann, in_diamond)] {
                let mut ca = Cyclic::new(9, 9, Rule::new(2, 1, 4, nb), false);
                ca.set(4 + offset.0, 4 + offset.1, 1);
                ca.step();
                assert_eq!(ca.grid().get(4, 4), u8::from(inside), "{offset:?} {nb:?}");
            }
        }
    }

    /// The last colour's successor is 0: the wheel closes. In the row 3, 0, 2 with four
    /// colours, the 3 advances to 0 (its neighbour is 0), while the 0 (no 1 beside it) and
    /// the 2 (no 3 beside it) stay.
    #[test]
    fn the_last_colour_advances_to_zero() {
        let mut ca = Cyclic::new(3, 1, Rule::new(1, 1, 4, VonNeumann), false);
        ca.set(0, 0, 3);
        ca.set(1, 0, 0);
        ca.set(2, 0, 2);
        ca.step();
        assert_eq!(ca.grid().cells(), &[0, 0, 2]);
    }

    /// With wrap on, the cell across the edge is a neighbour; with it off, it isn't.
    #[test]
    fn wrap_joins_opposite_edges() {
        for (wrap, expected) in [(true, 1), (false, 0)] {
            let mut ca = Cyclic::new(6, 4, Rule::new(1, 1, 3, VonNeumann), wrap);
            ca.set(5, 2, 1);
            ca.step();
            assert_eq!(ca.grid().get(0, 2), expected, "wrap {wrap}");
        }
    }

    /// The defining property of the rule: a cell either keeps its colour or moves to its
    /// successor, never anywhere else, under every preset.
    #[test]
    fn cells_only_ever_move_to_their_successor() {
        for (k, p) in PRESETS.iter().enumerate() {
            let mut ca = soup(64, 48, p.rule, true, k as u64);
            for _ in 0..30 {
                let before = ca.grid().clone();
                ca.step();
                for (&a, &b) in before.cells().iter().zip(ca.grid().cells()) {
                    assert!(
                        b == a || b == (a + 1) % p.rule.colours,
                        "{}: {a} -> {b}",
                        p.name
                    );
                }
            }
        }
    }

    /// The fast count must match the obvious one exactly, for random rules over the whole
    /// range the UI offers (range 1 to 10, 2 to 32 colours, any threshold up to the whole
    /// neighbourhood), both neighbourhoods, both edges, and worlds whose sides are not
    /// multiples of anything, some narrower than the neighbourhood.
    #[test]
    fn fast_step_matches_the_definition() {
        let mut rng = Rng::new(42);
        let mut pick = |lo: usize, hi: usize| lo + (rng.next_u64() % (hi - lo + 1) as u64) as usize;
        let mut moved = 0;
        for trial in 0..100 {
            let range = pick(1, MAX_RANGE);
            let nb = if pick(0, 1) == 0 { Moore } else { VonNeumann };
            let colours = pick(2, usize::from(MAX_COLOURS)) as u8;
            let probe = Rule::new(range, 1, colours, nb);
            // Half the trials keep the threshold near the expected count of next-colour
            // cells, so cells do advance; the rest range up to the whole neighbourhood.
            let most = probe.neighbours();
            let typical = (2 * most / usize::from(colours)).max(1);
            let threshold = pick(1, if trial % 2 == 0 { typical } else { most });
            let rule = Rule { threshold, ..probe };
            let wrap = pick(0, 1) == 0;
            let (w, h) = (pick(11, 40), pick(11, 40));
            let mut ca = soup(w, h, rule, wrap, trial);
            for g in 0..5 {
                let expected = naive_step(ca.grid(), rule, wrap);
                ca.step();
                assert_eq!(
                    ca.grid(),
                    &expected,
                    "trial {trial}, {rule:?}, wrap {wrap}, gen {g}"
                );
            }
            moved += usize::from(ca.generation > 0 && ca.changed() > 0);
        }
        assert!(moved >= 40, "only {moved} trials had cells advancing");
    }

    /// The band test of Fisch, Gravner and Griffeath (1991, section 2), on the box
    /// neighbourhood of range r: horizontal bands of colours 1, 2, ..., N-1 from top to
    /// bottom, each r rows deep, on a background of 0. A cell in the bottom row of a band
    /// sees r full rows of the next band below: r(2r+1) cells of its successor, the most any
    /// cell sees. The paper: "if θ > (2ρ+1)ρ, then the initial configuration never changes";
    /// below the cutoff "a wave propagates upward" with bands of width ρ. At θ = r(2r+1)
    /// exactly, only each band's bottom row (and the row of 0s above the top band) advances,
    /// so the whole stack moves up one row per generation, unchanged.
    #[test]
    fn the_band_test_moves_up_or_freezes() {
        for (r, colours) in [(1, 5), (2, 8), (3, 6)] {
            let (w, h) = (40, 80);
            let band_top = 50;
            let bands = |shift: usize| {
                let mut g = Grid::new(w, h, 0u8);
                for k in 1..colours {
                    for dy in 0..r {
                        let y = band_top - shift + (usize::from(k) - 1) * r + dy;
                        for x in 0..w {
                            g.set(x, y, k);
                        }
                    }
                }
                g
            };
            let cutoff = (2 * r + 1) * r;

            let mut frozen = Cyclic::new(w, h, Rule::new(r, cutoff + 1, colours, Moore), true);
            frozen.grid = bands(0);
            for _ in 0..10 {
                assert_eq!(frozen.step(), 0, "r {r}: above the cutoff nothing moves");
            }

            let mut wave = Cyclic::new(w, h, Rule::new(r, cutoff, colours, Moore), true);
            wave.grid = bands(0);
            for t in 1..=20 {
                wave.step();
                assert_eq!(wave.grid(), &bands(t), "r {r}: generation {t}");
            }
        }
    }

    /// Fisch, Gravner and Griffeath, "Cyclic cellular automata in two dimensions" (1991),
    /// section 4, property (Δ1): in the basic CCA (range 1, von Neumann, threshold 1), a bond
    /// between neighbours whose colours differ by -1, 0 or 1 (mod N) is open, and "an open
    /// bond stays open at all subsequent times".
    #[test]
    fn open_bonds_stay_open_in_the_basic_cca() {
        let n = 14;
        let mut ca = soup(60, 40, Rule::new(1, 1, n, VonNeumann), true, 7);
        let open = |a: u8, b: u8| matches!((b + n - a) % n, 0 | 1) || (a + n - b) % n == 1;
        let bonds = |ca: &Cyclic| -> Vec<bool> {
            let g = ca.grid();
            let (w, h) = (g.width(), g.height());
            let mut out = Vec::new();
            for y in 0..h {
                for x in 0..w {
                    let c = g.get(x, y);
                    out.push(open(c, g.get((x + 1) % w, y)));
                    out.push(open(c, g.get(x, (y + 1) % h)));
                }
            }
            out
        };
        let mut before = bonds(&ca);
        for t in 0..300 {
            ca.step();
            let after = bonds(&ca);
            for (i, (&b, &a)) in before.iter().zip(&after).enumerate() {
                assert!(!b || a, "bond {i} closed at generation {t}");
            }
            before = after;
        }
    }

    /// A clock (Fisch, Gravner and Griffeath 1991, section 4): a loop on which each colour
    /// is followed by its successor. Their example for N = 14 is the boundary of a 5 x 4
    /// rectangle, which has 14 cells. In the basic CCA every clock cell has its successor
    /// next to it, so the clock advances every generation whatever surrounds it.
    #[test]
    fn a_clock_ticks_whatever_surrounds_it() {
        let n = 14;
        let mut ca = soup(30, 30, Rule::new(1, 1, n, VonNeumann), true, 3);
        let mut ring = Vec::new();
        let (x0, y0) = (10, 10);
        ring.extend((0..5).map(|i| (x0 + i, y0)));
        ring.extend((1..4).map(|i| (x0 + 4, y0 + i)));
        ring.extend((0..4).rev().map(|i| (x0 + i, y0 + 3)));
        ring.extend((1..3).rev().map(|i| (x0, y0 + i)));
        assert_eq!(ring.len(), 14);
        // Colours fall along the loop, so each cell's successor is the cell before it.
        for (k, &(x, y)) in ring.iter().enumerate() {
            ca.set(x, y, (n - (k as u8) % n) % n);
        }
        for t in 1..=100u64 {
            ca.step();
            for (k, &(x, y)) in ring.iter().enumerate() {
                let start = u64::from((n - (k as u8) % n) % n);
                assert_eq!(u64::from(ca.grid().get(x, y)), (start + t) % u64::from(n));
            }
        }
    }

    /// Their lemma (section 5): if there is a clock anywhere, every site is eventually
    /// periodic with period N. On a small world one clock soon drives every cell, so every
    /// cell then advances every generation.
    #[test]
    fn a_clock_eventually_drives_every_cell() {
        let n = 8;
        let mut ca = soup(40, 40, Rule::new(1, 1, n, VonNeumann), true, 11);
        // The boundary of a 3 x 3 square has 8 cells.
        let ring = [
            (0, 0),
            (1, 0),
            (2, 0),
            (2, 1),
            (2, 2),
            (1, 2),
            (0, 2),
            (0, 1),
        ];
        for (k, &(dx, dy)) in ring.iter().enumerate() {
            ca.set(18 + dx, 18 + dy, (n - k as u8) % n);
        }
        let cells = 40 * 40;
        let entrained = (0..5_000).any(|_| ca.step() == cells);
        assert!(entrained, "after {} generations", ca.generation);
        for _ in 0..50 {
            assert_eq!(ca.step(), cells);
        }
    }

    /// Shalizi and Shalizi, "Quantifying self-organization in cyclic cellular automata",
    /// arXiv:nlin/0507067 (2005; first in SPIE Proceedings 5114, 2003), section 2: with four colours and the range-1 box, "Fixation occurs with
    /// T ≥ 4", while T = 1 and T = 2 are locally periodic (incoherent oscillations, and
    /// spirals that entrain the whole lattice). Their runs used 100 x 100 with wrap-around.
    #[test]
    fn four_colours_freeze_at_threshold_four_but_not_below() {
        for seed in 0..4 {
            let mut frozen = soup(100, 100, Rule::new(1, 4, 4, Moore), true, seed);
            assert!(
                fixation_time(&mut frozen, 2_000).is_some(),
                "T = 4, seed {seed}"
            );
            for t in [1, 2] {
                let mut live = soup(100, 100, Rule::new(1, t, 4, Moore), true, seed);
                for _ in 0..500 {
                    live.step();
                }
                assert!(
                    live.changed() > 100 * 100 / 10,
                    "T = {t}, seed {seed}: {}",
                    live.changed()
                );
            }
        }
    }

    /// Fisch, Gravner and Griffeath (1991), Plate G: "Fixation of a CCA in the bootstrap
    /// regime" for range 2, von Neumann, threshold 5, three colours.
    #[test]
    fn the_bootstrap_rule_fixates() {
        for seed in 0..4 {
            let mut ca = soup(200, 200, Rule::new(2, 5, 3, VonNeumann), true, seed);
            assert!(fixation_time(&mut ca, 2_000).is_some(), "seed {seed}");
        }
    }

    /// Fisch, Gravner and Griffeath, "Cyclic cellular automata in two dimensions" (1991):
    /// the 14-colour basic CCA on a 256 x 256 torus ends with "every site of the entire
    /// lattice ... part of some demon, cycling through the 14 states in order", reached
    /// "after several thousand more updates". So eventually every cell changes every
    /// generation, and keeps doing so. (Here it took 950 to 1,900 generations for seeds 0
    /// to 2.) Slow: run with `make slow`.
    #[test]
    #[ignore]
    fn the_basic_cca_ends_with_every_cell_cycling() {
        for seed in 0..3 {
            let mut ca = soup(256, 256, Rule::new(1, 1, 14, VonNeumann), true, seed);
            let all = 256 * 256;
            let done = (0..30_000).any(|_| ca.step() == all);
            assert!(
                done,
                "seed {seed}: not all cycling by generation {}",
                ca.generation
            );
            println!("seed {seed}: all cycling at generation {}", ca.generation);
            for _ in 0..100 {
                assert_eq!(ca.step(), all, "seed {seed}: and they keep cycling");
            }
        }
    }

    /// Prints the cost of a generation per cell for a few rules, to calibrate
    /// `work_per_cell`. Run with `cargo test --release -p cyclic timing -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn timing() {
        let extremes = [
            Rule::new(1, 1, 2, VonNeumann),
            Rule::new(1, 1, MAX_COLOURS, VonNeumann),
            Rule::new(MAX_RANGE, 100, 3, Moore),
            Rule::new(MAX_RANGE, 100, MAX_COLOURS, Moore),
        ];
        let rules = PRESETS
            .iter()
            .map(|p| (p.name, p.rule))
            .chain(extremes.map(|r| ("extreme", r)));
        for (name, rule) in rules {
            let mut ca = soup(640, 400, rule, true, 1);
            // The quickest of several runs: the machine's other work only ever slows one.
            let gens = 10;
            let ns = (0..5)
                .map(|_| {
                    let start = std::time::Instant::now();
                    for _ in 0..gens {
                        ca.step();
                    }
                    start.elapsed().as_nanos() as f64 / (gens as f64 * 640.0 * 400.0)
                })
                .fold(f64::INFINITY, f64::min);
            println!(
                "{:20} {:14} {ns:5.1} ns/cell, estimate {} Life cells",
                name,
                rule.notation(),
                ca.work_per_cell()
            );
        }
    }
}
