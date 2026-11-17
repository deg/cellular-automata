//! Site percolation on a square grid. Every cell has its own random number, drawn once, and
//! is open when that number is below p. Raising p opens cells in the order of their numbers,
//! so the grid only ever fills in and clusters only ever merge: Newman and Ziff's method
//! (*Efficient Monte Carlo algorithm and high-precision results for percolation*, Physical
//! Review Letters 85, 4104–4107, 2000; in detail in Physical Review E 64, 016706, 2001).
//! Each opened cell joins the clusters of its open neighbours with a weighted union-find,
//! as in their Section II B, so the clusters are known at every p without relabelling.
//!
//! Clusters join across edges only (four neighbours), and the grid's edges do not wrap. A
//! cluster *spans* when it holds a cell of the top row and a cell of the bottom row, the
//! definition Mertens uses for his exact spanning probabilities (*Exact site-percolation
//! probability on the square lattice*, Journal of Physics A 55, 334002, 2022; arXiv:2109.12102).

use ca_core::Rng;

/// The parent of a closed cell.
const CLOSED: u32 = u32::MAX;

/// A cluster's edge flags: it holds a cell of the top row, of the bottom row.
const TOP: u8 = 1;
const BOTTOM: u8 = 2;

/// Samples of the largest cluster's share per unit of p, for the chart.
pub const TICKS: usize = 1_000;

pub struct Percolation {
    width: usize,
    height: usize,
    /// Each cell's random number, in [0, 1). A cell is open when its number is below p.
    number: Vec<f64>,
    /// The cells in the order they open: by their numbers, smallest first.
    order: Vec<u32>,
    /// How many cells are open: the first `opened` of `order`.
    opened: usize,
    p: f64,
    /// Union-find: a closed cell is `CLOSED`, a cluster's root points to itself, and every
    /// other open cell to another cell of its cluster.
    parent: Vec<u32>,
    /// At a root: its cluster's size in cells.
    size: Vec<u32>,
    /// At a root: which of the top and bottom rows its cluster touches.
    edges: Vec<u8>,
    clusters: usize,
    /// The root of the largest cluster, once a cell is open.
    largest: Option<u32>,
    /// The number of the cell whose opening first joined the top row to the bottom: the
    /// grid spans for every p above it.
    spans_at: Option<f64>,
    /// `samples[k]`: the largest cluster's share of the grid at p = k / TICKS, for every
    /// such p up to the current one.
    samples: Vec<f64>,
}

impl Percolation {
    /// A `width` x `height` grid with fresh random numbers from `seed`, every cell closed.
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let number = (0..width * height).map(|_| rng.next_f64()).collect();
        Self::from_numbers(width, height, number)
    }

    /// A grid with the given numbers, row by row, every cell closed. Equal numbers open in
    /// the order of their cells.
    pub fn from_numbers(width: usize, height: usize, number: Vec<f64>) -> Self {
        let n = width * height;
        assert_eq!(number.len(), n);
        assert!(n < CLOSED as usize);
        let mut order: Vec<u32> = (0..n as u32).collect();
        order.sort_unstable_by(|&a, &b| {
            number[a as usize]
                .total_cmp(&number[b as usize])
                .then(a.cmp(&b))
        });
        let mut grid = Self {
            width,
            height,
            number,
            order,
            opened: 0,
            p: 0.0,
            parent: vec![CLOSED; n],
            size: vec![0; n],
            edges: vec![0; n],
            clusters: 0,
            largest: None,
            spans_at: None,
            samples: Vec::with_capacity(TICKS + 1),
        };
        grid.reset();
        grid
    }

    /// Every cell closed again, p = 0, with the same numbers.
    fn reset(&mut self) {
        self.parent.fill(CLOSED);
        self.opened = 0;
        self.p = 0.0;
        self.clusters = 0;
        self.largest = None;
        self.spans_at = None;
        self.samples.clear();
        self.samples.push(0.0);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn p(&self) -> f64 {
        self.p
    }

    /// Open exactly the cells whose numbers are below `p` (clamped to [0, 1]). Raising p
    /// opens more cells; lowering it starts again from a closed grid.
    pub fn set_p(&mut self, p: f64) {
        let p = p.clamp(0.0, 1.0);
        if p < self.p {
            self.reset();
        }
        while self.opened < self.order.len() {
            let i = self.order[self.opened] as usize;
            let u = self.number[i];
            if u >= p {
                break;
            }
            // Every tick up to this cell's number sees the grid as it is before it opens.
            self.sample_up_to(u);
            self.open(i);
            self.opened += 1;
        }
        self.sample_up_to(p);
        self.p = p;
    }

    /// Record the largest cluster's share at every tick k / TICKS not above `p` not yet
    /// recorded.
    fn sample_up_to(&mut self, p: f64) {
        let share = self.largest_size() as f64 / self.number.len().max(1) as f64;
        while self.samples.len() <= TICKS && self.samples.len() as f64 / TICKS as f64 <= p {
            self.samples.push(share);
        }
    }

    fn open(&mut self, i: usize) {
        let (x, y) = (i % self.width, i / self.width);
        self.parent[i] = i as u32;
        self.size[i] = 1;
        self.edges[i] =
            if y == 0 { TOP } else { 0 } | if y + 1 == self.height { BOTTOM } else { 0 };
        self.clusters += 1;
        if self.largest.is_none() {
            self.largest = Some(i as u32);
        }
        if x > 0 {
            self.join(i, i - 1);
        }
        if x + 1 < self.width {
            self.join(i, i + 1);
        }
        if y > 0 {
            self.join(i, i - self.width);
        }
        if y + 1 < self.height {
            self.join(i, i + self.width);
        }
        if self.spans_at.is_none() {
            let root = self.find(i);
            if self.edges[root] == TOP | BOTTOM {
                self.spans_at = Some(self.number[i]);
            }
        }
    }

    /// Merge the clusters of open cell `a` and cell `b`, if `b` is open: the smaller tree
    /// goes under the larger one's root (Newman and Ziff's weighted union).
    fn join(&mut self, a: usize, b: usize) {
        if self.parent[b] == CLOSED {
            return;
        }
        let (mut ra, mut rb) = (self.find(a), self.find(b));
        if ra == rb {
            return;
        }
        if self.size[ra] < self.size[rb] {
            std::mem::swap(&mut ra, &mut rb);
        }
        self.parent[rb] = ra as u32;
        self.size[ra] += self.size[rb];
        self.edges[ra] |= self.edges[rb];
        self.clusters -= 1;
        // The merged cluster is bigger than either part, so if either was the largest, it now
        // is; `size` at a former root still holds that cluster's last size.
        if self.size[ra] >= self.largest.map_or(0, |l| self.size[l as usize]) {
            self.largest = Some(ra as u32);
        }
    }

    /// The root of open cell `i`'s cluster, halving the path on the way.
    fn find(&mut self, mut i: usize) -> usize {
        while self.parent[i] as usize != i {
            let grandparent = self.parent[self.parent[i] as usize];
            self.parent[i] = grandparent;
            i = grandparent as usize;
        }
        i
    }

    /// The root of cell `i`'s cluster (`i` row by row), or `None` if it is closed. Two cells
    /// are in the same cluster when their roots are equal.
    pub fn root_of(&mut self, i: usize) -> Option<usize> {
        (self.parent[i] != CLOSED).then(|| self.find(i))
    }

    /// The size of the cluster of the cell at `(x, y)`, or `None` if it is closed.
    pub fn cluster_size(&mut self, x: usize, y: usize) -> Option<usize> {
        let root = self.root_of(y * self.width + x)?;
        Some(self.size[root] as usize)
    }

    #[cfg(test)]
    pub fn is_open(&self, x: usize, y: usize) -> bool {
        self.parent[y * self.width + x] != CLOSED
    }

    /// The cell's random number: it is open for every p above this.
    #[cfg(test)]
    pub fn number(&self, x: usize, y: usize) -> f64 {
        self.number[y * self.width + x]
    }

    pub fn open_cells(&self) -> usize {
        self.opened
    }

    pub fn clusters(&self) -> usize {
        self.clusters
    }

    /// The root of the largest cluster (ties: the one that reached that size first).
    pub fn largest_root(&self) -> Option<usize> {
        self.largest.map(|l| l as usize)
    }

    pub fn largest_size(&self) -> usize {
        self.largest.map_or(0, |l| self.size[l as usize] as usize)
    }

    /// Whether some cluster joins the top row to the bottom row.
    #[cfg(test)]
    pub fn spans(&self) -> bool {
        self.spans_at.is_some()
    }

    /// The p above which this grid spans, once p has passed it.
    pub fn spans_at(&self) -> Option<f64> {
        self.spans_at
    }

    /// The largest cluster's share of the grid at p = k / TICKS, for k from 0 up to the
    /// current p.
    pub fn samples(&self) -> &[f64] {
        &self.samples
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};

    /// The clusters by a plain breadth-first flood fill over the open cells, independent of
    /// the union-find: each open cell's label (closed cells `usize::MAX`), and each label's
    /// size.
    fn flood_fill(grid: &Percolation) -> (Vec<usize>, Vec<usize>) {
        let (w, h) = (grid.width(), grid.height());
        let mut label = vec![usize::MAX; w * h];
        let mut sizes = Vec::new();
        for start in 0..w * h {
            if label[start] != usize::MAX || !grid.is_open(start % w, start / w) {
                continue;
            }
            let id = sizes.len();
            let mut queue = VecDeque::from([start]);
            label[start] = id;
            let mut count = 0;
            while let Some(i) = queue.pop_front() {
                count += 1;
                let (x, y) = (i % w, i / w);
                let near = [
                    (x > 0).then(|| i - 1),
                    (x + 1 < w).then(|| i + 1),
                    (y > 0).then(|| i - w),
                    (y + 1 < h).then(|| i + w),
                ];
                for j in near.into_iter().flatten() {
                    if label[j] == usize::MAX && grid.is_open(j % w, j / w) {
                        label[j] = id;
                        queue.push_back(j);
                    }
                }
            }
            sizes.push(count);
        }
        (label, sizes)
    }

    /// Whether some flood-fill cluster has a cell in the top row and one in the bottom row.
    fn flood_spans(w: usize, h: usize, label: &[usize]) -> bool {
        let top: std::collections::HashSet<usize> = label[..w]
            .iter()
            .copied()
            .filter(|&l| l != usize::MAX)
            .collect();
        label[(h - 1) * w..].iter().any(|l| top.contains(l))
    }

    /// The union-find's clusters are the flood fill's: the same partition of the open cells,
    /// the same sizes, the same count, the same largest size, and spanning agrees.
    fn assert_matches_flood_fill(grid: &mut Percolation) {
        let (w, h) = (grid.width(), grid.height());
        let (label, sizes) = flood_fill(grid);
        let mut root_to_label = HashMap::new();
        let mut label_to_root = HashMap::new();
        for i in 0..w * h {
            match grid.root_of(i) {
                None => assert_eq!(label[i], usize::MAX, "cell {i} closed in one only"),
                Some(root) => {
                    assert_ne!(label[i], usize::MAX, "cell {i} open in one only");
                    assert_eq!(*root_to_label.entry(root).or_insert(label[i]), label[i]);
                    assert_eq!(*label_to_root.entry(label[i]).or_insert(root), root);
                    let (x, y) = (i % w, i / w);
                    assert_eq!(grid.cluster_size(x, y), Some(sizes[label[i]]));
                }
            }
        }
        assert_eq!(grid.clusters(), sizes.len());
        assert_eq!(
            grid.largest_size(),
            sizes.iter().copied().max().unwrap_or(0)
        );
        if let Some(root) = grid.largest_root() {
            assert_eq!(grid.size[root] as usize, grid.largest_size());
            assert_eq!(
                grid.parent[root] as usize, root,
                "the largest is held at a root"
            );
        }
        assert_eq!(grid.spans(), flood_spans(w, h, &label), "spanning");
        assert_eq!(
            grid.open_cells(),
            label.iter().filter(|&&l| l != usize::MAX).count()
        );
    }

    /// Cluster labelling against an independent flood fill, on grids of several shapes, at p
    /// below, near and above the threshold, reached both by raising p and by lowering it.
    #[test]
    fn clusters_match_a_flood_fill() {
        for (seed, (w, h)) in [(1, 1), (1, 9), (9, 1), (7, 5), (30, 20), (64, 64)]
            .into_iter()
            .enumerate()
        {
            let mut grid = Percolation::new(w, h, seed as u64);
            for p in [0.2, 0.45, 0.55, 0.5927, 0.62, 0.7, 0.9, 0.6, 0.3] {
                grid.set_p(p);
                assert_matches_flood_fill(&mut grid);
            }
        }
    }

    /// Each cell is open exactly when its number is below p.
    #[test]
    fn a_cell_is_open_when_its_number_is_below_p() {
        let mut grid = Percolation::new(20, 10, 3);
        for p in [0.1, 0.5, 0.8, 0.4] {
            grid.set_p(p);
            for y in 0..10 {
                for x in 0..20 {
                    assert_eq!(grid.is_open(x, y), grid.number(x, y) < p);
                }
            }
        }
    }

    /// A grid from a picture: `#` cells open at p = 0.5 (their numbers rise along the
    /// picture's rows, so they open in reading order), `.` cells never.
    fn picture(rows: &[&str]) -> Percolation {
        let (w, h) = (rows[0].len(), rows.len());
        let numbers = rows
            .iter()
            .flat_map(|r| r.chars())
            .enumerate()
            .map(|(i, c)| {
                if c == '#' {
                    0.4 * i as f64 / (w * h) as f64
                } else {
                    0.9
                }
            })
            .collect();
        let mut grid = Percolation::from_numbers(w, h, numbers);
        grid.set_p(0.5);
        grid
    }

    /// A column spans; a row, a diagonal staircase (whose cells touch only at corners) and a
    /// column with a gap do not; a path that winds up and down does.
    #[test]
    fn spanning_needs_an_edge_connected_path_from_top_to_bottom() {
        assert!(picture(&["..#..", "..#..", "..#..", "..#.."]).spans());
        assert!(!picture(&[".....", "#####", ".....", "....."]).spans());
        assert!(!picture(&["#....", ".#...", "..#..", "...#."]).spans());
        assert!(!picture(&["..#..", "..#..", ".....", "..#.."]).spans());
        assert!(picture(&["#.###", "#.#.#", "###.#", "....#"]).spans());
        // One row is both the top and the bottom.
        assert!(picture(&["..#.."]).spans());
        assert!(!picture(&["....."]).spans());
    }

    /// The span is completed by the last cell to open, in the middle of a column whose two
    /// halves opened first, from the top and from the bottom. So the halves' edge flags must
    /// be carried through each merge: the top half alone never touches the bottom row.
    #[test]
    fn a_span_completed_in_the_middle_is_seen_at_that_cells_number() {
        // A 3 x 5 grid; column 1 opens top (0.1, 0.2), bottom (0.3, 0.4), then middle (0.5).
        let mut numbers = vec![0.9; 15];
        for (y, u) in [(0, 0.1), (1, 0.2), (4, 0.3), (3, 0.4), (2, 0.5)] {
            numbers[y * 3 + 1] = u;
        }
        let mut grid = Percolation::from_numbers(3, 5, numbers);
        grid.set_p(0.45);
        assert!(!grid.spans());
        assert_eq!(grid.clusters(), 2);
        grid.set_p(0.55);
        assert_eq!(grid.spans_at(), Some(0.5));
        assert_eq!((grid.clusters(), grid.largest_size()), (1, 5));
    }

    /// At p = 0 nothing is open; at p = 1 everything is, in one cluster that spans.
    #[test]
    fn p_zero_and_p_one() {
        let mut grid = Percolation::new(40, 30, 5);
        grid.set_p(0.0);
        assert_eq!(
            (grid.open_cells(), grid.clusters(), grid.largest_size()),
            (0, 0, 0)
        );
        assert!(!grid.spans());
        assert_eq!(grid.samples(), &[0.0]);
        grid.set_p(1.0);
        assert_eq!((grid.open_cells(), grid.clusters()), (1_200, 1));
        assert_eq!(grid.largest_size(), 1_200);
        assert!(grid.spans());
        assert_eq!(grid.samples().len(), TICKS + 1);
        assert_eq!(grid.samples()[TICKS], 1.0);
        assert_eq!(grid.cluster_size(39, 29), Some(1_200));
    }

    /// As p rises with the numbers fixed, the open cells only grow, clusters only merge
    /// (every cluster lies inside one cluster at any higher p), and the largest never
    /// shrinks. The number of clusters is not monotonic: a new isolated cell adds one.
    #[test]
    fn raising_p_only_opens_cells_and_merges_clusters() {
        let (w, h) = (50, 40);
        let mut grid = Percolation::new(w, h, 11);
        let mut before: Vec<Option<usize>> = vec![None; w * h];
        let mut largest = 0;
        for step in 0..=100 {
            grid.set_p(step as f64 / 100.0);
            let now: Vec<Option<usize>> = (0..w * h).map(|i| grid.root_of(i)).collect();
            let mut map = HashMap::new();
            for i in 0..w * h {
                if let Some(old) = before[i] {
                    let new = now[i].expect("an open cell stayed open");
                    assert_eq!(*map.entry(old).or_insert(new), new, "a cluster split");
                }
            }
            assert!(grid.largest_size() >= largest);
            largest = grid.largest_size();
            before = now;
        }
        let samples = grid.samples();
        assert!(samples.windows(2).all(|s| s[0] <= s[1]));
    }

    /// The chart's samples are what a fresh grid shows at each tick, however p got there.
    #[test]
    fn samples_are_the_largest_share_at_each_tick() {
        let mut grid = Percolation::new(30, 30, 8);
        grid.set_p(0.83);
        grid.set_p(0.6123);
        grid.set_p(0.75);
        assert_eq!(grid.samples().len(), 751);
        for k in [0, 300, 590, 600, 612, 613, 700, 750] {
            let mut fresh = Percolation::new(30, 30, 8);
            fresh.set_p(k as f64 / TICKS as f64);
            assert_eq!(
                grid.samples()[k],
                fresh.largest_size() as f64 / 900.0,
                "tick {k}"
            );
        }
    }

    /// Lowering p gives the grid that raising it to the same p gives.
    #[test]
    fn lowering_p_matches_raising_it() {
        let mut a = Percolation::new(60, 40, 2);
        a.set_p(0.8);
        a.set_p(0.58);
        let mut b = Percolation::new(60, 40, 2);
        b.set_p(0.58);
        assert_eq!(a.open_cells(), b.open_cells());
        assert_eq!(a.clusters(), b.clusters());
        assert_eq!(a.largest_size(), b.largest_size());
        assert_eq!(a.spans_at(), b.spans_at());
    }

    /// A_{n,m}(k): of the configurations of an n-wide, m-tall grid with k open cells, how
    /// many span, for each k, by trying every configuration.
    fn spanning_counts(n: usize, m: usize) -> Vec<u64> {
        let cells = n * m;
        let mut counts = vec![0; cells + 1];
        for mask in 0u32..1 << cells {
            let numbers = (0..cells)
                .map(|i| if mask >> i & 1 == 1 { 0.25 } else { 0.75 })
                .collect();
            let mut grid = Percolation::from_numbers(n, m, numbers);
            grid.set_p(0.5);
            if grid.spans() {
                counts[mask.count_ones() as usize] += 1;
            }
        }
        counts
    }

    fn binomial(n: u64, k: u64) -> u64 {
        (0..k).fold(1, |b, i| b * (n - i) / (i + 1))
    }

    /// Mertens (2022, arXiv:2109.12102), Section 2, gives the first and last spanning counts
    /// exactly: A(m) = n (3a); A(m + 1) = n(nm − m) + 2(n − 1)(m − 2) (3b); every
    /// configuration with fewer than n closed cells spans, A(nm − k) = C(nm, k) (4a); and
    /// the parity rule (6): the alternating sum of the A(k) is (−1)^s, s = ⌊m/2⌋n + ⌈m/2⌉.
    /// Checked on square and oblong grids, which also pins which way spanning runs: an
    /// n-wide grid has n straight columns of m cells.
    #[test]
    fn spanning_counts_match_mertens_formulas() {
        for (n, m) in [(2, 2), (3, 3), (4, 4), (3, 4), (4, 3), (2, 5)] {
            let a = spanning_counts(n, m);
            let (nn, mm) = (n as u64, m as u64);
            let cells = n * m;
            assert!(a[..m].iter().all(|&c| c == 0), "{n} x {m}: below m");
            assert_eq!(a[m], nn, "{n} x {m}: (3a)");
            assert_eq!(
                a[m + 1],
                nn * (nn * mm - mm) + 2 * (nn - 1) * (mm - 2),
                "{n} x {m}: (3b)"
            );
            for k in 0..n {
                assert_eq!(
                    a[cells - k],
                    binomial(cells as u64, k as u64),
                    "{n} x {m}: (4a)"
                );
            }
            let alternating: i64 = a
                .iter()
                .enumerate()
                .map(|(k, &c)| if k % 2 == 0 { c as i64 } else { -(c as i64) })
                .sum();
            let s = (m / 2) * n + m.div_ceil(2);
            assert_eq!(
                alternating,
                if s % 2 == 0 { 1 } else { -1 },
                "{n} x {m}: (6)"
            );
        }
    }

    /// The p at which an n x n grid spans with probability exactly 1/2, from Mertens's
    /// Table 4 (2022, arXiv:2109.12102), where "all decimals are exact". Here rounded to 15 places.
    const MERTENS_MEDIAN: [(usize, f64); 5] = [
        (2, 0.541_196_100_146_197),
        (3, 0.559_296_316_001_324),
        (4, 0.569_724_133_968_027),
        (16, 0.589_887_013_723_259),
        (24, 0.591_276_289_864_952),
    ];

    /// The spanning probability R_n(p) = Σ A(k) p^k (1 − p)^(N − k) of the exhaustive counts,
    /// solved for R = 1/2 by bisection, is Mertens's exact median for n = 2, 3 and 4.
    #[test]
    fn exact_spanning_probability_matches_mertens_for_small_grids() {
        for &(n, median) in &MERTENS_MEDIAN[..3] {
            let a = spanning_counts(n, n);
            let r = |p: f64| -> f64 {
                a.iter()
                    .enumerate()
                    .map(|(k, &c)| c as f64 * p.powi(k as i32) * (1.0 - p).powi((n * n - k) as i32))
                    .sum()
            };
            let (mut lo, mut hi) = (0.0, 1.0);
            for _ in 0..60 {
                let mid = (lo + hi) / 2.0;
                if r(mid) < 0.5 { lo = mid } else { hi = mid }
            }
            assert!(
                (lo - median).abs() < 1e-12,
                "{n} x {n}: {lo} against {median}"
            );
        }
    }

    /// The p of first spanning of each of `grids` random n x n grids: a grid spans at p
    /// when this is below p.
    fn spanning_ps(n: usize, grids: u64, seed: u64) -> Vec<f64> {
        (0..grids)
            .map(|g| {
                let mut grid = Percolation::new(n, n, seed + g);
                grid.set_p(1.0);
                grid.spans_at().unwrap()
            })
            .collect()
    }

    fn count_below(ps: &[f64], p: f64) -> u64 {
        ps.iter().filter(|&&s| s < p).count() as u64
    }

    /// The threshold estimated from many grids. At Mertens's exact median for 16 x 16 and
    /// 24 x 24 grids, each grid spans with probability exactly 1/2, so the count that do is
    /// binomial: it must be within 4 standard deviations of half. 100,000 grids pin the
    /// median to about ±0.0005 in p.
    #[test]
    #[ignore = "slow: 200,000 grids"]
    fn half_the_grids_span_at_mertens_median() {
        for &(n, median) in &MERTENS_MEDIAN[3..] {
            let grids = 100_000;
            let count = count_below(&spanning_ps(n, grids, 1_000 * n as u64), median);
            let sd = (grids as f64 * 0.25).sqrt();
            let off = (count as f64 - grids as f64 / 2.0).abs();
            assert!(off < 4.0 * sd, "{n} x {n}: {count} of {grids} spanned");
        }
    }

    /// On bigger grids the median moves on towards the threshold of the infinite lattice,
    /// p_c = 0.59274605079210(2) (Jacobsen, Journal of Physics A 48, 454003, 2015;
    /// arXiv:1507.03027). Mertens's exact medians rise towards it from below for every n up
    /// to 24, and he finds them converging as n^(−1 − 1/ν) = n^(−7/4) (his eq. 15, with
    /// ν = 4/3), which keeps them below it. So on 128 x 128 grids the median lies between
    /// his 24 x 24 median and p_c: fewer than half the grids span at the first, more than
    /// half at the second, each within 4 standard deviations.
    #[test]
    #[ignore = "slow: 4,000 grids of 128 x 128"]
    fn the_median_of_big_grids_lies_between_mertens_and_the_threshold() {
        let pc = 0.592_746_050_792_10;
        let grids = 4_000;
        let sd = (grids as f64 * 0.25).sqrt();
        let half = grids as f64 / 2.0;
        let ps = spanning_ps(128, grids, 7);
        let below = count_below(&ps, MERTENS_MEDIAN[4].1) as f64;
        let above = count_below(&ps, pc) as f64;
        assert!(below < half + 4.0 * sd, "{below} of {grids} at p = 0.59128");
        assert!(above > half - 4.0 * sd, "{above} of {grids} at p_c");
    }
}
