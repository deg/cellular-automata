//! Diffusion-limited aggregation (DLA) on the square lattice.
//!
//! The model of T. A. Witten and L. M. Sander, Physical Review Letters 47, 1400 (1981), as
//! Sander describes it in "Diffusion-limited aggregation: a kinetic critical phenomenon?",
//! Contemporary Physics 41, 203 (2000), sections 1, 5.1 and 7:
//! - the cluster starts as one occupied site, the seed;
//! - a particle is released far away and walks at random, one lattice step at a time to one
//!   of its four nearest neighbours, until it reaches a site next to the cluster (one of
//!   whose four nearest neighbours is occupied);
//! - there it stops and joins the cluster, and the next particle is released.
//!
//! The original was on the square lattice (Sander section 5.1); "nearest-neighbour sites"
//! is the wording of P. Meakin's abstract, Physical Review A 27, 1495 (1983). Meakin also
//! tried sticking probabilities from 0.1 to 1; here a particle arriving next to the cluster
//! sticks with probability `stick` each time, and otherwise walks on. A step towards an
//! occupied site is not taken; another direction is drawn instead (our choice: the papers
//! read don't say).
//!
//! A line seed is a substrate along the bottom row, with the left and right edges joined,
//! as in Meakin's "Diffusion-controlled deposition on fibers and surfaces", Physical
//! Review A 27, 2616 (1983).
//!
//! # Speed-ups
//!
//! Each leaves the distribution of where a particle sticks unchanged, and a test checks it
//! against the plain walk:
//! - **Launch circle.** A particle from far away first meets a circle around the cluster
//!   at a uniformly random point, so it starts there, a few sites outside the cluster's
//!   reach (Sander section 7.1, crediting M. E. Sander). For the line seed: a uniformly
//!   random point on a row a few sites above the highest particle.
//! - **Bringing walkers back.** A walker that wanders far outside is put straight back on
//!   the launch circle where it would first have crossed it, drawn from the exact
//!   distribution, Sander's equation (16) (the method of images; E. Sander, L. M. Sander
//!   and R. M. Ziff, Computers in Physics 8, 420 (1994)). It is the wrapped Cauchy
//!   distribution, and its counterpart for a line is the Cauchy distribution.
//! - **Big steps.** A walker with no cluster within `k` + 1 sites (in the max-norm) can't
//!   stick before it first reaches the square of radius `k` around it, so it jumps straight
//!   to that square's edge, choosing the point with exactly the probability that the plain
//!   walk would reach it there first ([`exit_probabilities`]). The distance to the cluster
//!   comes from a map of each site's distance to the nearest particle, kept up to
//!   [`GAP_CAP`] (a simpler cousin of Ball and Brady's hierarchical maps, Sander section
//!   7.2), or from the cluster's reach when the walker is outside it. Sander credits big
//!   steps outside the cluster to P. Meakin.

use std::f64::consts::{PI, SQRT_2};
use std::sync::OnceLock;

use ca_core::{Grid, Rng};

/// A site no particle has joined.
pub const EMPTY: u32 = u32::MAX;

/// The distance map counts up to this many sites; further is "at least this far".
pub const GAP_CAP: u8 = 15;

/// The largest big step, in sites: the radius of the largest square with a table.
const MAX_JUMP: usize = 64;

/// How far outside the cluster's reach particles start, in sites. Far enough that a
/// launch site, rounded to the lattice, is never next to the cluster.
const LAUNCH_GAP: f64 = 3.0;

/// The chart's fit of the fractal dimension starts at this many particles: smaller
/// clusters are too few sites to be fractal.
pub const FIT_FROM: f64 = 100.0;

/// The four steps of the walk: right, left, down, up.
const STEPS: [(i64, i64); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seed {
    /// One particle in the middle of the world.
    Point,
    /// The whole bottom row, with the left and right edges joined.
    Line,
}

/// A walker's site. It may be outside the world: the walk is on the unbounded lattice.
type Site = (i64, i64);

pub struct Aggregate {
    /// When each site joined: 0 for the seed, k for the k-th particle, or [`EMPTY`].
    arrival: Grid<u32>,
    /// Each site's distance to the nearest particle in the max-norm, up to [`GAP_CAP`].
    gap: Grid<u8>,
    seed: Seed,
    /// The chance that a particle next to the cluster sticks there.
    pub stick: f64,
    rng: Rng,
    /// Particles joined, not counting the seed.
    joined: u64,
    /// Point seed: the furthest particle's distance from the seed. Line seed: the highest
    /// particle's height above the bottom row.
    reach: f64,
    /// Sums over the cluster of x, y and x² + y², relative to the middle, for the radius of
    /// gyration.
    sums: [f64; 3],
    /// (radius of gyration, sites) as the cluster grew, at roughly evenly spaced sizes on a
    /// log scale. Point seed only.
    history: Vec<(f64, f64)>,
    next_record: u64,
    /// Walker moves so far, each a step or a big step: the work done.
    moves: u64,
    /// Whether to take big steps and bring walkers back from nearby. Tests turn it off to
    /// compare with the plain walk.
    fast: bool,
}

impl Aggregate {
    pub fn new(width: usize, height: usize, seed: Seed, stick: f64, rng_seed: u64) -> Self {
        let mut a = Self {
            arrival: Grid::new(width, height, EMPTY),
            gap: Grid::new(width, height, GAP_CAP),
            seed,
            stick,
            rng: Rng::new(rng_seed),
            joined: 0,
            reach: 0.0,
            sums: [0.0; 3],
            history: Vec::new(),
            next_record: 1,
            moves: 0,
            fast: true,
        };
        match seed {
            Seed::Point => a.join((width as i64 / 2, height as i64 / 2), 0),
            Seed::Line => {
                for x in 0..width {
                    a.arrival.set(x, height - 1, 0);
                }
                for y in 0..height {
                    let gap = (height - 1 - y).min(GAP_CAP as usize) as u8;
                    for x in 0..width {
                        a.gap.set(x, y, gap);
                    }
                }
            }
        }
        a
    }

    pub fn arrival(&self) -> &Grid<u32> {
        &self.arrival
    }

    pub fn seed(&self) -> Seed {
        self.seed
    }

    /// Particles joined, not counting the seed.
    pub fn particles(&self) -> u64 {
        self.joined
    }

    /// Point seed: the furthest particle's distance from the seed. Line seed: the highest
    /// particle's height above the line.
    pub fn reach(&self) -> f64 {
        self.reach
    }

    /// The largest reach the world has room for.
    pub fn room(&self) -> f64 {
        let (w, h) = (self.arrival.width(), self.arrival.height());
        match self.seed {
            Seed::Point => (w.min(h) / 2) as f64 - 2.0,
            Seed::Line => h as f64 - 2.0,
        }
    }

    /// Whether the cluster has reached the edge of its world, so no more can join.
    pub fn finished(&self) -> bool {
        self.reach >= self.room()
    }

    /// Walker moves so far, each a step or a big step.
    pub fn moves(&self) -> u64 {
        self.moves
    }

    /// The radius of gyration about the cluster's centre of mass: the root mean square
    /// distance of its sites from their average.
    pub fn radius_of_gyration(&self) -> f64 {
        let n = (self.joined + 1) as f64;
        let [sx, sy, s2] = self.sums;
        (s2 / n - (sx / n).powi(2) - (sy / n).powi(2))
            .max(0.0)
            .sqrt()
    }

    /// (radius of gyration, sites) as the cluster grew. Point seed only.
    pub fn history(&self) -> &[(f64, f64)] {
        &self.history
    }

    /// The fractal dimension measured so far: the slope of ln(sites) against ln(radius of
    /// gyration), fitted by least squares from [`FIT_FROM`] sites on. Returns (slope,
    /// intercept), or `None` until the cluster has grown at least fourfold past
    /// [`FIT_FROM`].
    pub fn dimension(&self) -> Option<(f64, f64)> {
        let points: Vec<(f64, f64)> = self
            .history
            .iter()
            .filter(|&&(r, n)| n >= FIT_FROM && r > 0.0)
            .map(|&(r, n)| (r.ln(), n.ln()))
            .collect();
        let last = points.last()?;
        if points.len() < 5 || last.1 < (4.0 * FIT_FROM).ln() {
            return None;
        }
        let k = points.len() as f64;
        let mx = points.iter().map(|p| p.0).sum::<f64>() / k;
        let my = points.iter().map(|p| p.1).sum::<f64>() / k;
        let sxy: f64 = points.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
        let sxx: f64 = points.iter().map(|p| (p.0 - mx).powi(2)).sum();
        let slope = sxy / sxx;
        Some((slope, my - slope * mx))
    }

    /// Release one particle and walk it until it sticks. Returns where it joined, or `None`
    /// if the cluster has no room left.
    pub fn add_particle(&mut self) -> Option<(usize, usize)> {
        if self.finished() {
            return None;
        }
        let site = self.walk();
        self.join(site, self.joined as u32 + 1);
        Some((site.0 as usize, site.1 as usize))
    }

    /// Release a particle and walk it until it sticks, without adding it: the site where it
    /// would join.
    fn walk(&mut self) -> Site {
        let mut p = self.launch();
        loop {
            p = self.bring_back(p);
            p = self.move_walker(p);
            if self.is_next_to_cluster(p) && self.rng.chance(self.stick) {
                return p;
            }
        }
    }

    /// The middle of the world, where the point seed sits.
    fn middle(&self) -> Site {
        (
            self.arrival.width() as i64 / 2,
            self.arrival.height() as i64 / 2,
        )
    }

    /// Where the launch circle (or row) is: its radius about the middle, or its height
    /// above the bottom row.
    fn launch_distance(&self) -> f64 {
        match self.seed {
            Seed::Point => self.reach + LAUNCH_GAP,
            Seed::Line => self.reach.round() + LAUNCH_GAP,
        }
    }

    /// How far out a walker is: its distance from the middle, or height above the bottom row.
    fn distance_out(&self, p: Site) -> f64 {
        match self.seed {
            Seed::Point => {
                let c = self.middle();
                ((p.0 - c.0) as f64).hypot((p.1 - c.1) as f64)
            }
            Seed::Line => (self.arrival.height() as i64 - 1 - p.1) as f64,
        }
    }

    /// A new walker, at a uniformly random point of the launch circle or row.
    fn launch(&mut self) -> Site {
        let d = self.launch_distance();
        match self.seed {
            Seed::Point => {
                let c = self.middle();
                let angle = 2.0 * PI * self.rng.next_f64();
                (
                    c.0 + (d * angle.cos()).round() as i64,
                    c.1 + (d * angle.sin()).round() as i64,
                )
            }
            Seed::Line => {
                let w = self.arrival.width() as f64;
                let x = (self.rng.next_f64() * w) as i64;
                (x, self.arrival.height() as i64 - 1 - d as i64)
            }
        }
    }

    /// How far out a walker may go before it is brought back to the launch circle or row.
    fn return_distance(&self) -> f64 {
        let launch = self.launch_distance();
        match (self.seed, self.fast) {
            (Seed::Point, true) => (3.0 * launch).max(launch + 24.0),
            (Seed::Point, false) => 8.0 * launch,
            (Seed::Line, true) => launch + 32.0,
            (Seed::Line, false) => launch + 8.0 * self.arrival.width() as f64,
        }
    }

    /// A walker beyond the return distance, moved to where it would first come back to
    /// the launch circle or row; any other walker unchanged.
    fn bring_back(&mut self, p: Site) -> Site {
        let out = self.distance_out(p);
        if out <= self.return_distance() {
            return p;
        }
        let launch = self.launch_distance();
        match self.seed {
            Seed::Point => {
                let c = self.middle();
                let (x, y) = return_to_circle(
                    ((p.0 - c.0) as f64, (p.1 - c.1) as f64),
                    launch,
                    self.rng.next_f64(),
                );
                (c.0 + x.round() as i64, c.1 + y.round() as i64)
            }
            Seed::Line => {
                let w = self.arrival.width() as f64;
                let shift = return_to_line(out - launch, self.rng.next_f64());
                let x = (p.0 as f64 + shift).rem_euclid(w);
                let x = if x.is_finite() { x as i64 } else { 0 };
                (
                    x.min(w as i64 - 1),
                    self.arrival.height() as i64 - 1 - launch as i64,
                )
            }
        }
    }

    /// Whether site `p` is in the world and occupied.
    fn is_occupied(&self, p: Site) -> bool {
        self.inside(p)
            .is_some_and(|(x, y)| self.arrival.get(x, y) != EMPTY)
    }

    /// The world's cell for walker site `p`, if it has one. The line seed's world wraps
    /// left to right, so only its height can be outside.
    fn inside(&self, p: Site) -> Option<(usize, usize)> {
        let (w, h) = (self.arrival.width() as i64, self.arrival.height() as i64);
        let x = match self.seed {
            Seed::Point => p.0,
            Seed::Line => p.0.rem_euclid(w),
        };
        (x >= 0 && x < w && p.1 >= 0 && p.1 < h).then_some((x as usize, p.1 as usize))
    }

    /// Whether a particle at `p` would touch the cluster: one of its four nearest
    /// neighbours is occupied.
    fn is_next_to_cluster(&self, p: Site) -> bool {
        STEPS
            .iter()
            .any(|&(dx, dy)| self.is_occupied((p.0 + dx, p.1 + dy)))
    }

    /// A lower bound on the distance, in the max-norm, from `p` to the nearest particle.
    fn clearance(&self, p: Site) -> i64 {
        let from_map = self
            .inside(p)
            .map_or(0, |(x, y)| i64::from(self.gap.get(x, y)));
        // Every particle is within `reach` of the middle (or of the bottom row), so a walker
        // further out is at least the difference away: by the max-norm, at least that
        // over √2 for the point seed.
        let beyond = self.distance_out(p) - self.reach;
        let from_reach = match self.seed {
            Seed::Point => (beyond / SQRT_2).floor(),
            Seed::Line => beyond.floor(),
        };
        from_map.max(from_reach.max(0.0) as i64)
    }

    /// One move of the walker: a big step if it is far enough from the cluster, else one
    /// step to a random unoccupied neighbour.
    fn move_walker(&mut self, p: Site) -> Site {
        self.moves += 1;
        let mut k = if self.fast {
            (self.clearance(p) - 1).clamp(0, MAX_JUMP as i64) as usize
        } else {
            0
        };
        if self.seed == Seed::Line {
            // The square must not wrap round to overlap itself.
            k = k.min((self.arrival.width() - 1) / 2);
        }
        let q = if k >= 2 {
            let (side, along) = jump(k, &mut self.rng);
            let (k, along) = (k as i64, along);
            match side {
                0 => (p.0 + k, p.1 + along),
                1 => (p.0 - k, p.1 + along),
                2 => (p.0 + along, p.1 + k),
                _ => (p.0 + along, p.1 - k),
            }
        } else {
            loop {
                let (dx, dy) = STEPS[(self.rng.next_u64() >> 62) as usize];
                let q = (p.0 + dx, p.1 + dy);
                if !self.is_occupied(q) {
                    break q;
                }
            }
        };
        match self.seed {
            Seed::Point => q,
            Seed::Line => (q.0.rem_euclid(self.arrival.width() as i64), q.1),
        }
    }

    /// Add a particle at `p`, numbered `index`, and update the distance map, the reach, and
    /// the sums for the radius of gyration.
    fn join(&mut self, p: Site, index: u32) {
        let (x, y) = self.inside(p).expect("particles join inside the world");
        debug_assert_eq!(self.arrival.get(x, y), EMPTY);
        self.arrival.set(x, y, index);
        if index > 0 {
            self.joined += 1;
        }
        let (w, h) = (self.arrival.width() as i64, self.arrival.height() as i64);
        let cap = i64::from(GAP_CAP);
        for dy in -cap..=cap {
            let ny = y as i64 + dy;
            if ny < 0 || ny >= h {
                continue;
            }
            for dx in -cap..=cap {
                let nx = match self.seed {
                    Seed::Point => x as i64 + dx,
                    Seed::Line => (x as i64 + dx).rem_euclid(w),
                };
                if nx < 0 || nx >= w {
                    continue;
                }
                let d = dx.abs().max(dy.abs()) as u8;
                let i = self.gap.index_of(nx as usize, ny as usize);
                let cell = &mut self.gap.cells_mut()[i];
                *cell = (*cell).min(d);
            }
        }
        let out = self.distance_out(p);
        self.reach = self.reach.max(out);
        let c = self.middle();
        let (rx, ry) = ((p.0 - c.0) as f64, (p.1 - c.1) as f64);
        self.sums[0] += rx;
        self.sums[1] += ry;
        self.sums[2] += rx * rx + ry * ry;
        let sites = self.joined + 1;
        if self.seed == Seed::Point && sites >= self.next_record {
            self.history.push((self.radius_of_gyration(), sites as f64));
            self.next_record = (sites + 1).max((sites as f64 * 1.03).ceil() as u64);
        }
    }
}

/// Sander's equation (16): a walker at `from` (relative to the middle) outside the circle
/// of radius `radius` is put on the circle where it first crosses it, drawn with `u`, a
/// uniform random number in [0, 1). The angle it turns through has the wrapped Cauchy
/// distribution with parameter `radius / |from|`.
pub fn return_to_circle(from: (f64, f64), radius: f64, u: f64) -> (f64, f64) {
    let (x0, y0) = from;
    let r0 = x0.hypot(y0);
    let v = (r0 - radius) / (r0 + radius) * (PI * u).tan();
    let d = r0 * (1.0 + v * v);
    (
        radius * ((1.0 - v * v) * x0 - 2.0 * v * y0) / d,
        radius * ((1.0 - v * v) * y0 + 2.0 * v * x0) / d,
    )
}

/// The line seed's counterpart of [`return_to_circle`]: how far sideways a walker
/// `height` above the launch row lands when it first comes back down to it, drawn with
/// `u`. The Cauchy distribution with scale `height` (on a line without wrapping; the
/// caller wraps it round the world).
pub fn return_to_line(height: f64, u: f64) -> f64 {
    height * (PI * (u - 0.5)).tan()
}

/// For a walk starting at the middle of the square of radius `k` (the sites with
/// max(|x|, |y|) ≤ k), the chance that the first site it reaches on the square's edge is
/// (k, j), for j from −(k − 1) to k − 1. (It never reaches a corner first.) By symmetry
/// each side has the same chances, which sum to 1/4.
///
/// The chance is the value at the middle of the discrete harmonic function that is 1 at
/// (k, j) and 0 on the rest of the edge. Separating variables, with sines across the
/// square, gives
///
/// P(k, j) = Σ over odd m < 2k of sin(mπ(j + k) / 2k) · sin(mπ/2) / (2k · cosh(λₘ k)),
///
/// where cosh λₘ = 2 − cos(mπ / 2k) (the discrete Laplacian's eigenvalue for that sine).
pub fn exit_probabilities(k: usize) -> Vec<f64> {
    let n = 2 * k;
    let k_f = k as f64;
    (-(k as i64 - 1)..=(k as i64 - 1))
        .map(|j| {
            (1..n)
                .step_by(2)
                .map(|m| {
                    let theta = m as f64 * PI / n as f64;
                    let lambda = (2.0 - theta.cos()).acosh();
                    (theta * (j as f64 + k_f)).sin() * (theta * k_f).sin()
                        / (2.0 * k_f * (lambda * k_f).cosh())
                })
                .sum()
        })
        .collect()
}

/// The cumulative [`exit_probabilities`] for each square radius up to [`MAX_JUMP`],
/// normalised to end at 1. Built once.
fn exit_tables() -> &'static [Vec<f64>] {
    static TABLES: OnceLock<Vec<Vec<f64>>> = OnceLock::new();
    TABLES.get_or_init(|| {
        (0..=MAX_JUMP)
            .map(|k| {
                if k == 0 {
                    return Vec::new();
                }
                let p = exit_probabilities(k);
                let total: f64 = p.iter().sum();
                let mut sum = 0.0;
                p.iter()
                    .map(|q| {
                        sum += q / total;
                        sum
                    })
                    .collect()
            })
            .collect()
    })
}

/// A big step across the square of radius `k`: which side the walker leaves by (0 right,
/// 1 left, 2 down, 3 up) and where along it, from −(k − 1) to k − 1.
fn jump(k: usize, rng: &mut Rng) -> (u8, i64) {
    let side = (rng.next_u64() >> 62) as u8;
    let table = &exit_tables()[k];
    let u = rng.next_f64();
    let i = table.partition_point(|&c| c <= u).min(table.len() - 1);
    (side, i as i64 - (k as i64 - 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Pearson's chi-square statistic for observed counts against expected ones.
    fn chi_square(observed: &[f64], expected: &[f64]) -> f64 {
        observed
            .iter()
            .zip(expected)
            .map(|(o, e)| (o - e).powi(2) / e)
            .sum()
    }

    /// The chi-square statistic for whether two sets of counts over the same bins come
    /// from the same distribution, and its degrees of freedom.
    fn homogeneity(a: &HashMap<Site, f64>, b: &HashMap<Site, f64>) -> (f64, usize) {
        let (na, nb): (f64, f64) = (a.values().sum(), b.values().sum());
        let mut bins: Vec<&Site> = a.keys().chain(b.keys()).collect();
        bins.sort();
        bins.dedup();
        let mut chi = 0.0;
        for bin in &bins {
            let (oa, ob) = (*a.get(bin).unwrap_or(&0.0), *b.get(bin).unwrap_or(&0.0));
            let total = oa + ob;
            let (ea, eb) = (total * na / (na + nb), total * nb / (na + nb));
            chi += (oa - ea).powi(2) / ea + (ob - eb).powi(2) / eb;
        }
        (chi, bins.len() - 1)
    }

    /// A generous upper limit for a chi-square statistic with `dof` degrees of freedom: the
    /// mean plus five standard deviations. A correct rule fails it about once in a few
    /// thousand runs (the seeds are fixed, so a run that passes always passes).
    fn chi_limit(dof: usize) -> f64 {
        dof as f64 + 5.0 * (2.0 * dof as f64).sqrt()
    }

    /// The exit chances found a different way: relax the harmonic function on the square
    /// (Gauss–Seidel) with 1 at (k, j) on the edge, and read it at the middle.
    fn relaxed_exit(k: i64, j: i64) -> f64 {
        let n = (2 * k + 1) as usize;
        let mut h = vec![0.0f64; n * n];
        let at = |x: i64, y: i64| ((y + k) as usize) * n + (x + k) as usize;
        h[at(k, j)] = 1.0;
        for _ in 0..20_000 {
            for y in -(k - 1)..k {
                for x in -(k - 1)..k {
                    h[at(x, y)] =
                        (h[at(x + 1, y)] + h[at(x - 1, y)] + h[at(x, y + 1)] + h[at(x, y - 1)])
                            / 4.0;
                }
            }
        }
        h[at(0, 0)]
    }

    #[test]
    fn exit_chances_add_up_to_a_quarter_a_side() {
        for k in 1..=MAX_JUMP {
            let total: f64 = exit_probabilities(k).iter().sum();
            assert!((total - 0.25).abs() < 1e-12, "k = {k}: {total}");
        }
        // The smallest square is one step: each neighbour a quarter.
        assert_eq!(exit_probabilities(1).len(), 1);
        assert!((exit_probabilities(1)[0] - 0.25).abs() < 1e-15);
    }

    /// The closed form against relaxation, an independent way to the same harmonic
    /// function.
    #[test]
    fn exit_chances_match_relaxation() {
        for k in 2..=6 {
            let p = exit_probabilities(k as usize);
            for j in -(k - 1)..k {
                let want = relaxed_exit(k, j);
                let got = p[(j + k - 1) as usize];
                assert!(
                    (got - want).abs() < 1e-9,
                    "k = {k}, j = {j}: {got} vs {want}"
                );
            }
        }
    }

    /// A big step must land where the plain walk first leaves the square: compare the
    /// sampled jumps with plain walks from the middle of a square of radius 5.
    #[test]
    fn big_steps_land_where_plain_walks_leave_the_square() {
        let k = 5i64;
        let trials = 40_000;
        let mut rng = Rng::new(11);
        let mut walked: HashMap<Site, f64> = HashMap::new();
        let mut jumped: HashMap<Site, f64> = HashMap::new();
        for _ in 0..trials {
            let mut p = (0i64, 0i64);
            while p.0.abs() < k && p.1.abs() < k {
                let (dx, dy) = STEPS[(rng.next_u64() >> 62) as usize];
                p = (p.0 + dx, p.1 + dy);
            }
            *walked.entry(p).or_default() += 1.0;
            let (side, along) = jump(k as usize, &mut rng);
            let q = match side {
                0 => (k, along),
                1 => (-k, along),
                2 => (along, k),
                _ => (along, -k),
            };
            *jumped.entry(q).or_default() += 1.0;
        }
        assert!(walked.keys().all(|p| p.0.abs() != p.1.abs()), "a corner");
        let (chi, dof) = homogeneity(&walked, &jumped);
        assert_eq!(dof, 4 * (2 * k as usize - 1) - 1);
        assert!(chi < chi_limit(dof), "chi-square {chi:.1} on {dof}");
    }

    /// The walk is unbiased: each of the four steps equally often, and after n steps the
    /// spread along each axis has variance n / 2.
    #[test]
    fn the_walk_is_unbiased() {
        let mut a = Aggregate::new(64, 64, Seed::Point, 1.0, 5);
        a.fast = false;
        let far = (1_000_000i64, 1_000_000i64);
        let mut counts = [0.0; 4];
        for _ in 0..400_000 {
            let q = a.move_walker(far);
            let i = STEPS
                .iter()
                .position(|&(dx, dy)| (far.0 + dx, far.1 + dy) == q)
                .unwrap();
            counts[i] += 1.0;
        }
        let chi = chi_square(&counts, &[100_000.0; 4]);
        assert!(chi < chi_limit(3), "steps {counts:?}");

        let (walks, n) = (4_000, 100);
        let (mut sx, mut sxx, mut syy) = (0.0, 0.0, 0.0);
        for _ in 0..walks {
            let mut p = far;
            for _ in 0..n {
                p = a.move_walker(p);
            }
            let (x, y) = ((p.0 - far.0) as f64, (p.1 - far.1) as f64);
            sx += x;
            sxx += x * x;
            syy += y * y;
        }
        let w = walks as f64;
        // Standard errors: the mean of x is about 7 / √4000 ≈ 0.11; the variance about
        // 50 · √(2 / 4000) ≈ 1.1.
        assert!((sx / w).abs() < 0.6, "mean x {}", sx / w);
        assert!((sxx / w - 50.0).abs() < 6.0, "variance x {}", sxx / w);
        assert!((syy / w - 50.0).abs() < 6.0, "variance y {}", syy / w);
    }

    /// Particles start all round the launch circle alike: count launches in the four
    /// quadrants, each a quarter turn of the next (so the lattice rounds them alike), and
    /// along the line seed's launch row in 16 equal stretches.
    #[test]
    fn launches_are_spread_evenly() {
        for seed in [Seed::Point, Seed::Line] {
            let mut a = Aggregate::new(160, 160, seed, 1.0, 8);
            for _ in 0..200 {
                a.add_particle();
            }
            let c = a.middle();
            let bins = if seed == Seed::Point { 4 } else { 16 };
            let mut counts = vec![0.0; bins];
            for _ in 0..32_000 {
                let p = a.launch();
                let (x, y) = (p.0 - c.0, p.1 - c.1);
                let bin = match seed {
                    Seed::Point if x > 0 && y >= 0 => 0,
                    Seed::Point if x <= 0 && y > 0 => 1,
                    Seed::Point if x < 0 && y <= 0 => 2,
                    Seed::Point => 3,
                    Seed::Line => p.0 as usize / 10,
                };
                counts[bin] += 1.0;
            }
            let expected = vec![32_000.0 / bins as f64; bins];
            let chi = chi_square(&counts, &expected);
            assert!(chi < chi_limit(bins - 1), "{seed:?}: {counts:?}");
        }
    }

    /// Where the first particle sticks, over many tries of a fresh cluster.
    fn first_sticks(seed: Seed, fast: bool, trials: usize) -> HashMap<Site, f64> {
        let mut a = Aggregate::new(33, 33, seed, 1.0, 1);
        a.fast = fast;
        let mut seen = HashMap::new();
        for _ in 0..trials {
            *seen.entry(a.walk()).or_default() += 1.0;
        }
        seen
    }

    /// By symmetry, the first particle sticks to each side of a lone seed equally often,
    /// with the speed-ups and without. A walk drifting one way would fail.
    #[test]
    fn the_first_particle_sticks_to_each_side_of_the_seed_equally() {
        for fast in [true, false] {
            let seen = first_sticks(Seed::Point, fast, 8_000);
            let mut sites: Vec<_> = seen.keys().copied().collect();
            sites.sort();
            assert_eq!(sites, vec![(15, 16), (16, 15), (16, 17), (17, 16)]);
            let counts: Vec<f64> = sites.iter().map(|s| seen[s]).collect();
            let chi = chi_square(&counts, &[2_000.0; 4]);
            assert!(chi < chi_limit(3), "fast {fast}: {seen:?}");
        }
    }

    /// The speed-ups must not change where particles stick. On a fixed lopsided cluster,
    /// compare the sites where walkers stick, with the speed-ups and with the plain walk.
    #[test]
    fn speed_ups_stick_where_the_plain_walk_does() {
        for seed in [Seed::Point, Seed::Line] {
            let mut sites = [HashMap::new(), HashMap::new()];
            for (i, fast) in [true, false].into_iter().enumerate() {
                let mut a = Aggregate::new(24, 24, seed, 1.0, 3);
                // An L of particles off the seed (or two posts on the line): a cluster
                // with no symmetry to hide a bias behind.
                let shape: &[Site] = match seed {
                    Seed::Point => &[(13, 12), (14, 12), (15, 12), (15, 11), (15, 10), (12, 13)],
                    Seed::Line => &[(5, 22), (5, 21), (5, 20), (14, 22), (15, 22)],
                };
                for (n, &p) in shape.iter().enumerate() {
                    a.join(p, n as u32 + 1);
                }
                a.fast = fast;
                for _ in 0..12_000 {
                    *sites[i].entry(a.walk()).or_default() += 1.0;
                }
            }
            let (chi, dof) = homogeneity(&sites[0], &sites[1]);
            assert!(
                chi < chi_limit(dof),
                "{seed:?}: chi-square {chi:.1} on {dof}"
            );
        }
    }

    /// Sander's equation (16) must put a walker where the plain walk first crosses the
    /// circle. Walk from three times the circle's radius until inside it, and compare the
    /// angle turned with the formula's, by the two-sample Kolmogorov–Smirnov statistic.
    ///
    /// The formula is for the continuum. On the lattice, a circle of radius 8 is too small
    /// for it from twice the radius (a distance of 0.089, failing); from three times, or
    /// for a circle of radius 20 from twice, it passes. So walkers are brought back only
    /// from at least three times the launch radius.
    #[test]
    fn walkers_come_back_where_equation_16_says() {
        let (radius, start, far) = (8.0f64, 24i64, 1000.0f64);
        let trials = 3_000;
        let mut rng = Rng::new(21);
        let mut walked = Vec::new();
        let mut formula = Vec::new();
        for _ in 0..trials {
            let mut p = (start, 0i64);
            loop {
                let r = (p.0 as f64).hypot(p.1 as f64);
                if r <= radius {
                    walked.push((p.1 as f64).atan2(p.0 as f64));
                    break;
                }
                if r > far {
                    // One walker in six gets this far. From here the formula's angle is
                    // nearly uniform (its parameter is radius / far = 0.02), so any error
                    // in it hardly shows.
                    let from = (p.0 as f64, p.1 as f64);
                    let (x, y) = return_to_circle(from, radius, rng.next_f64());
                    walked.push(y.atan2(x));
                    break;
                }
                let (dx, dy) = STEPS[(rng.next_u64() >> 62) as usize];
                p = (p.0 + dx, p.1 + dy);
            }
            let (x, y) = return_to_circle((start as f64, 0.0), radius, rng.next_f64());
            formula.push(y.atan2(x));
        }
        walked.sort_by(f64::total_cmp);
        formula.sort_by(f64::total_cmp);
        let mut gap: f64 = 0.0;
        for &v in walked.iter().chain(&formula) {
            let fa = walked.partition_point(|&w| w <= v) as f64 / trials as f64;
            let fb = formula.partition_point(|&w| w <= v) as f64 / trials as f64;
            gap = gap.max((fa - fb).abs());
        }
        // The 0.1% critical value for two samples of 3,000: 1.95 · √(2 / 3000) ≈ 0.050.
        assert!(gap < 0.05, "Kolmogorov–Smirnov distance {gap:.3}");
        // And the formula lands on the circle.
        let (x, y) = return_to_circle((30.0, -40.0), 10.0, 0.3);
        assert!((x.hypot(y) - 10.0).abs() < 1e-12);
    }

    /// Every particle joins next to an earlier one, by a nearest-neighbour (not
    /// diagonal) contact, so the cluster stays in one piece.
    fn check_contacts(a: &Aggregate) {
        let g = a.arrival();
        let (w, h) = (g.width() as i64, g.height() as i64);
        for y in 0..h {
            for x in 0..w {
                let k = g.get(x as usize, y as usize);
                if k == EMPTY || k == 0 {
                    continue;
                }
                let touches = STEPS.iter().any(|&(dx, dy)| {
                    let nx = match a.seed {
                        Seed::Point => x + dx,
                        Seed::Line => (x + dx).rem_euclid(w),
                    };
                    let ny = y + dy;
                    nx >= 0 && nx < w && ny >= 0 && ny < h && g.get(nx as usize, ny as usize) < k
                });
                assert!(touches, "particle {k} at ({x}, {y}) touches no earlier one");
            }
        }
    }

    /// The cluster is connected: a flood fill from the seed through nearest neighbours
    /// reaches every particle.
    fn check_connected(a: &Aggregate) {
        let g = a.arrival();
        let (w, h) = (g.width() as i64, g.height() as i64);
        let mut seen = vec![false; g.len()];
        let mut stack: Vec<(i64, i64)> = (0..g.len())
            .filter(|&i| g.cells()[i] == 0)
            .map(|i| {
                let (x, y) = g.coords_of(i);
                (x as i64, y as i64)
            })
            .collect();
        for &(x, y) in &stack {
            seen[g.index_of(x as usize, y as usize)] = true;
        }
        while let Some((x, y)) = stack.pop() {
            for (dx, dy) in STEPS {
                let nx = if a.seed == Seed::Line {
                    (x + dx).rem_euclid(w)
                } else {
                    x + dx
                };
                let ny = y + dy;
                if nx < 0 || nx >= w || ny < 0 || ny >= h {
                    continue;
                }
                let i = g.index_of(nx as usize, ny as usize);
                if g.cells()[i] != EMPTY && !seen[i] {
                    seen[i] = true;
                    stack.push((nx, ny));
                }
            }
        }
        let occupied = g.cells().iter().filter(|&&k| k != EMPTY).count();
        let reached = seen.iter().filter(|&&s| s).count();
        assert_eq!(reached, occupied);
    }

    #[test]
    fn clusters_stay_connected_and_particles_stick_only_next_to_them() {
        for (seed, stick) in [
            (Seed::Point, 1.0),
            (Seed::Point, 0.2),
            (Seed::Line, 1.0),
            (Seed::Line, 0.2),
        ] {
            let mut a = Aggregate::new(160, 160, seed, stick, 9);
            while a.particles() < 1_500 && a.add_particle().is_some() {}
            assert!(a.particles() > 500, "{seed:?} {stick}: {}", a.particles());
            check_contacts(&a);
            check_connected(&a);
        }
    }

    /// The distance map stays exact: each site's value is its max-norm distance to the
    /// nearest particle, capped.
    #[test]
    fn the_distance_map_is_exact() {
        for seed in [Seed::Point, Seed::Line] {
            let mut a = Aggregate::new(60, 50, seed, 1.0, 4);
            for _ in 0..300 {
                a.add_particle();
            }
            let g = a.arrival();
            let (w, h) = (g.width() as i64, g.height() as i64);
            let particles: Vec<(i64, i64)> = (0..g.len())
                .filter(|&i| g.cells()[i] != EMPTY)
                .map(|i| {
                    let (x, y) = g.coords_of(i);
                    (x as i64, y as i64)
                })
                .collect();
            for y in 0..h {
                for x in 0..w {
                    let d = particles
                        .iter()
                        .map(|&(px, py)| {
                            let dx = (px - x).abs();
                            let dx = if seed == Seed::Line {
                                dx.min(w - dx)
                            } else {
                                dx
                            };
                            dx.max((py - y).abs())
                        })
                        .min()
                        .unwrap()
                        .min(i64::from(GAP_CAP));
                    assert_eq!(
                        i64::from(a.gap.get(x as usize, y as usize)),
                        d,
                        "({x}, {y})"
                    );
                }
            }
        }
    }

    /// Meakin (1983, abstract): a smaller sticking probability gives denser clusters, so a
    /// cluster of the same size is smaller across.
    #[test]
    fn lower_sticking_makes_denser_clusters() {
        let size = |stick: f64| -> f64 {
            (0..4)
                .map(|s| {
                    let mut a = Aggregate::new(200, 200, Seed::Point, stick, s);
                    for _ in 0..1_000 {
                        a.add_particle();
                    }
                    a.radius_of_gyration()
                })
                .sum::<f64>()
                / 4.0
        };
        let (loose, sticky) = (size(0.1), size(1.0));
        assert!(loose < 0.85 * sticky, "{loose:.1} vs {sticky:.1}");
    }

    #[test]
    fn growth_stops_at_the_edge_of_the_world() {
        let mut a = Aggregate::new(40, 40, Seed::Point, 1.0, 2);
        let mut n = 0;
        while a.add_particle().is_some() {
            n += 1;
        }
        assert!(a.finished() && n > 50);
        assert!(a.reach() >= a.room());
        let mut line = Aggregate::new(30, 20, Seed::Line, 1.0, 2);
        while line.add_particle().is_some() {}
        assert!(line.reach() >= 18.0);
    }

    /// The radius of gyration of a known shape: the seed and its four neighbours, a plus
    /// sign, have mean square distance 4 / 5 from their centre.
    #[test]
    fn radius_of_gyration_of_a_plus_sign() {
        let mut a = Aggregate::new(11, 11, Seed::Point, 1.0, 0);
        for (n, p) in [(6, 5), (4, 5), (5, 6), (5, 4)].into_iter().enumerate() {
            a.join(p, n as u32 + 1);
        }
        assert!((a.radius_of_gyration() - 0.8f64.sqrt()).abs() < 1e-12);
    }

    /// The fractal dimension of large clusters. Witten and Sander's clusters are fractal
    /// with dimension near 1.7; sources:
    /// - P. Meakin, Physical Review A 27, 1495 (1983): on lattices, the radius of gyration
    ///   grows as N^β with β ≈ 6/(5d), a dimension of 5/3 in two dimensions;
    /// - S. Tolman and P. Meakin, Physical Review A 40, 428 (1989): 1.715 ± 0.004 off the
    ///   lattice, for clusters up to a million particles;
    /// - L. M. Sander, Contemporary Physics 41, 203 (2000), section 5.1: square-lattice
    ///   clusters of under about 100,000 particles show the same scaling as off-lattice
    ///   ones; the lattice shows only beyond (P. Meakin et al., Physical Review A 35,
    ///   5233 (1987)).
    ///
    /// The mean over twenty clusters of 30,000 particles must lie between 5/3 and 1.719,
    /// widened by three standard errors of the mean.
    #[test]
    #[ignore]
    fn large_clusters_have_the_published_fractal_dimension() {
        let dims: Vec<f64> = (0..20)
            .map(|s| {
                let mut a = Aggregate::new(1000, 1000, Seed::Point, 1.0, 100 + s);
                while a.particles() < 30_000 {
                    a.add_particle().expect("room for 30,000");
                }
                a.dimension().unwrap().0
            })
            .collect();
        let n = dims.len() as f64;
        let mean = dims.iter().sum::<f64>() / n;
        let sd = (dims.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
        let se = sd / n.sqrt();
        println!("dimensions {dims:.3?}: mean {mean:.4}, standard error {se:.4}");
        assert!(
            (5.0 / 3.0 - 3.0 * se..=1.719 + 3.0 * se).contains(&mean),
            "mean {mean:.4} ± {se:.4}"
        );
    }

    /// Not a test: times the growth of a cluster, for the panel's speed settings.
    #[test]
    #[ignore]
    fn timing() {
        for (size, seed) in [(500, Seed::Point), (1000, Seed::Point), (500, Seed::Line)] {
            let mut a = Aggregate::new(size, size, seed, 1.0, 1);
            let t = std::time::Instant::now();
            while a.add_particle().is_some() {}
            let s = t.elapsed().as_secs_f64();
            println!(
                "{size} {seed:?}: {} particles in {s:.2} s, {:.0} per second, {:.0} moves each, dimension {:?}",
                a.particles(),
                a.particles() as f64 / s,
                a.moves() as f64 / a.particles() as f64,
                a.dimension()
            );
        }
    }
}
