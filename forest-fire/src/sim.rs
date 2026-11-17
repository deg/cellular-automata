//! The Drossel-Schwabl forest-fire model, in the limit its theory needs.
//!
//! The model as a cellular automaton (Clar, Drossel and Schwabl, J. Phys.: Condens. Matter 8,
//! 6803 (1996), section I, with immunity g = 0): every site is empty, a tree or a burning
//! tree, and in parallel a burning tree becomes empty; a tree becomes burning if at least
//! one nearest neighbour is burning; a tree ignites with probability f otherwise (lightning);
//! an empty site grows a tree with probability p. The critical state needs a double
//! separation of time scales, a fire burning out much faster than trees grow and trees
//! growing much faster than lightning strikes (their Eq. 1.5). Equations, sections and
//! tables of the review are numbered here as in its arXiv version, cond-mat/9610201, which
//! the README links.
//!
//! The review's section II B 1 simulates that limit directly, and so does Grassberger
//! (New J. Phys. 4, 17 (2002), section II), and so does this: choose exactly θ = p/f sites
//! at random and grow a tree on each that is empty; then strike one random site, and if it
//! holds a tree, burn its whole forest cluster; no tree grows while a fire burns. The only
//! parameter left is θ. Here the cluster burns breadth first, one ring of nearest
//! neighbours per [`Forest::step`], which is rule 2 of the automaton, so the fire can be
//! watched spreading; with growth stopped meanwhile, the cells burnt and the fire's size are
//! the same as burning the cluster at once (Grassberger also burns breadth first).
//!
//! Nearest neighbours are the four sites across an edge. Neither source says "four" in
//! words: the review's mean-field equations count 2d neighbours on a hypercubic lattice
//! ((1 − ρ_f)^(2d) in its Eq. 2.14), and Grassberger's neighbours of site i are i ± 1 and
//! i ± L. The world wraps round as a torus. Grassberger's boundaries are helical (site i + L²
//! is site i, so leaving one row's end leads into the next row): also without edges, and
//! his densities on lattices of different sizes agree to about 10⁻⁴, so the difference
//! doesn't show at the precision tested here.

use ca_core::{Grid, Rng};

/// What a site holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Tree,
    Burning,
}

/// The fire burning now, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fire {
    /// Trees burnt so far, counting the ring burning now.
    size: u64,
    /// Lit, or joined, by hand: kept out of the statistics.
    by_hand: bool,
}

/// What the statistics collect, from fires lit by lightning. The first lightnings after a
/// restart (or a change of θ) are skipped, so the statistics are of the steady state, not
/// of a forest grown from empty: Grassberger (2002, section II) warns that transients
/// distort the tail of the fire-size distribution. (Here the opening's fires are not much
/// bigger than the steady state's largest: at θ = 1,000 on 640 x 400, the largest was
/// 72,380 trees in the first 2,500 lightnings, and 57,182 to 62,674 in later stretches.)
#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    /// Lightnings still to skip before counting starts.
    pub skip: u64,
    /// Lightnings counted, whether they hit a tree or an empty site.
    pub lightnings: u64,
    /// Fires counted: lightnings that hit a tree.
    pub fires: u64,
    /// Trees burnt by the counted fires.
    pub burnt: u64,
    /// The sum of the density measured after each counted lightning's fire (before new trees
    /// grow, as in Grassberger's Table 1).
    pub density_sum: f64,
    pub largest: u64,
    /// Fires by size: `bins[k]` counts sizes in `bin_range(k)`.
    pub bins: Vec<u64>,
}

impl Stats {
    fn new(skip: u64) -> Self {
        Self {
            skip,
            lightnings: 0,
            fires: 0,
            burnt: 0,
            density_sum: 0.0,
            largest: 0,
            bins: Vec::new(),
        }
    }

    /// The mean tree density after a fire, over the counted lightnings.
    pub fn mean_density(&self) -> Option<f64> {
        (self.lightnings > 0).then(|| self.density_sum / self.lightnings as f64)
    }

    /// The fire-size distribution P(s): for each bin with fires in it, a size in the middle
    /// of the bin (geometrically) and the fraction of fires per unit of size there. Bins are
    /// a quarter of a factor of 2 wide, so each holds a similar share of a power law's range;
    /// dividing by the bin's width keeps the slope of P(s) itself.
    pub fn distribution(&self) -> Vec<(f64, f64)> {
        let fires = self.fires.max(1) as f64;
        self.bins
            .iter()
            .enumerate()
            .filter(|&(_, &n)| n > 0)
            .map(|(k, &n)| {
                let r = bin_range(k);
                let width = (r.end - r.start) as f64;
                let middle = (r.start as f64 * (r.end - 1) as f64).sqrt();
                (middle, n as f64 / (width * fires))
            })
            .collect()
    }

    fn record(&mut self, size: Option<u64>, density: f64) {
        if self.skip > 0 {
            self.skip -= 1;
            return;
        }
        self.lightnings += 1;
        self.density_sum += density;
        if let Some(s) = size {
            self.fires += 1;
            self.burnt += s;
            self.largest = self.largest.max(s);
            let k = bin_of(s);
            if self.bins.len() <= k {
                self.bins.resize(k + 1, 0);
            }
            self.bins[k] += 1;
        }
    }
}

/// Bins up to sizes past 2^32 trees, more than any world here holds.
const BINS: usize = 130;

/// The lower edges of the bins: bin k starts at about 2^(k/4), but never at the same whole
/// number twice, so each bin holds at least one size. Computed once: `record` looks a bin
/// up for every fire.
fn bin_starts() -> &'static [u64; BINS + 1] {
    static STARTS: std::sync::OnceLock<[u64; BINS + 1]> = std::sync::OnceLock::new();
    STARTS.get_or_init(|| {
        let mut starts = [1; BINS + 1];
        for i in 1..=BINS {
            starts[i] = (2f64.powf(i as f64 / 4.0).round() as u64).max(starts[i - 1] + 1);
        }
        starts
    })
}

/// The sizes in bin `k`.
pub fn bin_range(k: usize) -> std::ops::Range<u64> {
    let starts = bin_starts();
    starts[k]..starts[k + 1]
}

fn bin_of(size: u64) -> usize {
    // The number of bin starts at or below `size`, less one.
    bin_starts().partition_point(|&start| start <= size) - 1
}

/// A forest on a torus.
pub struct Forest {
    grid: Grid<Cell>,
    /// Trees grown between lightnings: θ = p/f.
    theta: u32,
    rng: Rng,
    /// The sites burning now, and the next ring being gathered.
    front: Vec<usize>,
    next: Vec<usize>,
    fire: Option<Fire>,
    /// When each site last burnt, as a lightning number (0: never), for drawing scars.
    burnt_at: Vec<u64>,
    pub trees: u64,
    /// Trees grown and burnt since the start: `trees` changes by exactly their difference.
    pub grown: u64,
    pub burnt: u64,
    /// Lightning strikes so far.
    pub lightnings: u64,
    pub steps: u64,
    pub stats: Stats,
}

impl Forest {
    /// An empty forest of `width` x `height` sites, with θ = `theta` and random choices from
    /// `seed`.
    pub fn new(width: usize, height: usize, theta: u32, seed: u64) -> Self {
        let mut forest = Self {
            grid: Grid::new(width, height, Cell::Empty),
            theta,
            rng: Rng::new(seed),
            front: Vec::new(),
            next: Vec::new(),
            fire: None,
            burnt_at: vec![0; width * height],
            trees: 0,
            grown: 0,
            burnt: 0,
            lightnings: 0,
            steps: 0,
            stats: Stats::new(0),
        };
        forest.reset_stats();
        forest
    }

    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }

    pub fn theta(&self) -> u32 {
        self.theta
    }

    /// Change θ and start the statistics again: a distribution mixing two θ means nothing.
    pub fn set_theta(&mut self, theta: u32) {
        self.theta = theta;
        self.reset_stats();
    }

    /// Lightnings to skip before counting: enough for each site to be chosen for growth about
    /// ten times, so the forest planted from empty has burnt and regrown several times over.
    fn transient(&self) -> u64 {
        (10 * self.grid.len() as u64).div_ceil(u64::from(self.theta.max(1)))
    }

    pub fn reset_stats(&mut self) {
        self.stats = Stats::new(self.transient());
    }

    pub fn density(&self) -> f64 {
        self.trees as f64 / self.grid.len() as f64
    }

    #[cfg(test)]
    pub fn is_burning(&self) -> bool {
        self.fire.is_some()
    }

    /// The lightning number at which site (x, y) last burnt; 0 if it never has.
    pub fn burnt_at(&self, x: usize, y: usize) -> u64 {
        self.burnt_at[self.grid.index_of(x, y)]
    }

    /// The trees burnt so far by the fire burning now.
    #[cfg(test)]
    pub fn fire_size(&self) -> Option<u64> {
        self.fire.map(|f| f.size)
    }

    /// Work done by the last call of [`Forest::step`], in sites visited: what the UI budgets
    /// per frame.
    pub fn step(&mut self) -> usize {
        self.steps += 1;
        if self.fire.is_some() {
            self.burn_ring()
        } else {
            for _ in 0..self.theta {
                let i = self.random_site();
                self.grow_at(i);
            }
            let i = self.random_site();
            self.strike_at(i, false);
            self.theta as usize + 1
        }
    }

    fn random_site(&mut self) -> usize {
        // The high bits times the count: uniform enough, with no division.
        (((self.rng.next_u64() >> 32) * self.grid.len() as u64) >> 32) as usize
    }

    /// Grow a tree at site `i` if it is empty. Returns whether one grew.
    pub fn grow_at(&mut self, i: usize) -> bool {
        let cell = &mut self.grid.cells_mut()[i];
        if *cell == Cell::Empty {
            *cell = Cell::Tree;
            self.trees += 1;
            self.grown += 1;
            true
        } else {
            false
        }
    }

    /// Lightning at site `i`: if it holds a tree, that tree starts to burn. A lightning that
    /// finds no tree is over at once, and is counted with no fire.
    pub fn strike_at(&mut self, i: usize, by_hand: bool) {
        if !by_hand {
            self.lightnings += 1;
        }
        if self.grid.cells()[i] == Cell::Tree {
            self.ignite(i);
            self.fire = Some(Fire { size: 1, by_hand });
        } else if !by_hand {
            self.stats.record(None, self.density());
        }
    }

    /// Light the tree at (x, y) by hand. If a fire is burning already, the tree joins its
    /// front, and that fire is left out of the statistics. Does nothing to a site without a
    /// tree.
    pub fn light(&mut self, x: usize, y: usize) {
        let i = self.grid.index_of(x, y);
        if self.grid.cells()[i] != Cell::Tree {
            return;
        }
        match &mut self.fire {
            Some(fire) => {
                fire.by_hand = true;
                fire.size += 1;
                self.ignite(i);
            }
            None => self.strike_at(i, true),
        }
    }

    fn ignite(&mut self, i: usize) {
        self.grid.cells_mut()[i] = Cell::Burning;
        self.burnt_at[i] = self.lightnings.max(1);
        self.front.push(i);
    }

    /// One ring of the fire: every burning tree becomes empty, and every tree next to one
    /// catches fire.
    fn burn_ring(&mut self) -> usize {
        let (w, h) = (self.grid.width(), self.grid.height());
        let mut next = std::mem::take(&mut self.next);
        next.clear();
        let work = self.front.len();
        for k in 0..self.front.len() {
            let i = self.front[k];
            self.grid.cells_mut()[i] = Cell::Empty;
            let (x, y) = (i % w, i / w);
            let left = if x == 0 { i + w - 1 } else { i - 1 };
            let right = if x + 1 == w { i + 1 - w } else { i + 1 };
            let up = if y == 0 { i + w * (h - 1) } else { i - w };
            let down = if y + 1 == h { x } else { i + w };
            for n in [left, right, up, down] {
                if self.grid.cells()[n] == Cell::Tree {
                    self.grid.cells_mut()[n] = Cell::Burning;
                    self.burnt_at[n] = self.lightnings.max(1);
                    next.push(n);
                }
            }
        }
        self.trees -= work as u64;
        self.burnt += work as u64;
        self.next = std::mem::replace(&mut self.front, next);
        if self.front.is_empty() {
            let fire = self.fire.take().expect("burning");
            if !fire.by_hand {
                self.stats.record(Some(fire.size), self.density());
            }
        } else if let Some(fire) = &mut self.fire {
            fire.size += self.front.len() as u64;
        }
        work.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A forest drawn as rows of '.' (empty), 'T' (tree) and '*' (burning).
    fn forest(rows: &[&str]) -> Forest {
        let mut f = Forest::new(rows[0].len(), rows.len(), 1, 0);
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if c == 'T' {
                    f.grow_at(f.grid.index_of(x, y));
                }
            }
        }
        f
    }

    fn picture(f: &Forest) -> Vec<String> {
        let g = f.grid();
        (0..g.height())
            .map(|y| {
                (0..g.width())
                    .map(|x| match g.get(x, y) {
                        Cell::Empty => '.',
                        Cell::Tree => 'T',
                        Cell::Burning => '*',
                    })
                    .collect()
            })
            .collect()
    }

    /// The struck tree's cluster found independently, with each tree's distance from the
    /// strike: sweep the whole grid again and again, giving each tree one more than its
    /// nearest labelled neighbour, until nothing changes (no queue, no rings).
    fn cluster_and_distances(f: &Forest, start: usize) -> Vec<Option<u32>> {
        let g = f.grid();
        let (w, h) = (g.width(), g.height());
        let mut dist: Vec<Option<u32>> = vec![None; g.len()];
        dist[start] = Some(0);
        loop {
            let mut changed = false;
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    if g.cells()[i] != Cell::Tree {
                        continue;
                    }
                    for (dx, dy) in [(1, 0), (w - 1, 0), (0, 1), (0, h - 1)] {
                        let n = (y + dy) % h * w + (x + dx) % w;
                        if let Some(d) = dist[n]
                            && dist[i].is_none_or(|di| di > d + 1)
                        {
                            dist[i] = Some(d + 1);
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                return dist;
            }
        }
    }

    /// Growth by hand: a tree grows on an empty site, and a site already holding one is left
    /// as it is and not counted (the review's rule 2 of its simulation method: "grow a tree
    /// at all chosen empty sites").
    #[test]
    fn a_tree_grows_only_on_an_empty_site() {
        let mut f = Forest::new(3, 2, 1, 0);
        assert!(f.grow_at(4));
        assert!(!f.grow_at(4));
        assert!(f.grow_at(0));
        assert_eq!(picture(&f), ["T..", ".T."]);
        assert_eq!((f.trees, f.grown), (2, 2));
    }

    /// Between lightnings exactly θ sites are chosen, and a tree grows on each distinct empty
    /// one: on an empty forest, the trees are the distinct sites chosen. The choices are
    /// replayed from the same seed.
    #[test]
    fn theta_growth_attempts_then_one_strike() {
        let theta = 40;
        let mut f = Forest::new(10, 8, theta, 7);
        let mut replay = Forest::new(10, 8, theta, 7);
        let mut chosen: Vec<usize> = (0..theta).map(|_| replay.random_site()).collect();
        let struck = replay.random_site();
        chosen.sort_unstable();
        chosen.dedup();
        assert!(chosen.len() < theta as usize, "the test wants a repeat");

        f.step();
        assert_eq!(f.lightnings, 1);
        let trees: Vec<usize> = (0..f.grid.len())
            .filter(|&i| f.grid.cells()[i] != Cell::Empty)
            .collect();
        assert_eq!(trees, chosen);
        assert_eq!(f.grown, chosen.len() as u64);
        assert_eq!(f.is_burning(), chosen.contains(&struck));
    }

    /// A hand trace of a fire: each step, the burning trees become empty and their nearest
    /// neighbours across an edge catch fire. The tree touching the cluster only at a corner
    /// survives; the fire crosses the right edge to the left one, as the world wraps; and no
    /// tree grows while it burns.
    #[test]
    fn a_fire_spreads_one_ring_a_step() {
        let mut f = forest(&[
            "TT....T", //
            ".T.T...", //
            ".TTT..T", //
            "....T..", //
        ]);
        f.strike_at(f.grid.index_of(1, 0), false);
        let expect = [
            vec!["T*....T", ".T.T...", ".TTT..T", "....T.."],
            vec!["*.....T", ".*.T...", ".TTT..T", "....T.."],
            vec!["......*", "...T...", ".*TT..T", "....T.."],
            vec![".......", "...T...", "..*T..T", "....T.."],
            vec![".......", "...T...", "...*..T", "....T.."],
            vec![".......", "...*...", "......T", "....T.."],
            vec![".......", ".......", "......T", "....T.."],
        ];
        assert_eq!(picture(&f), expect[0]);
        for (step, want) in expect.iter().enumerate().skip(1) {
            f.step();
            assert_eq!(&picture(&f), want, "after {step} steps");
        }
        assert!(!f.is_burning());
        assert_eq!(f.burnt, 8);
    }

    /// A fire burns exactly the cluster of trees joined edge to edge to the struck one,
    /// ring by ring: each tree burns at the step equal to its distance from the strike. The
    /// cluster and distances are found independently, by relaxing distances over the whole
    /// grid until they stop changing. Forests of several densities, around the percolation
    /// threshold, with many clusters.
    #[test]
    fn a_fire_burns_exactly_its_cluster() {
        for (seed, density) in [(1, 0.45), (2, 0.55), (3, 0.6), (4, 0.7)] {
            let mut f = Forest::new(41, 29, 1, seed);
            let mut rng = Rng::new(seed);
            for i in 0..f.grid.len() {
                if rng.chance(density) {
                    f.grow_at(i);
                }
            }
            let start = (0..f.grid.len())
                .find(|&i| f.grid.cells()[i] == Cell::Tree)
                .unwrap();
            let dist = cluster_and_distances(&f, start);
            let before = f.grid.cells().to_vec();
            f.strike_at(start, false);
            let mut step = 0;
            while f.is_burning() {
                for (i, (&c, &d)) in f.grid.cells().iter().zip(&dist).enumerate() {
                    assert_eq!(
                        c == Cell::Burning,
                        d == Some(step),
                        "seed {seed}, site {i}, step {step}"
                    );
                }
                f.step();
                step += 1;
            }
            let cluster = dist.iter().filter(|d| d.is_some()).count();
            let rings = dist.iter().flatten().max().unwrap() + 1;
            assert_eq!(step, rings);
            assert_eq!(f.burnt, cluster as u64);
            for i in 0..f.grid.len() {
                let want = if dist[i].is_some() {
                    Cell::Empty
                } else {
                    before[i]
                };
                assert_eq!(f.grid.cells()[i], want);
            }
        }
    }

    /// Over any run, trees are conserved: the trees standing are those grown less those
    /// burnt, the counted fires' sizes add up to the trees they burnt, and no site is left
    /// burning between fires.
    #[test]
    fn trees_are_conserved() {
        let mut f = Forest::new(60, 40, 50, 9);
        f.stats.skip = 0;
        let mut fires = 0;
        while f.lightnings < 2_000 || f.is_burning() {
            let was = f.fire_size();
            f.step();
            if was.is_some() && !f.is_burning() {
                fires += 1;
            }
            let standing = f.grid.cells().iter().filter(|&&c| c != Cell::Empty).count();
            assert_eq!(standing as u64, f.trees);
            assert_eq!(f.trees, f.grown - f.burnt);
        }
        assert!(f.grid.cells().iter().all(|&c| c != Cell::Burning));
        assert!(fires > 100);
        assert_eq!(f.stats.burnt, f.burnt);
        assert_eq!(f.stats.fires, fires);
        assert_eq!(f.stats.lightnings, f.lightnings);
        assert_eq!(f.stats.bins.iter().sum::<u64>(), f.stats.fires);
    }

    /// A fire lit by hand burns like any other but stays out of the statistics, and so does a
    /// lightning's fire that a hand-lit tree joins.
    #[test]
    fn fires_lit_by_hand_are_not_counted() {
        let mut f = forest(&["TT.TT.", "......"]);
        f.stats.skip = 0;
        f.light(0, 0);
        assert!(f.is_burning());
        while f.is_burning() {
            f.step();
        }
        assert_eq!((f.burnt, f.stats.fires, f.stats.lightnings), (2, 0, 0));
        f.strike_at(3, false);
        f.light(1, 1); // empty: nothing happens
        f.light(4, 0);
        while f.is_burning() {
            f.step();
        }
        assert_eq!((f.burnt, f.stats.fires, f.stats.lightnings), (4, 0, 0));
        f.strike_at(9, false); // an empty site: a lightning with no fire
        assert_eq!((f.stats.fires, f.stats.lightnings), (0, 1));
    }

    /// The bins cover every size once, in order, each about a quarter of a factor of 2.
    #[test]
    fn bins_cover_every_size_once() {
        assert_eq!(bin_range(0), 1..2);
        for k in 0..80 {
            assert_eq!(bin_range(k).end, bin_range(k + 1).start);
            assert!(!bin_range(k).is_empty());
            for s in bin_range(k).take(3) {
                assert_eq!(bin_of(s), k);
            }
        }
        assert_eq!(bin_range(40).start, 1024);
    }

    /// Run at θ on an `side` x `side` forest past the transient, then for `counted`
    /// lightnings, and return the statistics.
    fn settle(side: usize, theta: u32, counted: u64, seed: u64) -> Stats {
        let mut f = Forest::new(side, side, theta, seed);
        while f.stats.lightnings < counted {
            f.step();
        }
        f.stats.clone()
    }

    /// The mean density after fires at θ = 125 and 1,000, against Grassberger, *Critical
    /// behaviour of the Drossel-Schwabl forest fire model*, New J. Phys. 4, 17 (2002), Table
    /// 1 ("The density is measured after fires got extinct and before new trees are grown"):
    /// 0.379804 ± 0.000008 at θ = 125, L = 2^10, and 0.397570 ± 0.000010 at θ = 1000,
    /// L = 2^11. Ours are far shorter runs, so the tolerance is statistical, 5 × 10⁻⁴; a
    /// fire spreading to the eight nearest sites instead settles near 0.28 (the review's
    /// section II B 2). The test can't tell whether the density is measured after a fire or
    /// before it (the difference, ⟨s⟩/L², about 10⁻⁴, is below the spread between seeds):
    /// it is `Stats::record`, called as each fire ends, that fixes the point.
    #[test]
    #[ignore]
    fn the_density_settles_where_grassberger_measured_it() {
        for (side, theta, counted, want) in [
            (1024, 125, 400_000, 0.379_804),
            (2048, 1000, 100_000, 0.397_570),
        ] {
            let s = settle(side, theta, counted, 11);
            let got = s.mean_density().unwrap();
            eprintln!("θ = {theta}, L = {side}: {got:.5} (Grassberger {want})");
            assert!(
                (got - want).abs() < 0.000_5,
                "θ = {theta}: {got:.5}, Grassberger {want}"
            );
        }
    }

    /// In the steady state every tree burnt was grown first, so the mean fire size over all
    /// lightnings is (1 − ρ) θ, as Grassberger (2002, section II) states for large lattices:
    /// θ attempts, each landing on an empty site with probability 1 − ρ. On a finite
    /// lattice ρ is the density while trees grow, which rises from the density after a fire
    /// to the density before the next, ⟨s⟩/L² higher; so the midpoint is used. This checks
    /// only that the forest is stationary and the bookkeeping right (Grassberger: it tests
    /// "just stationarity and the absence of gross mistakes"): any rule of burning, eight
    /// neighbours included, passes it.
    #[test]
    #[ignore]
    fn the_mean_fire_is_one_minus_the_density_times_theta() {
        let (side, theta) = (512, 500.0);
        let s = settle(side, 500, 200_000, 12);
        let mean = s.burnt as f64 / s.lightnings as f64;
        let rho = s.mean_density().unwrap() + mean / (2 * side * side) as f64;
        let want = (1.0 - rho) * theta;
        eprintln!("mean fire {mean:.2}, (1 − ρ) θ = {want:.2}");
        assert!((mean - want).abs() / want < 0.001, "{mean} against {want}");
    }

    /// Fire sizes follow a power law, P(s) ∼ s^(1−τ), over the sizes well below the cut-off.
    /// Clar, Drossel and Schwabl's Tab. I gives τ = 2.14(3) in two dimensions, but
    /// Grassberger (2002) finds the effective exponent drifting with θ, so the test asks only
    /// for the bounds the review derives (section II A): n(s) "decreases at least like s^(−2)
    /// but not faster than s^(−3)", that is 2 ≤ τ ≤ 3.
    #[test]
    #[ignore]
    fn fire_sizes_follow_a_power_law_within_the_reviews_bounds() {
        let s = settle(1024, 2000, 300_000, 13);
        let points: Vec<(f64, f64)> = s
            .distribution()
            .into_iter()
            .filter(|&(x, _)| (10.0..=200.0).contains(&x))
            .map(|(x, y)| (x.ln(), y.ln()))
            .collect();
        let n = points.len() as f64;
        let (mx, my) = points
            .iter()
            .fold((0.0, 0.0), |(a, b), p| (a + p.0 / n, b + p.1 / n));
        let slope = points.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>()
            / points.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>();
        let tau = 1.0 - slope;
        assert!((2.0..=3.0).contains(&tau), "τ = {tau}");
        eprintln!("effective τ over 10 ≤ s ≤ 200 at θ = 2000: {tau:.3}");
    }
}
