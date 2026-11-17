//! The Nagel–Schreckenberg traffic model: cars on a one-lane ring road of cells, each with a
//! whole-number speed from 0 to a maximum. Each step, every car, all at once:
//!
//! 1. accelerates: its speed goes up by one, unless it is at the maximum;
//! 2. slows down for the car ahead: its speed drops to the number of empty cells in front of
//!    it, if it is more than that;
//! 3. dawdles: if its speed is above 0, it drops by one with probability `p`;
//! 4. moves on by its speed.
//!
//! Introduced by K. Nagel and M. Schreckenberg, J. Phys. I France 2, 2221–2229 (1992). That
//! paper could not be read for this demo (the publisher and HAL refuse automated fetches),
//! so the rules, their order and the parallel update are taken from three restatements by
//! its authors and their colleagues, which agree:
//!
//! - M. Schreckenberg, A. Schadschneider, K. Nagel and N. Ito, *Discrete stochastic models
//!   for traffic flow*, Phys. Rev. E 51, 2939–2949 (1995), arXiv:cond-mat/9412045, Sec. II
//!   ("The order of the update rules given above is crucial").
//! - K. Nagel, *High-speed microsimulations of traffic flow*, PhD thesis, University of
//!   Cologne (1995), Sec. 3.2.
//! - D. Chowdhury, L. Santen and A. Schadschneider, *Statistical physics of vehicular traffic
//!   and some related systems*, Physics Reports 329, 199–329 (2000), arXiv:cond-mat/0007053,
//!   Sec. 8, rules U1–U4.
//!
//! Flow is measured as the review and the PRE paper measure it: the cars passing a point per
//! step, averaged over the road, which is the sum of the cars' speeds divided by the road's
//! length.

use ca_core::{Grid, Rng};

/// A history cell with no car in it. Any other value is the speed of the car there.
pub const EMPTY: u8 = u8::MAX;

/// One car: the cell it is in, and the speed it moved at in the last step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Car {
    pub x: usize,
    pub v: u8,
}

/// A car's speed for its next move, from rules 1 to 3: `v` is its speed in the last step,
/// `gap` the number of empty cells in front of it, and `dawdle` whether rule 3's coin came up.
pub fn next_speed(v: u8, gap: usize, v_max: u8, dawdle: bool) -> u8 {
    let v = v.saturating_add(1).min(v_max); // 1: accelerate
    let v = v.min(gap.min(usize::from(u8::MAX - 1)) as u8); // 2: slow down for the car ahead
    if dawdle && v > 0 { v - 1 } else { v } // 3: dawdle
}

/// A ring road of `length` cells and the cars on it.
pub struct Road {
    length: usize,
    /// In order along the road: each car's leader is the next one, and the last car's leader
    /// is the first. Cars never pass, so the order never changes; only where it starts, as
    /// cars wrap round from the end of the road to its start.
    cars: Vec<Car>,
    pub v_max: u8,
    /// Rule 3's probability of dawdling.
    pub p: f64,
    /// Steps taken.
    pub time: u64,
    /// The sum of the speeds in the last step: the cells moved by all the cars together.
    moved: usize,
    rng: Rng,
}

impl Road {
    /// `count` cars in distinct cells picked at random, all standing still.
    pub fn random(length: usize, count: usize, v_max: u8, p: f64, seed: u64) -> Self {
        assert!(count <= length, "{count} cars don't fit on {length} cells");
        let mut rng = Rng::new(seed);
        // The first `count` cells of a partial Fisher–Yates shuffle: every set of cells is
        // equally likely, and the count is exact.
        let mut cells: Vec<usize> = (0..length).collect();
        for i in 0..count {
            let j = i + (rng.next_u64() % (length - i) as u64) as usize;
            cells.swap(i, j);
        }
        let mut chosen = cells[..count].to_vec();
        chosen.sort_unstable();
        let cars = chosen.into_iter().map(|x| Car { x, v: 0 }).collect();
        Self::with_rng(length, cars, v_max, p, rng)
    }

    /// The cars given, in any order, which must be in distinct cells of the road.
    #[cfg(test)]
    pub fn from_cars(length: usize, mut cars: Vec<Car>, v_max: u8, p: f64, seed: u64) -> Self {
        cars.sort_unstable_by_key(|c| c.x);
        assert!(
            cars.windows(2).all(|w| w[0].x < w[1].x),
            "two cars in one cell"
        );
        assert!(cars.iter().all(|c| c.x < length), "a car is off the road");
        Self::with_rng(length, cars, v_max, p, Rng::new(seed))
    }

    fn with_rng(length: usize, cars: Vec<Car>, v_max: u8, p: f64, rng: Rng) -> Self {
        Self {
            length,
            cars,
            v_max,
            p,
            time: 0,
            moved: 0,
            rng,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn cars(&self) -> &[Car] {
        &self.cars
    }

    pub fn density(&self) -> f64 {
        self.cars.len() as f64 / self.length as f64
    }

    /// The sum of the cars' speeds in the last step.
    pub fn moved(&self) -> usize {
        self.moved
    }

    /// The empty cells in front of car `i`. A car alone on the road is its own leader, with
    /// every other cell in front of it.
    #[cfg(test)]
    pub fn gap(&self, i: usize) -> usize {
        let next = self.cars[(i + 1) % self.cars.len()].x;
        (next + self.length - self.cars[i].x - 1) % self.length
    }

    /// One step, with rule 3's coin tossed by the road's own generator.
    pub fn step(&mut self) {
        let (p, rng) = (self.p, &mut self.rng);
        self.moved = update(&mut self.cars, self.length, self.v_max, || rng.chance(p));
        self.time += 1;
    }

    /// One step, asking `dawdle` once for each car, in order along the road, whether its
    /// coin came up.
    #[cfg(test)]
    pub fn step_with(&mut self, dawdle: impl FnMut() -> bool) {
        self.moved = update(&mut self.cars, self.length, self.v_max, dawdle);
        self.time += 1;
    }

    /// The road as one row of cells: [`EMPTY`], or the speed of the car there.
    pub fn write_row(&self, row: &mut [u8]) {
        row.fill(EMPTY);
        for car in &self.cars {
            row[car.x] = car.v;
        }
    }
}

/// Apply the four rules to every car at once; returns the sum of their speeds. Every car's
/// speed is worked out from where the cars were before any of them moved.
fn update(cars: &mut [Car], length: usize, v_max: u8, mut dawdle: impl FnMut() -> bool) -> usize {
    let n = cars.len();
    if n == 0 {
        return 0;
    }
    // Cars are updated in order, so car i's leader, car i + 1, hasn't moved yet when car i is
    // worked out. The last car's leader is the first car, which has: use where it was.
    let first_x = cars[0].x;
    let mut moved = 0;
    for i in 0..n {
        let next_x = if i + 1 < n { cars[i + 1].x } else { first_x };
        let car = &mut cars[i];
        let gap = (next_x + length - car.x - 1) % length;
        car.v = next_speed(car.v, gap, v_max, dawdle()); // rules 1-3
        car.x = (car.x + usize::from(car.v)) % length; // rule 4
        moved += usize::from(car.v);
    }
    moved
}

/// The road's recent past as a space-time diagram, one row per step: row 0 at the top is
/// the oldest kept, the last filled row the current one. As in the elementary demo, once
/// the rows are full the picture scrolls up.
pub struct History {
    grid: Grid<u8>,
    filled: usize,
    /// The step drawn in the last filled row.
    time: u64,
}

impl History {
    pub fn new(road: &Road, rows: usize) -> Self {
        let mut grid = Grid::new(road.length(), rows.max(1), EMPTY);
        road.write_row(&mut grid.cells_mut()[..road.length()]);
        Self {
            grid,
            filled: 1,
            time: road.time,
        }
    }

    pub fn grid(&self) -> &Grid<u8> {
        &self.grid
    }

    /// Add the road as it is now as the newest row.
    pub fn push(&mut self, road: &Road) {
        let (width, height) = (self.grid.width(), self.grid.height());
        let cells = self.grid.cells_mut();
        if self.filled < height {
            self.filled += 1;
        } else {
            cells.copy_within(width.., 0); // scroll everything up one row
        }
        let last = (self.filled - 1) * width;
        road.write_row(&mut cells[last..last + width]);
        self.time = road.time;
    }

    /// The step drawn in row `y`, or `None` for a row not reached yet.
    pub fn time_of_row(&self, y: usize) -> Option<u64> {
        (y < self.filled).then(|| self.time + 1 + y as u64 - self.filled as u64)
    }
}

/// The flow at one density, measured on a road of its own: run `warmup` steps for the
/// random start to be forgotten, then average the flow over `steps` more.
pub struct Probe {
    road: Road,
    warmup: u64,
    steps: u64,
    total_moved: u64,
}

impl Probe {
    pub fn new(road: Road, warmup: u64, steps: u64) -> Self {
        Self {
            road,
            warmup,
            steps,
            total_moved: 0,
        }
    }

    pub fn done(&self) -> bool {
        self.road.time >= self.warmup + self.steps
    }

    /// Take steps until done or until about `budget` car moves have been made; returns the
    /// car moves made.
    pub fn work(&mut self, budget: usize) -> usize {
        let per_step = self.road.cars().len().max(1);
        let mut spent = 0;
        while !self.done() && spent < budget {
            self.road.step();
            if self.road.time > self.warmup {
                self.total_moved += self.road.moved() as u64;
            }
            spent += per_step;
        }
        spent
    }

    /// The mean flow over the measured steps so far.
    pub fn flow(&self) -> f64 {
        let measured = self.road.time.saturating_sub(self.warmup).max(1);
        self.total_moved as f64 / (measured as f64 * self.road.length() as f64)
    }
}

/// The flow on a random road of `length` cells holding `count` cars, averaged over `steps`
/// steps after `warmup`.
#[cfg(test)]
pub fn measure_flow(
    length: usize,
    count: usize,
    v_max: u8,
    p: f64,
    (warmup, steps): (u64, u64),
    seed: u64,
) -> f64 {
    let mut probe = Probe::new(Road::random(length, count, v_max, p, seed), warmup, steps);
    while !probe.done() {
        probe.work(usize::MAX);
    }
    probe.flow()
}

/// The fundamental diagram, flow against density, for one maximum speed and dawdling
/// probability, measured one density at a time on test roads of their own, a little each
/// frame.
pub struct Sweep {
    pub v_max: u8,
    pub p: f64,
    next: usize,
    probe: Option<Probe>,
    points: Vec<(f64, f64)>,
}

/// The test roads: 49 densities from 0.02 to 0.98. Shorter roads flow faster near the
/// peak: at maximum speed 5, dawdling 0.5 and density 0.08, a 500-cell road averaged 0.347
/// even over 100,000 steps, where 2,000 cells give 0.320 over these 1,500, as do 50,000
/// (0.318; the thesis, Sec. 3.5, gives 0.32).
const SWEEP_LENGTH: usize = 2_000;
const SWEEP_DENSITIES: usize = 49;
const SWEEP_STEPS: (u64, u64) = (500, 1_000);

impl Sweep {
    pub fn new(v_max: u8, p: f64) -> Self {
        Self {
            v_max,
            p,
            next: 0,
            probe: None,
            points: Vec::with_capacity(SWEEP_DENSITIES),
        }
    }

    pub fn done(&self) -> bool {
        self.next >= SWEEP_DENSITIES
    }

    /// The densities measured so far, with their flows.
    pub fn points(&self) -> &[(f64, f64)] {
        &self.points
    }

    /// Measure for up to about `budget` car moves.
    pub fn work(&mut self, mut budget: usize) {
        while !self.done() && budget > 0 {
            let k = self.next;
            let (v_max, p) = (self.v_max, self.p);
            let probe = self.probe.get_or_insert_with(|| {
                let count = (k + 1) * SWEEP_LENGTH / (SWEEP_DENSITIES + 1);
                let road = Road::random(SWEEP_LENGTH, count, v_max, p, k as u64 + 1);
                Probe::new(road, SWEEP_STEPS.0, SWEEP_STEPS.1)
            });
            budget = budget.saturating_sub(probe.work(budget));
            if probe.done() {
                let density = probe.road.density();
                self.points.push((density, probe.flow()));
                self.probe = None;
                self.next += 1;
            }
        }
    }
}

/// The flow with no dawdling, once the road has settled: every car at full speed while
/// there is room for it, otherwise every empty cell moved into each step. (Review Eq. 63,
/// thesis Sec. 4.1.)
pub fn flow_without_dawdling(density: f64, v_max: u8) -> f64 {
    (density * f64::from(v_max)).min(1.0 - density)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cars_at(cells: &[(usize, u8)]) -> Vec<Car> {
        cells.iter().map(|&(x, v)| Car { x, v }).collect()
    }

    fn sorted(road: &Road) -> Vec<(usize, u8)> {
        let mut c: Vec<_> = road.cars().iter().map(|c| (c.x, c.v)).collect();
        c.sort_unstable();
        c
    }

    fn occupancy(road: &Road) -> Vec<bool> {
        let mut row = vec![EMPTY; road.length()];
        road.write_row(&mut row);
        row.iter().map(|&c| c != EMPTY).collect()
    }

    /// Rule 1: a car speeds up by one, but never past the maximum.
    #[test]
    fn rule_1_accelerates_to_the_maximum() {
        assert_eq!(next_speed(0, 100, 5, false), 1);
        assert_eq!(next_speed(4, 100, 5, false), 5);
        assert_eq!(next_speed(5, 100, 5, false), 5);
        // A lone car on a long road: speeds 1, 2, 3, 3, ..., so it is at 1, 3, 6, 9.
        let mut road = Road::from_cars(100, cars_at(&[(0, 0)]), 3, 0.0, 1);
        let mut trace = vec![];
        for _ in 0..4 {
            road.step();
            trace.push((road.cars()[0].x, road.cars()[0].v));
        }
        assert_eq!(trace, [(1, 1), (3, 2), (6, 3), (9, 3)]);
    }

    /// Rule 2: a car slows to the number of empty cells in front of it, and no further.
    #[test]
    fn rule_2_slows_to_the_gap() {
        assert_eq!(next_speed(3, 1, 5, false), 1);
        assert_eq!(next_speed(5, 0, 5, false), 0);
        assert_eq!(next_speed(2, 3, 5, false), 3, "a gap of 3 allows 2 + 1");
        assert_eq!(
            next_speed(2, 2, 5, false),
            2,
            "a gap of 2 stops it speeding up"
        );
    }

    /// Rule 3: after rules 1 and 2, a dawdling car loses one unit of speed, but never goes
    /// below 0. The order matters: a car braking for the car ahead can dawdle as well and
    /// overreact, slowing to less than its gap, which is what starts jams (review, Sec. 8:
    /// with rules 2 and 3 swapped "there will be no overreactions at braking"). Swapped,
    /// the last case below would give 1.
    #[test]
    fn rule_3_dawdles_after_braking() {
        assert_eq!(
            next_speed(2, 100, 5, true),
            2,
            "speeds up to 3, dawdles to 2"
        );
        assert_eq!(next_speed(0, 0, 5, true), 0, "a stopped car can't dawdle");
        assert_eq!(
            next_speed(0, 5, 5, true),
            0,
            "a car pulling away may not get going"
        );
        assert_eq!(
            next_speed(2, 1, 2, true),
            0,
            "brakes to 1, then overreacts to 0"
        );
    }

    /// Rule 4, round the ring: a car moves on by its speed, from the road's end to its start.
    /// The car in the last cell has the car in cell 0 as its leader, and must see where that
    /// car was before the step, not where it has just moved: gap 0, so it stays put.
    #[test]
    fn rule_4_moves_round_the_ring() {
        let mut road = Road::from_cars(100, cars_at(&[(98, 2)]), 3, 0.0, 1);
        road.step();
        assert_eq!(road.cars()[0], Car { x: 1, v: 3 });

        let mut road = Road::from_cars(10, cars_at(&[(0, 0), (9, 0)]), 5, 0.0, 1);
        road.step();
        assert_eq!(sorted(&road), [(1, 1), (9, 0)]);
        assert_eq!(road.moved(), 1);
    }

    /// The edges: a car alone has every other cell in front of it, so on a ring of 5 its top
    /// speed is 4; a full road never moves; a single hole moves back one cell a step.
    #[test]
    fn a_lone_car_a_full_road_and_a_single_hole() {
        let mut road = Road::from_cars(5, cars_at(&[(0, 0)]), 9, 0.0, 1);
        assert_eq!(road.gap(0), 4);
        let speeds: Vec<u8> = (0..6)
            .map(|_| {
                road.step();
                road.cars()[0].v
            })
            .collect();
        assert_eq!(speeds, [1, 2, 3, 4, 4, 4]);

        let mut full = Road::random(20, 20, 5, 0.5, 3);
        for _ in 0..10 {
            full.step();
            assert_eq!(full.moved(), 0);
        }

        let mut holed = Road::random(20, 19, 5, 0.0, 3);
        let hole = |r: &Road| occupancy(r).iter().position(|&o| !o).unwrap();
        let mut at = hole(&holed);
        for _ in 0..40 {
            holed.step();
            assert_eq!(hole(&holed), (at + 19) % 20, "the hole moves back one cell");
            assert_eq!(holed.moved(), 1);
            at = hole(&holed);
        }
    }

    /// How a jam starts: one car dawdles, the car behind has to brake for it, and if that
    /// car dawdles too it overreacts and stops dead. The coins are chosen here: two cars at
    /// full speed 2, two cells apart.
    #[test]
    fn one_dawdle_can_stop_the_car_behind() {
        let mut road = Road::from_cars(100, cars_at(&[(0, 2), (3, 2)]), 2, 0.0, 1);
        let mut coins = [false, true].into_iter(); // the front car dawdles
        road.step_with(|| coins.next().unwrap());
        assert_eq!(sorted(&road), [(2, 2), (4, 1)]);
        let mut calm = Road::from_cars(100, cars_at(&[(2, 2), (4, 1)]), 2, 0.0, 1);
        calm.step_with(|| false);
        assert_eq!(
            sorted(&calm),
            [(3, 1), (6, 2)],
            "the car behind brakes to 1"
        );
        let mut coins = [true, false].into_iter(); // now the car behind dawdles as well
        road.step_with(|| coins.next().unwrap());
        assert_eq!(sorted(&road), [(2, 0), (6, 2)], "and stops");
    }

    /// Elementary Rule 184, in Wolfram's numbering, as the elementary demo computes it: a
    /// cell's next value is bit (left, centre, right) of 184, on a ring.
    fn rule_184(row: &[bool]) -> Vec<bool> {
        let n = row.len();
        (0..n)
            .map(|i| {
                let (l, c, r) = (row[(i + n - 1) % n], row[i], row[(i + 1) % n]);
                let pattern = (l as u8) << 2 | (c as u8) << 1 | r as u8;
                184u8 >> pattern & 1 == 1
            })
            .collect()
    }

    /// With maximum speed 1 and no dawdling, the model is Rule 184 (review, Sec. 8: "In the
    /// special case vmax = 1 the deterministic limit of the NaSch model is equivalent to the
    /// CA rule 184 in Wolfram's notation"): a car moves one cell right exactly when the cell
    /// ahead is empty. Checked cell for cell from random roads dense enough to have queues
    /// like 111, where only the front car may move.
    #[test]
    fn max_speed_1_without_dawdling_is_rule_184() {
        for (count, seed) in [(10, 1), (50, 2), (101, 3), (150, 4), (190, 5)] {
            let mut road = Road::random(201, count, 1, 0.0, seed);
            let mut row = occupancy(&road);
            for t in 0..300 {
                road.step();
                row = rule_184(&row);
                assert_eq!(occupancy(&road), row, "{count} cars, step {t}");
            }
        }
    }

    /// Whatever the settings, cars are neither made nor lost, never share a cell, never move
    /// further than the empty cells in front of them (so never pass the car ahead), and keep
    /// to speeds from 0 to the maximum; each moves exactly its speed.
    #[test]
    fn cars_are_conserved_and_never_overlap_or_pass() {
        for v_max in 1..=8 {
            for (i, p) in [0.0, 0.1, 0.5, 0.9, 1.0].into_iter().enumerate() {
                for count in [1, 7, 60, 150, 199, 200] {
                    let seed = u64::from(v_max) * 100 + i as u64 * 10 + count as u64;
                    let mut road = Road::random(200, count, v_max, p, seed);
                    for _ in 0..100 {
                        let before = road.cars().to_vec();
                        let gaps: Vec<usize> = (0..count).map(|i| road.gap(i)).collect();
                        road.step();
                        assert_eq!(road.cars().len(), count);
                        assert_eq!(occupancy(&road).iter().filter(|&&o| o).count(), count);
                        for ((b, a), gap) in before.iter().zip(road.cars()).zip(gaps) {
                            assert!(a.v <= v_max);
                            assert!(usize::from(a.v) <= gap, "moved past the car ahead");
                            assert_eq!(a.x, (b.x + usize::from(a.v)) % 200);
                        }
                    }
                }
            }
        }
    }

    /// The model with no dawdling settles into one of two steady states (thesis, Sec. 4.1;
    /// review, Sec. 8.2.1 and Eq. 63, J = min(c vmax, 1 - c)). Below density 1/(vmax + 1)
    /// every car drives at full speed, and the whole pattern moves vmax cells forward each
    /// step. Above it, cars move into every empty cell, the flow is 1 - c, and the pattern
    /// (stop-and-go waves) moves one cell backwards each step. Checked exactly, as integers.
    #[test]
    fn without_dawdling_the_road_settles_to_the_known_flows() {
        let length = 1_000;
        let shifted = |r: &Road, by: isize| {
            let mut c: Vec<_> = r
                .cars()
                .iter()
                .map(|c| {
                    (
                        (c.x as isize + by).rem_euclid(length as isize) as usize,
                        c.v,
                    )
                })
                .collect();
            c.sort_unstable();
            c
        };
        for density in [0.05, 0.1, 0.3, 0.5, 0.8] {
            let count = (density * length as f64) as usize;
            let mut road = Road::random(length, count, 5, 0.0, 9);
            for _ in 0..2_000 {
                road.step();
            }
            let free = density < 1.0 / 6.0;
            let shift = if free { 5 } else { -1 };
            for _ in 0..50 {
                let expected = shifted(&road, shift);
                road.step();
                assert_eq!(sorted(&road), expected, "density {density}");
                let moved = if free { 5 * count } else { length - count };
                assert_eq!(road.moved(), moved, "density {density}");
            }
            let flow = road.moved() as f64 / length as f64;
            assert!((flow - flow_without_dawdling(density, 5)).abs() < 1e-12);
        }
    }

    /// Rows of Figs. 4.2 and 4.3 of Nagel's thesis (Sec. 4.1): the model with no dawdling,
    /// maximum speed 5, on a ring of 70 cells. Each row is the road "after one further
    /// complete velocity-update and just before the car motion" (caption of Fig. 3.2): a
    /// digit is a car and the speed it is about to move at. Rows in one slice are consecutive
    /// steps; the slices are separated in the figures by "now steady" or a disturbance.
    const THESIS_FIG_4_2: [&[&str]; 2] = [
        &[
            "...........5.....1.1.1.............4....5............4.....3..........",
            "................1.1.1.2................5.....5...........4....4.......",
            ".................1.1.2..3...................5.....5..........4....5...",
            ".5................1.2..3...4.....................5.....5.........5....",
            "5.....5............2..3...4....5......................5.....5.........",
            ".....5.....5.........3...4....5.....5......................5.....5....",
            "5.........5.....5.......4....5.....5.....5......................5.....",
        ],
        &[
            ".....5.........5.....5......5.....5.....5.....5......................5",
            "....5.....5.........5.....5......5.....5.....5.....5..................",
            ".........5.....5.........5.....5......5.....5.....5.....5.............",
            "..............5.....5.........5.....5......5.....5.....5.....5........",
            "...................5.....5.........5.....5......5.....5.....5.....5...",
            ".5......................5.....5.........5.....5......5.....5.....5....",
            "5.....5......................5.....5.........5.....5......5.....5.....",
            ".....5.....5......................5.....5.........5.....5......5.....5",
            "....5.....5.....5......................5.....5.........5.....5......5.",
            "...5.....5.....5.....5......................5.....5.........5.....5...",
        ],
    ];
    const THESIS_FIG_4_3: [&[&str]; 4] = [
        &[
            "..........1.1.4....03...2..4....04.......4....1.1.4....00..5.....01.1.",
            "...........1.2....01...2..3....01....5.......1.1.2....001.......01.1.2",
            ".3..........2..2..1.2....3...1.1.2........3...1.2..2..01.2......1.1.2.",
            "3...4.........2..1.2..3.....1.1.2..3.........1.2..2..01.2..3.....1.2..",
            "...4....5.......1.2..3...3...1.2..3...4.......2..2..01.2..3...3...2..3",
            "..4....5.....3...2..3...3...1.2..3...4....5.....2..01.2..3...3...2..3.",
        ],
        &[
            ".4....5.....3...2..3...3...1.2..3...4....5.....2..01.2..3...3...2..3..",
            "4....5.....3...2..3...3...1.2..3...4....5.....2..01.2..3...3...2..3...",
            "....5.....3...2..3...3...1.2..3...4....5.....2..01.2..3...3...2..3...4",
            "...5.....3...2..3...3...1.2..3...4....5.....2..01.2..3...3...2..3...4.",
            "..5.....3...2..3...3...1.2..3...4....5.....2..01.2..3...3...2..3...4..",
        ],
        &[
            ".5.....3...2..3...3...1.1..3...4....5.....2..01.2..3...3...2..3...4...",
            "5.....3...2..3...3...1.1.2....4....5.....2..01.2..3...3...2..3...4....",
            ".....3...2..3...3...1.1.2..3......5.....2..01.2..3...3...2..3...4....5",
            "....3...2..3...3...1.1.2..3...4........2..01.2..3...3...2..3...4....5.",
            "...3...2..3...3...1.1.2..3...4....5......01.2..3...3...2..3...4....5..",
            "..3...2..3...3...1.1.2..3...4....5.....1.1.2..3...3...2..3...4....5...",
        ],
        &[
            ".3...2..3...3...1.1.2..3...4....5.....1.1.2..3...3...2..3...4....5....",
            "3...2..3...3...1.1.2..3...4....5.....1.1.2..3...3...2..3...4....5.....",
        ],
    ];

    /// A thesis row's cars: (cell, speed about to be moved at).
    fn row_cars(row: &str) -> Vec<(usize, u8)> {
        row.bytes()
            .enumerate()
            .filter(|&(_, b)| b != b'.')
            .map(|(x, b)| (x, b - b'0'))
            .collect()
    }

    /// The thesis's rows, step for step: a row shows each car before it moves, so the road
    /// in this model's terms (each car where it has just moved to, with the speed it moved
    /// at) is every car moved on by its digit.
    #[test]
    fn matches_the_thesis_figures_without_dawdling() {
        let moved = |row: &str| -> Vec<(usize, u8)> {
            let mut c: Vec<_> = row_cars(row)
                .into_iter()
                .map(|(x, v)| ((x + usize::from(v)) % 70, v))
                .collect();
            c.sort_unstable();
            c
        };
        let mut checked = 0;
        for slice in THESIS_FIG_4_2.iter().chain(&THESIS_FIG_4_3) {
            for pair in slice.windows(2) {
                assert_eq!(pair[0].len(), 70);
                let cars = moved(pair[0]).into_iter().map(|(x, v)| Car { x, v });
                let mut road = Road::from_cars(70, cars.collect(), 5, 0.0, 1);
                road.step();
                assert_eq!(sorted(&road), moved(pair[1]), "after {}", pair[0]);
                checked += 1;
            }
        }
        assert_eq!(checked, 30);
    }

    /// Fig. 3.3 of Nagel's thesis (Sec. 3.3.1): the model with dawdling, maximum speed 5, at
    /// density 0.1, 71 cells wide. Its rows don't wrap round as a ring of 71 would, so it
    /// is taken to be a stretch of a longer road, and the front car, whose leader is off the
    /// picture, is skipped. Every speed it shows must be
    /// what rules 1 to 3 give, with or without the car dawdling. It includes overreactions,
    /// where a braking car also dawdles and ends below its gap, which the rules allow only in
    /// this order.
    const THESIS_FIG_3_3: &[&str] = &[
        "........4................5......5......5........00.1.......4..........5",
        "..4.........5.................4......5......2...00..2..........5.......",
        "......4..........5................4.......3...0.01....2.............5..",
        "4.........4...........4...............5......01.1.2.....3..............",
        "....5.........5...........5................0.1.0.2..3......4...........",
        ".........5.........4...........4...........1..00...3...3.......5.......",
        ".4............4........4...........5........1.00......3...3.........5..",
        ".....4............5........4............3....000.........2...3.........",
        ".........4.............5.......4...........0.001...........3....3......",
        ".............4..............5......4.......0.00.2.............4....4...",
        "4................4...............5.....2...1.01...2...............3....",
        "....4................4................2..1..01.2....3................4.",
        "........5................5..............0.1.0.2..2.....3...............",
        ".............5................5.........0..00...1..2......3............",
        "..4...............4................3....1..00....2...3.......3.........",
        "......5...............4...............2..0.00......2....3.......3......",
        "..5........4..............4.............01.00........3.....4.......4...",
        ".5.....4.......5..............5.........0.000...........3......4.......",
        "......4....5........5..............4....1.001..............3.......4...",
        "4.........5.....5........4.............1.000.1................4........",
        "....5..........4.....5.......5..........0000..1...................5....",
        ".........4.........5......4.......4.....0000...2.......................",
        ".............5..........4.....5.......0.0001.....2.....................",
        "..................4.........4......1..0.000.2......2...................",
        ".5....................4.........2...0.1.000...3......3.................",
        "......5...................5.......1.0..0001......4......4..............",
        "..5........5...................2...01..001.2.........4......5..........",
        ".......5........4................1.0.0.01.1..3...........5.......4.....",
        "....5.......5.......5.............00.0.0.1.1....3.............4......5.",
        ".........4.......5.......5........01.1.0..0.1......3..............4....",
        ".............4........5.......2...1.0.00..1..2........4...............5",
        ".................5.........4....2..01.01...2...3..........5............",
        "5.....................4........2..01.00.2....3....3............4.......",
        ".....4....................5......00.000...3.....3....4.............5...",
    ];

    #[test]
    fn matches_the_thesis_figure_with_dawdling() {
        let (mut checked, mut overreactions) = (0, 0);
        for pair in THESIS_FIG_3_3.windows(2) {
            // Where each car stands when its next speed is worked out.
            let at: Vec<(usize, u8)> = row_cars(pair[0])
                .into_iter()
                .map(|(x, v)| (x + usize::from(v), v))
                .collect();
            let next: std::collections::HashMap<usize, u8> =
                row_cars(pair[1]).into_iter().collect();
            for w in at.windows(2) {
                let ((x, v), (leader, _)) = (w[0], w[1]);
                let gap = leader - x - 1;
                let shown = next[&x];
                let (keep, dawdle) = (next_speed(v, gap, 5, false), next_speed(v, gap, 5, true));
                assert!(
                    shown == keep || shown == dawdle,
                    "car at {x} in {}",
                    pair[0]
                );
                if shown + 1 == keep && keep == gap as u8 && v.saturating_add(1).min(5) > keep {
                    overreactions += 1;
                }
                checked += 1;
            }
        }
        assert_eq!(checked, 284);
        assert!(overreactions >= 20, "{overreactions} overreactions");
    }

    /// A car alone at full speed is at the maximum with probability 1 - p and one below it
    /// otherwise, so its mean speed is vmax - p (thesis, Sec. 3.2; review, Sec. 8.3: at low
    /// density J = c vF with vF = vmax - p).
    #[test]
    fn a_lone_car_averages_the_maximum_less_p() {
        let mut road = Road::from_cars(100_000, cars_at(&[(0, 5)]), 5, 0.3, 11);
        let steps = 100_000;
        let total: usize = (0..steps)
            .map(|_| {
                road.step();
                road.moved()
            })
            .sum();
        let mean = total as f64 / steps as f64;
        assert!((mean - 4.7).abs() < 0.01, "mean speed {mean}");
    }

    /// A single empty cell on a full road moves back one cell whenever the car behind it
    /// pulls away, which happens with probability 1 - p: near density 1 the flow is
    /// (1 - p)(1 - c), "as flow is determined by holes moving backwards at a speed 1 - p"
    /// (review, Sec. 8.3).
    #[test]
    fn a_single_hole_moves_back_at_1_minus_p() {
        let mut road = Road::random(1_000, 999, 5, 0.25, 12);
        let steps = 100_000;
        let total: usize = (0..steps)
            .map(|_| {
                road.step();
                road.moved()
            })
            .sum();
        let speed = total as f64 / steps as f64;
        assert!((speed - 0.75).abs() < 0.01, "hole speed {speed}");
    }

    /// The history keeps rows in order and numbers them by step, filling then scrolling.
    #[test]
    fn the_history_scrolls_and_numbers_its_rows() {
        let mut road = Road::random(30, 10, 5, 0.5, 4);
        let mut history = History::new(&road, 4);
        assert_eq!(history.time_of_row(0), Some(0));
        assert_eq!(history.time_of_row(1), None);
        let mut rows = vec![];
        for _ in 0..6 {
            road.step();
            history.push(&road);
            let mut row = vec![0; 30];
            road.write_row(&mut row);
            rows.push(row);
        }
        assert_eq!(history.time_of_row(0), Some(3));
        assert_eq!(history.time_of_row(3), Some(6));
        assert_eq!(&history.grid().cells()[3 * 30..], rows[5].as_slice());
        assert_eq!(&history.grid().cells()[..30], rows[2].as_slice());
    }

    /// The sweep measures every density once, in order, whatever the budget per call, and
    /// the same as measuring each density whole.
    #[test]
    fn the_sweep_measures_each_density_once() {
        let mut sweep = Sweep::new(5, 0.5);
        while !sweep.done() {
            sweep.work(77_777);
        }
        let points = sweep.points();
        assert_eq!(points.len(), SWEEP_DENSITIES);
        assert!((points[0].0 - 0.02).abs() < 1e-12 && (points[48].0 - 0.98).abs() < 1e-12);
        let count = 5 * SWEEP_LENGTH / (SWEEP_DENSITIES + 1);
        let alone = measure_flow(SWEEP_LENGTH, count, 5, 0.5, SWEEP_STEPS, 5);
        assert_eq!(points[4].1, alone);
    }

    /// With maximum speed 1 the flow is known exactly for any p, in the limit of a long
    /// road: J = (1 - sqrt(1 - 4 q c (1 - c))) / 2 with q = 1 - p (Schreckenberg,
    /// Schadschneider, Nagel and Ito 1995, Eq. 5.11, proved exact in their Appendix A;
    /// review Eq. 78).
    #[test]
    #[ignore]
    fn max_speed_1_has_the_exact_flow() {
        let length = 10_000;
        for p in [0.25, 0.5, 0.75] {
            for c in [0.1, 0.3, 0.5, 0.7, 0.9] {
                let count = (c * length as f64) as usize;
                let measured = measure_flow(length, count, 1, p, (2_000, 20_000), 21);
                let q = 1.0 - p;
                let exact = (1.0 - (1.0 - 4.0 * q * c * (1.0 - c)).sqrt()) / 2.0;
                assert!(
                    (measured - exact).abs() < 0.002,
                    "p {p}, density {c}: measured {measured}, exact {exact}"
                );
            }
        }
    }

    /// With maximum speed 5 and p = 0.5 the flow is greatest at density 0.085 ± 0.005
    /// (review, Sec. 8.4 and Fig. 28). The curve is flat near its top, so this only checks
    /// that the highest of a coarse scan is within a step of the scan of there. The
    /// greatest flow is 0.32 cars a step (thesis, Sec. 3.5: "our maximum of the flow is only
    /// 0.32 vehicles per time step"; the thesis's simulations are mainly at these settings,
    /// Sec. 3.2, though that sentence doesn't restate them).
    #[test]
    #[ignore]
    fn max_speed_5_flows_most_near_density_0_085() {
        let length = 10_000;
        let scan: Vec<(f64, f64)> = (5..=13)
            .map(|k| {
                let c = k as f64 / 100.0;
                let count = k * length / 100;
                (c, measure_flow(length, count, 5, 0.5, (5_000, 20_000), 31))
            })
            .collect();
        let best = scan
            .iter()
            .copied()
            .fold((0.0, 0.0), |a, b| if b.1 > a.1 { b } else { a });
        assert!(
            (best.0 - 0.085).abs() <= 0.016,
            "highest flow at density {}: {scan:?}",
            best.0
        );
        assert!((best.1 - 0.32).abs() < 0.005, "greatest flow {}", best.1);
    }
}
