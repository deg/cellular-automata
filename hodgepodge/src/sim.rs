//! The hodgepodge machine of Martin Gerhardt and Heike Schuster.
//!
//! Each cell holds a state from 0 to `n`: 0 is *healthy*, `n` is *ill*, and every state in
//! between is *infected*, more so the higher it is. Every generation, all cells update at
//! once, from the cell and its neighbours (the 4 sharing an edge, or those and the 4 sharing
//! a corner):
//! - a healthy cell becomes `[A / k1] + [B / k2]`, where A and B are the numbers of infected
//!   and ill neighbours and `[x]` rounds down, each fraction separately;
//! - an infected cell becomes `[S / (A + 1)] + g`, where S is the sum of the states of the
//!   cell and its neighbours, and A + 1 the number of infected cells among them, itself
//!   included;
//! - an ill cell becomes healthy.
//!
//! A result above `n` is `n`. Cells beyond an edge that does not wrap count as nothing: a
//! cell there has fewer neighbours.
//!
//! Sources. The rule is the one A. K. Dewdney gives for Gerhardt and Schuster's machine in
//! "Computer Recreations: The hodgepodge machine makes waves", Scientific American 259,
//! no. 2 (August 1988), 104-107: states 0 to n, `[A/k1] + [B/k2]` for a healthy cell with
//! "a rounding-down process" for each bracket, `[S/A] + g` for an infected one capped at n,
//! ill to healthy, von Neumann or Moore neighbourhoods, and edge cells with "fewer cells in
//! their neighborhood". Gerhardt and Schuster's own paper (Physica D 36, 209-221, 1989,
//! doi:10.1016/0167-2789(89)90081-X) could not be read here (ScienceDirect refused the
//! fetch).
//!
//! Dewdney calls A "the number of infected neighbors" in both rules, which leaves an infected
//! cell with no infected neighbours dividing by zero. Two programs divide by the infected
//! cells *including the cell itself*, A + 1, as here: Gary William Flake's `hp` from The
//! Computational Beauty of Nature ("floor( S / ( Ninf + 1 ) ) + g", its manual page, 1997)
//! and `hodge.c`, Jörg Heitkötter's 1993 program "drastically cut down by John Burkardt"
//! (`B++; nval = S / B + g`, 2007), whose comments and code disagree on which count is which.
//! Others divide differently: Wilensky's NetLogo B-Z Reaction model, as Štys et al. quote it
//! (arXiv:1507.08783), by A + B + 1, ill neighbours included; Olaf Conrad's SAGA tool
//! (2009, citing Dewdney and Gerhardt and Schuster's book Das digitale Universum) by every
//! cell in the block that is not healthy, ill ones and itself included; Cliff Reiter's J
//! version averages the infected cells only, on states scaled to 0..1.
//!
//! The healthy rule's order differs too. Dewdney, Flake and NetLogo divide the infected
//! neighbours by k1 and the ill by k2, as here. Reiter (his `forh1` floors the ill count over
//! his `a`: "a and b are thresholds over which the number of ill and infected neighbors must
//! pass"), Heitkötter and Burkardt (`A = scount(ILL)`, then `A / k1 + B / k2`) and SAGA
//! (`nSick / m_tSick + ...`) put the ill first. Dewdney's order is kept here.

use ca_core::{Grid, Rng};

/// The most states offered: a state fits in a byte.
pub const MAX_N: u8 = 255;

/// Which cells are a cell's neighbours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Neighbourhood {
    /// The 4 cells sharing an edge (von Neumann).
    VonNeumann,
    /// Those and the 4 sharing a corner, 8 in all (Moore).
    Moore,
}

impl Neighbourhood {
    /// The neighbours' offsets (dx, dy).
    fn offsets(self) -> &'static [(isize, isize)] {
        const EDGES: [(isize, isize); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];
        const ALL: [(isize, isize); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        match self {
            Self::VonNeumann => &EDGES,
            Self::Moore => &ALL,
        }
    }

    #[cfg(test)]
    pub fn size(self) -> usize {
        self.offsets().len()
    }
}

/// What a state means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    Healthy,
    /// Infected, at this level (1 to n − 1).
    Infected(u8),
    Ill,
}

/// The four numbers of Gerhardt and Schuster's rule, and the neighbourhood.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rule {
    /// The ill state; the states are 0 to `n`.
    pub n: u8,
    /// Infected neighbours per level of infection a healthy cell catches.
    pub k1: u8,
    /// Ill neighbours per level of infection a healthy cell catches.
    pub k2: u8,
    /// How much an infected cell's level rises each generation, on top of its neighbourhood's.
    pub g: u8,
    pub neighbourhood: Neighbourhood,
}

impl Rule {
    pub const fn new(n: u8, k1: u8, k2: u8, g: u8, neighbourhood: Neighbourhood) -> Self {
        Self {
            n,
            k1,
            k2,
            g,
            neighbourhood,
        }
    }

    pub fn health(self, state: u8) -> Health {
        if state == 0 {
            Health::Healthy
        } else if state >= self.n {
            Health::Ill
        } else {
            Health::Infected(state)
        }
    }

    /// The next state of a cell in `state` whose neighbours have states summing to `sum`,
    /// `infected` of them infected and `ill` of them ill.
    #[inline]
    pub fn apply(self, state: u8, sum: u32, infected: u32, ill: u32) -> u8 {
        let n = u32::from(self.n);
        let next = match self.health(state) {
            Health::Healthy => infected / u32::from(self.k1) + ill / u32::from(self.k2),
            Health::Infected(level) => {
                (u32::from(level) + sum) / (infected + 1) + u32::from(self.g)
            }
            Health::Ill => 0,
        };
        next.min(n) as u8
    }

    /// The next state of a cell in `state` with these neighbours: the rule on one
    /// neighbourhood, written out for the tests to trace by hand.
    #[cfg(test)]
    pub fn next(self, state: u8, neighbours: &[u8]) -> u8 {
        let mut sum = 0;
        let (mut infected, mut ill) = (0, 0);
        for &v in neighbours {
            sum += u32::from(v);
            match self.health(v) {
                Health::Healthy => {}
                Health::Infected(_) => infected += 1,
                Health::Ill => ill += 1,
            }
        }
        self.apply(state, sum, infected, ill)
    }
}

/// A named setting.
pub struct Preset {
    pub name: &'static str,
    pub rule: Rule,
    /// What to look for.
    pub note: &'static str,
}

use Neighbourhood::Moore;

/// Named settings. n = 100, k1 = 2 and k2 = 3 are the values of Gerhardt and Schuster's
/// sample experiment as Dewdney reports it (p. 105), which then varied g from low to high
/// and saw activity die out, then four types of behaviour in turn. He gives no values of g,
/// so these were chosen here by running the rule: the infected share does what he describes
/// for each type (see the slow tests). His sample used the von Neumann neighbourhood on a
/// 20 x 20 grid; with that neighbourhood, k1 = 2 and k2 = 3, this rule died out from random
/// starts at each g tried from 1 to 30 (1, 2, 3, 5, 8, 10, 15, 20, 25 and 30; 20 seeds each,
/// 20 x 20 without wrapping; at g = 40, 13 of the 20 survived 3,000 generations), so the
/// presets use the Moore neighbourhood, with which the sequence he describes appears. The
/// reason: with 4 neighbours, rounding down and k1, k2 of 2 or more, a straight front
/// cannot advance, since the healthy cell ahead of it has exactly 1 infected or 1 ill
/// neighbour, and [1/2] = [1/3] = 0; only cells in a concave corner, with 2 such neighbours,
/// catch it (see `with_4_neighbours_a_straight_front_stands_still`). With k1 = k2 = 1 a
/// single neighbour infects, and the front moves.
pub const PRESETS: &[Preset] = &[
    // Dewdney's Type 2: plateaus of infection about 30 cycles long, punctuated by many
    // healthy cells. Here: plateaus about 26 generations long.
    Preset {
        name: "Plateaus",
        rule: Rule::new(100, 2, 3, 5, Moore),
        note: "Nearly every cell infected, until most of the world falls ill and recovers at once; then it fills up again.",
    },
    // Dewdney's Type 3: a very regular alternation between saturation and virtual
    // disappearance of infected cells every 20 cycles or so. Here: every 14 or so.
    Preset {
        name: "Pulses",
        rule: Rule::new(100, 2, 3, 10, Moore),
        note: "The whole world beats: infection fills it, falls ill and mostly clears, again and again, a beat every 14 generations or so.",
    },
    // Rings, the regular waves of Dewdney's Type 2 and Type 3 pictures, before his Type 4.
    Preset {
        name: "Rings",
        rule: Rule::new(100, 2, 3, 25, Moore),
        note: "Rings of infection spread from scattered centres, as from the sites where a chemical wave starts.",
    },
    // Dewdney's Type 4: within a few cycles the infected share fluctuates with some
    // regularity about a saturation value (his: about 75 percent), and the waves spiral.
    Preset {
        name: "Spirals",
        rule: Rule::new(100, 2, 3, 30, Moore),
        note: "Waves that wind into spirals, turning forever, while the share of infected cells holds steady.",
    },
];

pub struct Hodgepodge {
    grid: Grid<u8>,
    pub rule: Rule,
    pub wrap: bool,
    pub generation: u64,
    /// Cells infected now.
    infected: usize,
    /// The world with a border of one cell, refilled every generation; reused.
    padded: Vec<u8>,
}

impl Hodgepodge {
    /// A world where every cell is healthy, which stays so until something is drawn.
    pub fn new(width: usize, height: usize, rule: Rule, wrap: bool) -> Self {
        Self {
            grid: Grid::new(width, height, 0),
            rule,
            wrap,
            generation: 0,
            infected: 0,
            padded: Vec::new(),
        }
    }

    pub fn grid(&self) -> &Grid<u8> {
        &self.grid
    }

    #[cfg(test)]
    pub fn set(&mut self, x: usize, y: usize, state: u8) {
        self.grid.set(x, y, state);
        self.count();
    }

    /// Cells infected now.
    pub fn infected(&self) -> usize {
        self.infected
    }

    /// Every cell healthy.
    pub fn clear(&mut self) {
        self.grid.cells_mut().fill(0);
        self.infected = 0;
    }

    /// Every cell in a state from 0 to n, each equally likely. Dewdney says only that
    /// Gerhardt and Schuster gave the cells "a random initial configuration of states" (and
    /// his Type 4 graph starts near no infected cells); this uniform start is the demo's
    /// choice.
    pub fn fill_random(&mut self, rng: &mut Rng) {
        let states = u64::from(self.rule.n) + 1;
        for c in self.grid.cells_mut() {
            *c = (rng.next_u64() % states) as u8;
        }
        self.count();
    }

    /// Infect the cells in a disc of `radius` around `(x, y)`, at random levels.
    pub fn paint_infection(&mut self, x: usize, y: usize, radius: usize, rng: &mut Rng) {
        let levels = u64::from(self.rule.n.saturating_sub(1).max(1));
        self.paint_disc(x, y, radius, || 1 + (rng.next_u64() % levels) as u8);
    }

    /// Make the cells in a disc of `radius` around `(x, y)` healthy.
    pub fn paint_healthy(&mut self, x: usize, y: usize, radius: usize) {
        self.paint_disc(x, y, radius, || 0);
    }

    fn paint_disc(&mut self, x: usize, y: usize, radius: usize, mut state: impl FnMut() -> u8) {
        let r = radius as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                if let Some((px, py)) = self.grid.offset(x, y, dx, dy) {
                    let s = state().min(self.rule.n);
                    self.grid.set(px, py, s);
                }
            }
        }
        self.count();
    }

    fn count(&mut self) {
        let n = self.rule.n;
        self.infected = self
            .grid
            .cells()
            .iter()
            .filter(|&&s| s > 0 && s < n)
            .count();
    }

    /// One generation.
    ///
    /// The world is copied into `padded` with a border of one cell: copies from the far side
    /// when the edges wrap, healthy (0, which adds nothing to any count or sum) when they
    /// don't. Then each cell reads its neighbours at fixed offsets, with no edge tests.
    pub fn step(&mut self) {
        let (w, h) = (self.grid.width(), self.grid.height());
        let (pw, ph) = (w + 2, h + 2);
        self.padded.clear();
        self.padded.resize(pw * ph, 0);
        let cells = self.grid.cells();
        for y in 0..h {
            let row = &cells[y * w..(y + 1) * w];
            let out = &mut self.padded[(y + 1) * pw..(y + 2) * pw];
            out[1..=w].copy_from_slice(row);
            if self.wrap {
                out[0] = row[w - 1];
                out[w + 1] = row[0];
            }
        }
        if self.wrap {
            self.padded.copy_within(h * pw..(h + 1) * pw, 0);
            self.padded.copy_within(pw..2 * pw, (h + 1) * pw);
        }

        let rule = self.rule;
        let n = rule.n;
        let offsets: Vec<isize> = rule
            .neighbourhood
            .offsets()
            .iter()
            .map(|&(dx, dy)| dy * pw as isize + dx)
            .collect();
        let padded = &self.padded;
        let mut infected = 0;
        for (y, row) in self.grid.cells_mut().chunks_exact_mut(w).enumerate() {
            let base = (y + 1) * pw + 1;
            for (x, cell) in row.iter_mut().enumerate() {
                let i = base + x;
                let (mut sum, mut inf, mut ill) = (0u32, 0u32, 0u32);
                for &o in &offsets {
                    let v = padded[i.wrapping_add_signed(o)];
                    sum += u32::from(v);
                    inf += u32::from(v > 0 && v < n);
                    ill += u32::from(v >= n);
                }
                let next = rule.apply(padded[i], sum, inf, ill);
                infected += usize::from(next > 0 && next < n);
                *cell = next;
            }
        }
        self.infected = infected;
        self.generation += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Neighbourhood::VonNeumann;

    /// Dewdney's sample values, with a g of our choice.
    fn sample(g: u8, neighbourhood: Neighbourhood) -> Rule {
        Rule::new(100, 2, 3, g, neighbourhood)
    }

    fn soup(w: usize, h: usize, rule: Rule, wrap: bool, seed: u64) -> Hodgepodge {
        let mut hp = Hodgepodge::new(w, h, rule, wrap);
        hp.fill_random(&mut Rng::new(seed));
        hp
    }

    /// The healthy rule by hand: [A/k1] + [B/k2], each bracket rounded down on its own
    /// (Dewdney: "a rounding-down process applied to the fraction it contains").
    #[test]
    fn a_healthy_cell_catches_whole_levels_from_each_kind_of_neighbour() {
        let r = sample(5, Moore);
        // 3 infected, 2 ill, 3 healthy: [3/2] + [2/3] = 1 + 0.
        assert_eq!(r.next(0, &[7, 50, 99, 100, 100, 0, 0, 0]), 1);
        // 5 infected, 3 ill: [5/2] + [3/3] = 2 + 1.
        assert_eq!(r.next(0, &[1, 2, 3, 4, 5, 100, 100, 100]), 3);
        // 1 infected, 2 ill: [1/2] + [2/3] = 0, though 1/2 + 2/3 is more than 1. Rounding
        // the sum instead would infect it.
        assert_eq!(r.next(0, &[60, 100, 100, 0, 0, 0, 0, 0]), 0);
        // The levels of the infected neighbours don't matter, only how many there are.
        assert_eq!(r.next(0, &[1, 1, 0, 0]), r.next(0, &[99, 99, 0, 0]));
        // k1 and k2 are not interchangeable: 3 ill alone give [3/3] = 1, 3 infected
        // alone [3/2] = 1, but 2 ill give 0 where 2 infected give 1.
        assert_eq!(r.next(0, &[100, 100, 0, 0]), 0);
        assert_eq!(r.next(0, &[40, 40, 0, 0]), 1);
    }

    /// A healthy cell among healthy cells stays healthy, so an all-healthy world is still.
    #[test]
    fn health_with_no_infection_around_lasts() {
        let r = sample(30, Moore);
        assert_eq!(r.next(0, &[0; 8]), 0);
        let mut hp = Hodgepodge::new(20, 10, r, true);
        for _ in 0..5 {
            hp.step();
        }
        assert!(hp.grid().cells().iter().all(|&s| s == 0));
    }

    /// The infected rule by hand: [S/(A+1)] + g, S over the cell and its neighbours, A + 1
    /// the infected cells among them, itself included; capped at n.
    #[test]
    fn an_infected_cell_takes_its_neighbourhoods_average_plus_g() {
        let r = sample(5, VonNeumann);
        // S = 40 + 50 + 0 + 100 + 30 = 220 over the 3 infected (40, 50, 30): 73, + 5.
        assert_eq!(r.next(40, &[50, 0, 100, 30]), 78);
        // Alone: its own level, + g.
        assert_eq!(r.next(40, &[0, 0, 0, 0]), 45);
        // Ill neighbours add to the sum but not to the count, pushing it up.
        // S = 10 + 400 = 410 over 1: capped at 100, ill.
        assert_eq!(r.next(10, &[100, 100, 100, 100]), 100);
        // Moore: S = 20 + 8 * 20 = 180 over 9: 20, + 5.
        assert_eq!(sample(5, Moore).next(20, &[20; 8]), 25);
    }

    /// An ill cell recovers, whatever is around it.
    #[test]
    fn an_ill_cell_becomes_healthy() {
        let r = sample(5, Moore);
        assert_eq!(r.next(100, &[100; 8]), 0);
        assert_eq!(r.next(100, &[50; 8]), 0);
        assert_eq!(r.next(100, &[0; 8]), 0);
    }

    /// One infected cell in a healthy world: it rises by g a generation to n, falls ill,
    /// and recovers; one infected neighbour is too few to infect anyone at k1 = 2.
    #[test]
    fn a_lone_infection_runs_its_course() {
        let mut hp = Hodgepodge::new(9, 9, sample(30, Moore), true);
        hp.set(4, 4, 50);
        let mut seen = vec![];
        for _ in 0..4 {
            hp.step();
            seen.push(hp.grid().get(4, 4));
            assert_eq!(hp.grid().cells().iter().filter(|&&s| s > 0).count(), {
                usize::from(hp.grid().get(4, 4) > 0)
            });
        }
        assert_eq!(seen, [80, 100, 0, 0]);
    }

    /// With k1 = 1, one infected neighbour infects: the infection spreads to the 8 cells
    /// around a seed in one generation, at level [1/1] = 1.
    #[test]
    fn infection_spreads_where_enough_neighbours_carry_it() {
        let mut hp = Hodgepodge::new(9, 9, Rule::new(100, 1, 1, 10, Moore), false);
        hp.set(4, 4, 50);
        hp.step();
        for y in 3..=5 {
            for x in 3..=5 {
                let want = if (x, y) == (4, 4) { 60 } else { 1 };
                assert_eq!(hp.grid().get(x, y), want, "({x}, {y})");
            }
        }
        assert_eq!(hp.infected(), 9);
    }

    /// Why Dewdney's 4-neighbour sample dies here: a straight front of infection, a full
    /// column of infected cells with healthy cells to its right, gives each healthy cell
    /// beside it 1 infected neighbour with 4 neighbours, and [1/2] = 0, so it never
    /// advances; then the column falls ill, giving each 1 ill neighbour, and [1/3] = 0. With
    /// 8 neighbours each has 3 infected neighbours, [3/2] = 1, and with k1 = 1 one is
    /// enough: in both the front moves.
    #[test]
    fn with_4_neighbours_a_straight_front_stands_still() {
        let spread = |rule: Rule| {
            let mut hp = Hodgepodge::new(12, 8, rule, true);
            for y in 0..8 {
                hp.set(3, y, 50);
            }
            hp.step();
            hp.step();
            hp.step();
            (4..12).any(|x| (0..8).any(|y| hp.grid().get(x, y) > 0))
        };
        assert!(!spread(Rule::new(100, 2, 3, 30, VonNeumann)));
        assert!(spread(Rule::new(100, 2, 3, 30, Moore)));
        assert!(spread(Rule::new(100, 1, 1, 30, VonNeumann)));
    }

    /// Every state stays in 0..=n, and an infected cell never jumps straight to healthy:
    /// S is at least the number of infected cells it is divided by, since each is at least
    /// 1, so the average is at least 1. Checked on random neighbourhoods.
    #[test]
    fn states_stay_in_range_and_infection_never_vanishes_at_once() {
        let mut rng = Rng::new(9);
        for _ in 0..20_000 {
            let n = 2 + (rng.next_u64() % 254) as u8;
            let rule = Rule::new(
                n,
                1 + (rng.next_u64() % 8) as u8,
                1 + (rng.next_u64() % 8) as u8,
                (rng.next_u64() % (u64::from(n) + 1)) as u8,
                if rng.chance(0.5) { Moore } else { VonNeumann },
            );
            let mut cells = [0u8; 9];
            for c in &mut cells {
                *c = (rng.next_u64() % (u64::from(n) + 1)) as u8;
            }
            let k = rule.neighbourhood.size();
            let next = rule.next(cells[0], &cells[1..=k]);
            assert!(next <= n, "{rule:?} {cells:?}");
            if let Health::Infected(_) = rule.health(cells[0]) {
                assert!(next > 0, "{rule:?} {cells:?}");
            }
        }
    }

    /// `step` agrees with the rule applied cell by cell, with neighbours found by
    /// `Grid::offset` (and wrapped by hand): the padding, offsets and edges are right.
    #[test]
    fn step_matches_the_rule_cell_by_cell() {
        for (wrap, neighbourhood, seed) in [
            (true, Moore, 1),
            (false, Moore, 2),
            (true, VonNeumann, 3),
            (false, VonNeumann, 4),
        ] {
            let rule = Rule::new(60, 2, 3, 7, neighbourhood);
            let (w, h) = (13, 7);
            let mut hp = soup(w, h, rule, wrap, seed);
            for _ in 0..3 {
                let before = hp.grid().clone();
                hp.step();
                for y in 0..h {
                    for x in 0..w {
                        let around: Vec<u8> = neighbourhood
                            .offsets()
                            .iter()
                            .filter_map(|&(dx, dy)| {
                                if wrap {
                                    let nx = (x as isize + dx).rem_euclid(w as isize) as usize;
                                    let ny = (y as isize + dy).rem_euclid(h as isize) as usize;
                                    Some(before.get(nx, ny))
                                } else {
                                    before.offset(x, y, dx, dy).map(|(a, b)| before.get(a, b))
                                }
                            })
                            .collect();
                        let want = rule.next(before.get(x, y), &around);
                        assert_eq!(hp.grid().get(x, y), want, "{wrap} {neighbourhood:?}");
                    }
                }
                let counted = hp.grid().cells().iter().filter(|&&s| s > 0 && s < 60);
                assert_eq!(hp.infected(), counted.count());
            }
        }
    }

    /// The rule treats every direction alike: on a wrapped square world, turning the world a
    /// quarter turn and stepping gives the same as stepping and then turning.
    #[test]
    fn a_quarter_turn_commutes_with_a_step() {
        let s = 12;
        let turn = |g: &Grid<u8>| {
            let mut t = Grid::new(s, s, 0);
            for y in 0..s {
                for x in 0..s {
                    t.set(s - 1 - y, x, g.get(x, y));
                }
            }
            t
        };
        for neighbourhood in [Moore, VonNeumann] {
            let rule = Rule::new(100, 2, 3, 12, neighbourhood);
            let mut a = soup(s, s, rule, true, 5);
            let mut b = Hodgepodge::new(s, s, rule, true);
            b.grid = turn(a.grid());
            for _ in 0..10 {
                a.step();
                b.step();
            }
            assert_eq!(turn(a.grid()).cells(), b.grid().cells());
        }
    }

    /// The share of infected cells after each of `gens` generations.
    fn shares(hp: &mut Hodgepodge, gens: usize) -> Vec<f64> {
        let cells = hp.grid().len() as f64;
        (0..gens)
            .map(|_| {
                hp.step();
                hp.infected() as f64 / cells
            })
            .collect()
    }

    /// Dewdney (p. 105): at low g, "apart from a few initial fluctuations, activity among
    /// the cells tended to die out; the cells became boringly and everlastingly healthy".
    /// At g = 1 on the smallest world offered (wrapped), every seed tried here died out
    /// within about 230 generations. Slow: `make slow`.
    #[test]
    #[ignore]
    fn at_low_g_the_infection_dies_out() {
        for seed in 0..5 {
            let mut hp = soup(160, 100, sample(1, Moore), true, seed);
            let died = (0..1_000).any(|_| {
                hp.step();
                hp.grid().cells().iter().all(|&s| s == 0)
            });
            assert!(died, "seed {seed}");
        }
    }

    /// Dewdney's Type 3 (p. 105): "a very regular alternation between saturation and
    /// virtual disappearance of infected cells every 20 cycles or so". The "Pulses" preset,
    /// on his 20 x 20 grid without wrapping and on the demo's default world: after 1,000
    /// generations the share swings from above 90% to below 15% (20 x 20) or 25%
    /// (320 x 200, where the troughs were about 18%) and back, with the beat between 10 and
    /// 30 generations. Slow: `make slow`.
    #[test]
    #[ignore]
    fn the_pulses_preset_alternates_regularly() {
        let rule = PRESETS.iter().find(|p| p.name == "Pulses").unwrap().rule;
        for (w, h, wrap, seed, trough) in [
            (20, 20, false, 0, 0.15),
            (20, 20, false, 1, 0.15),
            (20, 20, false, 2, 0.15),
            (320, 200, true, 0, 0.25),
            (320, 200, true, 1, 0.25),
        ] {
            let mut hp = soup(w, h, rule, wrap, seed);
            shares(&mut hp, 1_000);
            let s = shares(&mut hp, 300);
            let (lo, hi) = (
                s.iter().copied().fold(1.0, f64::min),
                s.iter().copied().fold(0.0, f64::max),
            );
            assert!(
                lo < trough && hi > 0.9,
                "{w} x {h}, seed {seed}: {lo} to {hi}"
            );
            // Beats: rises through the halfway mark.
            let beats = s.windows(2).filter(|p| p[0] < 0.5 && p[1] >= 0.5).count();
            let beat = 300.0 / beats as f64;
            assert!(
                (10.0..30.0).contains(&beat),
                "{w} x {h}, seed {seed}: beat {beat}"
            );
        }
    }

    /// Dewdney's Type 2 (p. 105): "a generally regular series of infection 'plateaus'
    /// roughly 30 cycles long, punctuated by the appearance of large numbers of healthy
    /// cells". The "Plateaus" preset on the demo's default world: after 1,000 generations,
    /// more than 60% of the infected shares are above 0.8, and the share drops below 0.6
    /// once every 20 to 35 generations (here, every 25). Slow: `make slow`.
    #[test]
    #[ignore]
    fn the_plateaus_preset_has_long_plateaus() {
        let rule = PRESETS.iter().find(|p| p.name == "Plateaus").unwrap().rule;
        for seed in 0..3 {
            let mut hp = soup(320, 200, rule, true, seed);
            shares(&mut hp, 1_000);
            let s = shares(&mut hp, 300);
            let high = s.iter().filter(|&&v| v > 0.8).count() as f64 / s.len() as f64;
            assert!(high > 0.6, "seed {seed}: {high} above 0.8");
            let drops = s.windows(2).filter(|p| p[0] >= 0.6 && p[1] < 0.6).count();
            let every = 300.0 / drops as f64;
            assert!(
                (20.0..35.0).contains(&every),
                "seed {seed}: a drop every {every}"
            );
        }
    }

    /// Dewdney's Type 4 (p. 105): "within a few cycles of start-up the number of infected
    /// cells would fluctuate with some regularity about a saturation value" (his,
    /// "approximately 75 percent"; here about 60, but with k1 and k2 the other way round,
    /// ill neighbours over k1, the share settled at 0.71 to 0.72 at g = 19 and 20 on every
    /// start tried, 0.73 to 0.75 on some at g = 15 to 17). The "Spirals" preset on the
    /// middle world:
    /// after 1,000 generations the share stays within 0.1 of its mean, which is between 0.4
    /// and 0.9. Slow: `make slow`.
    #[test]
    #[ignore]
    fn the_spirals_preset_holds_a_steady_share() {
        let rule = PRESETS.iter().find(|p| p.name == "Spirals").unwrap().rule;
        for seed in 0..3 {
            let mut hp = soup(320, 200, rule, true, seed);
            shares(&mut hp, 1_000);
            let s = shares(&mut hp, 300);
            let mean = s.iter().sum::<f64>() / s.len() as f64;
            assert!((0.4..0.9).contains(&mean), "seed {seed}: mean {mean}");
            assert!(s.iter().all(|v| (v - mean).abs() < 0.1), "seed {seed}");
        }
    }

    /// Prints the cost of a generation per cell, to set the app's work per frame. Run with
    /// `cargo test --release -p hodgepodge timing -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn timing() {
        for p in PRESETS {
            for neighbourhood in [VonNeumann, Moore] {
                let rule = Rule {
                    neighbourhood,
                    ..p.rule
                };
                let mut hp = soup(640, 400, rule, true, 1);
                let start = std::time::Instant::now();
                for _ in 0..100 {
                    hp.step();
                }
                let ns = start.elapsed().as_nanos() as f64 / (100.0 * 640.0 * 400.0);
                println!("{} {neighbourhood:?}: {ns:.2} ns a cell", p.name);
            }
        }
    }
}
