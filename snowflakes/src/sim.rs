//! Clifford Reiter's local cellular model of snow crystal growth.
//!
//! Source: C. A. Reiter, "A local cellular model for snow crystal growth", Chaos, Solitons &
//! Fractals 23(4) (2005) 1111-1119, doi:10.1016/S0960-0779(04)00374-1. The rule, the figures
//! and the parameter values quoted below are from the author's preprint,
//! <https://webbox.lafayette.edu/~reiterc/mvp/sfn/sfn_pp.pdf>, and his J script on the
//! paper's auxiliary page, <https://webbox.lafayette.edu/~reiterc/mvp/sfn/sfn.html>.
//!
//! Each cell of a hexagonal grid holds a real amount of water. A cell holding 1 or more is
//! **ice**. Each step, all cells update at once:
//!
//! 1. A cell is **receptive** if it is ice or has an ice neighbour.
//! 2. Receptive cells keep their water (it is frozen in place) and gain a constant `gamma`,
//!    water arriving from outside the plane of growth.
//! 3. The water of the other cells diffuses. The diffusing field is the water with every
//!    receptive cell set to zero; each cell's new value is a weighted average of that field
//!    over the cell and its six neighbours: weight `1 - alpha/2` for itself and `alpha/12`
//!    for each neighbour (Reiter's standard choice is `alpha = 1`: 1/2 and 1/12).
//!
//! A cell's new water is the sum of its share of the diffusion and, if it is receptive, its
//! frozen water plus `gamma`. So water that diffuses onto a receptive cell stays there: the
//! crystal is a sink, and grows where most vapour reaches it, at its tips.
//!
//! It starts from one ice cell (water 1) in a field of constant `beta`, the background
//! vapour. Cells outside a circle of radius [`Crystal::radius`] around the seed are held at
//! `beta`, the far field; growth stops when the ice comes within [`MARGIN`] of it.
//!
//! Radii are measured as Reiter's script measures them, in hex **rows**: a radius of `N`
//! is `N * sqrt(3)/2` cell widths. His function `cir` scales each row's offset by sin 60°,
//! and puts the fixed boundary at the distance of the top row, `N` rows from the seed, and
//! the circle where growth stops at the third row, 2 rows in. So his "radius 200" is a circle
//! of 173.2 cell widths, and the step counts in his tables come out the same here.

use ca_core::Grid;

/// The six neighbours of a hex cell in axial coordinates (see `ca_core::Layout::Hex`): the
/// two beside it in its row, and two each in the rows above and below. The same constant
/// as `sandpile::sim::HEX`, copied so this demo does not depend on the sandpile.
pub const HEX: [(isize, isize); 6] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, -1), (-1, 1)];

/// Growth stops when an ice cell comes this many rows from the boundary circle, as in
/// Reiter's script (the circle through its third row).
pub const MARGIN: usize = 2;

/// The model's parameters, in Reiter's notation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    /// Diffusion: neighbours weigh `alpha / 12` each, the cell itself `1 - alpha / 2`.
    pub alpha: f64,
    /// The background vapour level: every cell's water at the start, and the far field.
    /// Below 1, or the background would be ice. Takes effect only in [`Crystal::new`].
    pub beta: f64,
    /// Water added to every receptive cell each step.
    pub gamma: f64,
}

/// A named example from Reiter's paper.
pub struct Preset {
    pub name: &'static str,
    /// What the paper says about it, and where.
    pub note: &'static str,
    pub params: Params,
}

const fn params(alpha: f64, beta: f64, gamma: f64) -> Params {
    Params { alpha, beta, gamma }
}

/// Reiter's named examples: Figure 4 ("Selected dendrite, stellar and sector forms"), the
/// first four, described in his text; a plate, from the text ("When β = 0 and γ > 0, plates
/// form") with γ = 1 as in an animation on his auxiliary page; and Figure 5's α = 2.003.
pub const PRESETS: &[Preset] = &[
    Preset {
        name: "Thin dendrite",
        note: "Figure 4: secondary and even tertiary branches.",
        params: params(1.0, 0.3, 0.0001),
    },
    Preset {
        name: "Thick dendrite",
        note: "Figure 4: a fairly regular pattern of side branches.",
        params: params(1.0, 0.35, 0.001),
    },
    Preset {
        name: "Stellar",
        note: "Figure 4: dendrites that have nearly collapsed into a stellar form.",
        params: params(1.0, 0.6, 0.01),
    },
    Preset {
        name: "Sector",
        note: "Figure 4: a somewhat abstract sector form, with small gaps.",
        params: params(1.0, 0.9, 0.05),
    },
    Preset {
        name: "Plate",
        note: "With no background vapour (β = 0), plates form.",
        params: params(1.0, 0.0, 1.0),
    },
    Preset {
        name: "Many levels",
        note: "Figure 5: α = 2.003 gives many levels of dendrites.",
        params: params(2.003, 0.4, 0.0001),
    },
];

pub struct Crystal {
    /// Water per cell, on a square grid of axial coordinates with the seed at the centre.
    water: Grid<f64>,
    next: Grid<f64>,
    /// The field that diffuses: water, with receptive cells zeroed.
    field: Vec<f64>,
    frozen: Vec<bool>,
    receptive: Vec<bool>,
    /// For each row, the cells inside the boundary circle: the only cells that update.
    rows: Vec<std::ops::Range<usize>>,
    radius: usize,
    pub params: Params,
    /// The background the crystal was started in (`params.beta` may since have changed).
    beta: f64,
    pub generation: u64,
    ice: usize,
    /// The largest squared distance from the centre of any ice cell.
    reach2: i64,
    /// The most water in any cell, for shading.
    most: f64,
}

impl Crystal {
    /// A single ice seed in a field of `params.beta`, with room to grow to `radius` hex rows.
    pub fn new(radius: usize, params: Params) -> Self {
        assert!(radius > MARGIN + 1, "too small to grow");
        assert!(
            (0.0..1.0).contains(&params.beta),
            "the background must not be ice"
        );
        // No inside cell is `radius` or more steps along an axial coordinate from the
        // centre, so this leaves every inside cell's neighbours on the grid (held at beta).
        let c = radius + 2;
        let side = 2 * c + 1;
        let rows = (0..side)
            .map(|y| {
                let inside = |x: usize| within(distance2(x, y, c), radius);
                // The disc is convex, so the inside cells of a row are contiguous.
                match (0..side).find(|&x| inside(x)) {
                    Some(x0) => x0..(x0..side).find(|&x| !inside(x)).unwrap_or(side),
                    None => 0..0,
                }
            })
            .collect();
        let mut crystal = Self {
            water: Grid::new(side, side, params.beta),
            next: Grid::new(side, side, params.beta),
            field: vec![params.beta; side * side],
            frozen: vec![false; side * side],
            receptive: vec![false; side * side],
            rows,
            radius,
            params,
            beta: params.beta,
            generation: 0,
            ice: 0,
            reach2: 0,
            most: params.beta,
        };
        crystal.freeze(c, c);
        crystal
    }

    pub fn water(&self) -> &Grid<f64> {
        &self.water
    }

    /// The seed's cell.
    #[cfg(test)]
    pub fn center(&self) -> (usize, usize) {
        let c = self.water.width() / 2;
        (c, c)
    }

    /// Radius of the boundary circle, in hex rows.
    pub fn radius(&self) -> usize {
        self.radius
    }

    /// The background vapour level the crystal was started in.
    pub fn beta(&self) -> f64 {
        self.beta
    }

    /// Cells updated each step (those inside the boundary circle).
    pub fn active_cells(&self) -> usize {
        self.rows.iter().map(|r| r.len()).sum()
    }

    pub fn is_inside(&self, x: usize, y: usize) -> bool {
        self.rows.get(y).is_some_and(|r| r.contains(&x))
    }

    pub fn ice_cells(&self) -> usize {
        self.ice
    }

    /// The distance from the centre to the farthest ice, in hex rows (as the radius).
    pub fn reach(&self) -> f64 {
        (self.reach2 as f64 * 4.0 / 3.0).sqrt()
    }

    /// The most water in any cell (ice keeps gaining `gamma`, most at the oldest ice).
    pub fn most_water(&self) -> f64 {
        self.most
    }

    /// Whether the ice has come within [`MARGIN`] rows of the boundary, so growth has
    /// stopped.
    pub fn finished(&self) -> bool {
        !within(self.reach2, self.radius - MARGIN)
    }

    /// Whether cell `(x, y)` was receptive in the last step (false before the first).
    #[cfg(test)]
    pub fn was_receptive(&self, x: usize, y: usize) -> bool {
        self.receptive[self.water.index_of(x, y)]
    }

    /// Make a cell ice (water 1, or more if it already held more). Only cells short of the
    /// circle where growth stops can be frozen, so a stray click near the edge of the field
    /// can't end the growth.
    pub fn freeze(&mut self, x: usize, y: usize) {
        let d2 = distance2(x, y, self.water.width() / 2);
        if self.is_inside(x, y) && within(d2, self.radius - MARGIN) {
            let w = self.water.get(x, y).max(1.0);
            self.set(x, y, w);
        }
    }

    /// Set a cell's water, keeping the ice count and reach up to date. Only cells inside
    /// the boundary circle update, so the others are best left at the background.
    pub fn set(&mut self, x: usize, y: usize, w: f64) {
        let was_ice = self.water.get(x, y) >= 1.0;
        self.water.set(x, y, w);
        self.most = self.most.max(w);
        if w >= 1.0 && !was_ice {
            self.ice += 1;
            let c = self.water.width() / 2;
            self.reach2 = self.reach2.max(distance2(x, y, c));
        } else if w < 1.0 && was_ice {
            self.ice -= 1; // reach is not reduced: it only stops growth, and only grows
        }
    }

    /// The total water inside the boundary circle.
    #[cfg(test)]
    pub fn total_water(&self) -> f64 {
        self.rows
            .iter()
            .enumerate()
            .flat_map(|(y, r)| r.clone().map(move |x| (x, y)))
            .map(|(x, y)| self.water.get(x, y))
            .sum()
    }

    /// Cells that were receptive in the last step.
    #[cfg(test)]
    pub fn receptive_cells(&self) -> usize {
        self.receptive.iter().filter(|&&r| r).count()
    }

    /// Advance one step, unless growth has finished. Returns whether it stepped.
    pub fn step(&mut self) -> bool {
        if self.finished() {
            return false;
        }
        let n = self.water.width();
        let water = self.water.cells();
        // The neighbours' offsets in the flat array. HEX lists them in opposite pairs, so
        // one of each pair is enough: the other is its negative.
        let off = HEX.map(|(dx, dy)| dy * n as isize + dx);
        debug_assert!(off[0] == -off[1] && off[2] == -off[3] && off[4] == -off[5]);
        let (e, s, ne) = (off[0] as usize, off[2] as usize, off[5] as usize);

        for (y, r) in self.rows.iter().enumerate() {
            let row = y * n;
            for i in row + r.start..row + r.end {
                self.frozen[i] = water[i] >= 1.0;
            }
        }
        // Cells outside the circle are never frozen: growth stops MARGIN cells before it.
        for (y, r) in self.rows.iter().enumerate() {
            let row = y * n;
            for i in row + r.start..row + r.end {
                let f = &self.frozen;
                let rec = f[i] || f[i + e] || f[i - e] || f[i + s] || f[i - s];
                let rec = rec || f[i + ne] || f[i - ne];
                self.receptive[i] = rec;
                self.field[i] = if rec { 0.0 } else { water[i] };
            }
        }
        let (a12, gamma) = (self.params.alpha / 12.0, self.params.gamma);
        let c = (n / 2) as i64;
        let (mut ice, mut reach2, mut most) = (0, self.reach2, self.most);
        let next = self.next.cells_mut();
        for (y, r) in self.rows.iter().enumerate() {
            let row = y * n;
            for i in row + r.start..row + r.end {
                let v = &self.field;
                let sum = pair_sum(
                    v[i + e] + v[i - e],
                    v[i + s] + v[i - s],
                    v[i + ne] + v[i - ne],
                );
                let mut w = v[i] + a12 * (sum - 6.0 * v[i]);
                if self.receptive[i] {
                    w += water[i] + gamma;
                }
                next[i] = w;
                most = most.max(w);
                if w >= 1.0 {
                    ice += 1;
                    if !self.frozen[i] {
                        let (q, r) = ((i - row) as i64 - c, y as i64 - c);
                        reach2 = reach2.max(q * q + q * r + r * r);
                    }
                }
            }
        }
        std::mem::swap(&mut self.water, &mut self.next);
        (self.ice, self.reach2, self.most) = (ice, reach2, most);
        self.generation += 1;
        true
    }
}

/// The squared Euclidean distance of cell `(x, y)` from the centre `(c, c)`, in cell
/// widths. In axial coordinates `(q, r)` the cell sits at `(q + r/2, r * sqrt(3)/2)`, so this
/// is `q² + qr + r²`: an integer, so the boundary circle has exactly the hexagon's symmetry.
fn distance2(x: usize, y: usize, c: usize) -> i64 {
    let (q, r) = (x as i64 - c as i64, y as i64 - c as i64);
    q * q + q * r + r * r
}

/// Whether a squared distance `d2` (in cell widths, from [`distance2`]) lies strictly inside
/// a circle of `rows` hex rows: `d2 < (rows * sqrt(3)/2)²`, kept in integers so the circle is
/// exactly symmetric. Reiter's `cir` likewise counts a cell on the circle as outside.
fn within(d2: i64, rows: usize) -> bool {
    let rows = rows as i64;
    4 * d2 < 3 * rows * rows
}

/// The sum of a cell's six neighbours, given as the sums of its three opposite pairs.
///
/// Floating-point addition depends on order, and a turn of the hexagon visits a cell's
/// neighbours in a different order, which would let rounding break the crystal's symmetry.
/// Any of the hexagon's turns and mirrors maps opposite pairs to opposite pairs, and `a + b`
/// equals `b + a` exactly, so summing the three pair sums in sorted order gives a result
/// that depends only on the six values, not on which neighbour is which.
fn pair_sum(a: f64, b: f64, c: f64) -> f64 {
    let lo = a.min(b).min(c);
    let hi = a.max(b).max(c);
    let mid = a.min(b).max(a.max(b).min(c));
    (lo + mid) + hi
}

#[cfg(test)]
mod tests {
    use super::*;

    const STANDARD: Params = params(1.0, 0.4, 0.001);

    /// Cell `(x, y)` from offsets `(dq, dr)` from the centre.
    fn at(crystal: &Crystal, dq: isize, dr: isize) -> (usize, usize) {
        let (c, _) = crystal.center();
        (
            c.checked_add_signed(dq).unwrap(),
            c.checked_add_signed(dr).unwrap(),
        )
    }

    fn water_at(crystal: &Crystal, dq: isize, dr: isize) -> f64 {
        let (x, y) = at(crystal, dq, dr);
        crystal.water().get(x, y)
    }

    fn hex_distance(dq: isize, dr: isize) -> isize {
        (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
    }

    /// Run until growth finishes, or `limit` steps.
    fn grow(crystal: &mut Crystal, limit: u64) {
        while crystal.generation < limit && crystal.step() {}
    }

    /// The pair sums must give the same result in any order, to the last bit: the exact
    /// symmetry test below depends on it.
    #[test]
    fn pair_sum_ignores_order() {
        let (a, b, c) = (0.1, 1e-17, 0.7);
        let want = pair_sum(a, b, c);
        for (x, y, z) in [(a, c, b), (b, a, c), (b, c, a), (c, a, b), (c, b, a)] {
            assert_eq!(pair_sum(x, y, z).to_bits(), want.to_bits());
        }
        assert!((want - 0.8).abs() < 1e-15);
    }

    /// Reiter's Figure 2 works one step of the rule by hand on a 19-cell patch with
    /// `gamma = 0.1` (receptive 1.000 becomes 1.100). Its rows, top to bottom, are
    /// 3, 4, 5, 4, 3 cells; a row of `k` cells at axial row `r` starts at `q = -2 - min(r, 0)`
    /// relative to the middle cell. The figure's outer ring depends on cells outside the
    /// patch that it does not show, so only the middle cell and its six neighbours, whose
    /// inputs all lie in the patch, are compared; the patch sits in a field of 0. The
    /// figure's "receptive" panel is checked for all 19 cells: with no ice outside the patch,
    /// it is decided by the patch alone.
    #[test]
    fn figure_2_hand_worked_step() {
        let before: [&[f64]; 5] = [
            &[1.0, 1.0, 1.0],
            &[0.5, 1.0, 1.0, 1.0],
            &[0.2, 0.5, 1.0, 1.0, 1.0],
            &[0.2, 0.2, 0.5, 1.0],
            &[0.2, 0.2, 0.5],
        ];
        // The "receptive" panel: receptive cells show their water, the others 0.
        let receptive: [&[bool]; 5] = [
            &[true, true, true],
            &[true, true, true, true],
            &[false, true, true, true, true],
            &[false, true, true, true],
            &[false, false, true],
        ];
        // The "add together" panel, middle cell and its neighbours (q, r, value).
        let after = [
            (0, -1, 1.100),
            (1, -1, 1.100),
            (-1, 0, 0.633),
            (0, 0, 1.100),
            (1, 0, 1.100),
            (-1, 1, 0.350),
            (0, 1, 0.617),
        ];
        let mut crystal = Crystal::new(10, params(1.0, 0.0, 0.1));
        let cells = |r: isize, k: usize| (0..k as isize).map(move |i| (-2 - r.min(0) + i, r));
        for (r, row) in (-2..=2).zip(before) {
            for ((dq, dr), &w) in cells(r, row.len()).zip(row) {
                let (x, y) = at(&crystal, dq, dr);
                crystal.set(x, y, w);
            }
        }
        crystal.step();
        for (r, row) in (-2..=2).zip(receptive) {
            for ((dq, dr), &want) in cells(r, row.len()).zip(row) {
                let (x, y) = at(&crystal, dq, dr);
                assert_eq!(
                    crystal.was_receptive(x, y),
                    want,
                    "receptive at ({dq}, {dr})"
                );
            }
        }
        for (dq, dr, want) in after {
            let got = water_at(&crystal, dq, dr);
            assert!((got - want).abs() < 0.0005, "({dq}, {dr}): {got} != {want}");
        }
    }

    /// The first step from a lone seed, worked by hand from the rule. A neighbour weighs
    /// `alpha/12` and the cell itself `1 - alpha/2`. Receptive: the seed and its six
    /// neighbours.
    /// - The seed's neighbours are all receptive, so nothing diffuses in: 1 + gamma.
    /// - A neighbour of the seed keeps beta + gamma, plus alpha/12 of each of its three
    ///   non-receptive neighbours (beta each).
    /// - At hex distance 2, a cell on an axis keeps `1 - alpha/2` of its beta and gets
    ///   alpha/12 of the five of its neighbours that are not receptive; a cell between two
    ///   axes touches two receptive cells, so gets four.
    /// - Farther out nothing has changed: beta.
    ///
    /// At alpha = 1 (weights 1/2 and 1/12) that is 1.25 beta + gamma, 11/12 beta and 10/12
    /// beta; at alpha = 2 (weights 0 and 1/6) 1.5 beta + gamma, 5/6 beta and 4/6 beta.
    #[test]
    fn first_step_by_hand() {
        for (alpha, ring1, axis, between) in [
            (1.0, 1.25, 11.0 / 12.0, 10.0 / 12.0),
            (2.0, 1.5, 5.0 / 6.0, 4.0 / 6.0),
            (0.5, 1.125, 0.75 + 5.0 / 24.0, 0.75 + 4.0 / 24.0),
        ] {
            let p = Params { alpha, ..STANDARD };
            let Params { beta, gamma, .. } = p;
            let mut crystal = Crystal::new(20, p);
            crystal.step();
            for dr in -5..=5isize {
                for dq in -5..=5isize {
                    let on_axis = dq == 0 || dr == 0 || dq + dr == 0;
                    let want = match hex_distance(dq, dr) {
                        0 => 1.0 + gamma,
                        1 => ring1 * beta + gamma,
                        2 if on_axis => axis * beta,
                        2 => between * beta,
                        _ => beta,
                    };
                    let got = water_at(&crystal, dq, dr);
                    assert!(
                        (got - want).abs() < 1e-12,
                        "alpha {alpha} ({dq}, {dr}): {got} != {want}"
                    );
                }
            }
            assert_eq!((crystal.ice_cells(), crystal.receptive_cells()), (1, 7));
        }
    }

    /// Diffusion moves water but never makes or loses any: the only change in the total is
    /// the `gamma` added to each receptive cell. This holds until the disturbance reaches
    /// the boundary circle, which is held at the background (it spreads at most one cell a
    /// step, and only from the crystal).
    #[test]
    fn only_gamma_changes_the_total_water() {
        let mut crystal = Crystal::new(80, params(1.0, 0.35, 0.001));
        let mut total = crystal.total_water();
        for _ in 0..50 {
            crystal.step();
            let added = crystal.params.gamma * crystal.receptive_cells() as f64;
            let now = crystal.total_water();
            assert!(
                (now - total - added).abs() < 1e-9,
                "generation {}: {now} - {total} != {added}",
                crystal.generation
            );
            total = now;
        }
        assert!(crystal.ice_cells() > 1, "the test should cover some growth");
    }

    /// A crystal grown from a centred seed has the hexagon's twelve symmetries exactly, to
    /// the last bit: the rule treats all six neighbours alike, the boundary circle is
    /// symmetric (see `distance2`), and neighbour sums don't depend on order (see
    /// `pair_sum`). In axial coordinates a 60-degree turn maps (dq, dr) to (-dr, dq + dr) and
    /// swapping dq and dr is a mirror. A wrong neighbour offset would also break this.
    #[test]
    fn grown_crystal_has_six_fold_and_mirror_symmetry() {
        let mut crystal = Crystal::new(40, params(1.0, 0.35, 0.001));
        grow(&mut crystal, 3_000);
        assert!(
            crystal.ice_cells() > 200,
            "grew only {}",
            crystal.ice_cells()
        );
        let (c, _) = crystal.center();
        let c = c as isize;
        let mut compared = 0;
        for dr in -c..=c {
            for dq in -c..=c {
                let (x, y) = at(&crystal, dq, dr);
                if !crystal.is_inside(x, y) {
                    continue;
                }
                let here = water_at(&crystal, dq, dr).to_bits();
                let turned = water_at(&crystal, -dr, dq + dr).to_bits();
                let mirrored = water_at(&crystal, dr, dq).to_bits();
                assert_eq!(here, turned, "turning ({dq}, {dr})");
                assert_eq!(here, mirrored, "mirroring ({dq}, {dr})");
                compared += 1;
            }
        }
        assert_eq!(compared, crystal.active_cells());
    }

    /// Reiter: "When β = 0 and γ = 0, no growth appears." No vapour and no added water:
    /// the seed stays the only ice, and the water stays what the seed held.
    #[test]
    fn no_vapour_and_no_added_water_means_no_growth() {
        let mut crystal = Crystal::new(20, params(1.0, 0.0, 0.0));
        for _ in 0..500 {
            crystal.step();
        }
        assert_eq!(crystal.ice_cells(), 1);
        assert_eq!(crystal.total_water(), 1.0);
    }

    /// Reiter: "When β = 0 and γ > 0, plates form", and "regardless of β, at high levels of
    /// γ, we get plates". With beta = 0 nothing diffuses, so every cell of the ring around
    /// the ice gains gamma alike and freezes at the same step; with gamma = 1 every receptive
    /// cell freezes the step after it becomes receptive, whatever beta is. Either way the
    /// ice is always a perfect hexagon, centred on the seed.
    #[test]
    fn plates_are_hexagons() {
        for p in [
            params(1.0, 0.0, 0.05),
            params(1.0, 0.0, 1.0),
            params(1.0, 0.4, 1.0),
            params(1.0, 0.9, 1.0),
        ] {
            let mut crystal = Crystal::new(30, p);
            let mut largest = 0;
            for _ in 0..200 {
                crystal.step();
                // The ice must be every cell within some hex distance k, and no others.
                let k = (0..30)
                    .take_while(|&k| water_at(&crystal, k, 0) >= 1.0)
                    .last()
                    .unwrap();
                let hexagon = 3 * k * (k + 1) + 1;
                assert_eq!(crystal.ice_cells(), hexagon as usize, "{p:?}");
                for dr in -k..=k {
                    for dq in -k..=k {
                        if hex_distance(dq, dr) <= k {
                            assert!(water_at(&crystal, dq, dr) >= 1.0, "{p:?} ({dq}, {dr})");
                        }
                    }
                }
                largest = largest.max(k);
            }
            assert!(largest >= 5, "{p:?}: the plate should grow");
        }
    }

    /// The smallest distance `k` along an axis (in cell widths) at which an ice cell
    /// ends growth: the first not strictly inside the circle `radius - MARGIN` rows out.
    fn stop_distance(crystal: &Crystal) -> isize {
        (0..)
            .find(|&k: &isize| !within((k * k) as i64, crystal.radius() - MARGIN))
            .unwrap()
    }

    /// Ice reaching the circle stops growth, so it never gets near the cells held at beta.
    #[test]
    fn growth_stops_near_the_boundary() {
        let mut crystal = Crystal::new(12, params(1.0, 0.0, 1.0));
        grow(&mut crystal, 1_000);
        assert!(crystal.finished());
        // A gamma = 1 plate grows one ring a step, and its corners, on the axes, are its
        // farthest cells. 10 rows is 8.66 cell widths, so the corners stop it at 9.
        assert_eq!(stop_distance(&crystal), 9);
        assert_eq!(crystal.generation, 9);
        let ice = crystal.ice_cells();
        assert!(!crystal.step());
        assert_eq!(crystal.ice_cells(), ice);
    }

    /// Freezing a cell where growth would stop must do nothing; just inside, it must work.
    #[test]
    fn freezing_near_the_edge_is_refused() {
        let mut crystal = Crystal::new(20, STANDARD);
        let stop = stop_distance(&crystal);
        let (x, y) = at(&crystal, stop, 0);
        crystal.freeze(x, y);
        assert_eq!((crystal.ice_cells(), crystal.finished()), (1, false));
        let (x, y) = at(&crystal, stop - 1, 0);
        crystal.freeze(x, y);
        assert_eq!((crystal.ice_cells(), crystal.finished()), (2, false));
    }

    /// Reiter's Figure 4 (beta = 0.3, gamma = 0.0001) and his text: at low beta and low
    /// gamma the model grows dendrites, not plates. The ice should be six thin arms, out to
    /// the boundary along the six axes, covering only a small part of the disc they span.
    #[test]
    fn thin_dendrite_grows_six_arms() {
        let mut crystal = Crystal::new(50, PRESETS[0].params);
        grow(&mut crystal, 20_000);
        assert!(
            crystal.finished(),
            "still growing at {}",
            crystal.generation
        );
        // The disc out to the farthest ice, in cell widths, holds pi r² / (sqrt(3)/2) cells.
        let r = crystal.reach() * 3f64.sqrt() / 2.0;
        let disc = std::f64::consts::PI * r * r / (3f64.sqrt() / 2.0);
        let fraction = crystal.ice_cells() as f64 / disc;
        assert!(fraction < 0.3, "ice covers {fraction:.2} of its disc");
        let tip = stop_distance(&crystal) - 2;
        for (dq, dr) in HEX {
            assert!(
                water_at(&crystal, dq * tip, dr * tip) >= 1.0,
                "arm ({dq}, {dr})"
            );
        }
    }

    /// Reiter's auxiliary page has tables of crystals grown to radius 100, 200 and 400
    /// (<https://webbox.lafayette.edu/~reiterc/mvp/sfn/nn/index.html>, `n/` and `nw/`).
    /// Each picture's file name ends in the step its growth stopped at, as his script
    /// returns it: one less than the steps taken (`mk_dla` returns `k-1`). So
    /// `n_h0o6_x0o01_00856.png` (radius 200, beta 0.6, gamma 0.01) means 857 steps here.
    /// These catch any change to the rule, or to the size and units of the circles.
    #[test]
    fn step_counts_match_reiters_tables() {
        for (radius, beta, gamma, file_step) in [
            (100, 0.9, 0.05, 84),  // nn_h0o9_x0o05_00084.png
            (200, 0.9, 0.05, 171), // n_h0o9_x0o05_00171.png
            (200, 0.6, 0.01, 856), // n_h0o6_x0o01_00856.png
        ] {
            let mut crystal = Crystal::new(radius, params(1.0, beta, gamma));
            grow(&mut crystal, 20_000);
            assert!(crystal.finished());
            assert_eq!(
                crystal.generation,
                file_step + 1,
                "radius {radius}, beta {beta}, gamma {gamma}"
            );
        }
    }

    /// The same, for the slow ones: the dendrites at radius 200 (thousands of steps each),
    /// and the sector at radius 400.
    #[test]
    #[ignore = "slow: about 17,000 steps of a 94,000-cell field"]
    fn slow_step_counts_match_reiters_tables() {
        for (radius, beta, gamma, file_step) in [
            (200, 0.3, 0.0001, 7814), // n_h0o3_x0o0001_07814.png
            (200, 0.35, 0.001, 5049), // n_h0o35_x0o001_05049.png
            (200, 0.4, 0.0001, 3726), // n_h0o4_x0o0001_03726.png
            (400, 0.9, 0.05, 344),    // nw_h0o9_x0o05_00344.png
        ] {
            let mut crystal = Crystal::new(radius, params(1.0, beta, gamma));
            grow(&mut crystal, 20_000);
            assert!(crystal.finished());
            assert_eq!(
                crystal.generation,
                file_step + 1,
                "radius {radius}, beta {beta}, gamma {gamma}"
            );
        }
    }

    /// Figure 5's alpha = 2.003 (beta 0.4, gamma 0.0001) at radius 100, the only test of a
    /// diffusion other than alpha = 1 over a whole growth. The counts were found
    /// independently by a port of Reiter's J script, run with his circles.
    #[test]
    fn many_levels_at_radius_100() {
        let mut crystal = Crystal::new(100, PRESETS[5].params);
        grow(&mut crystal, 20_000);
        assert!(crystal.finished());
        assert_eq!((crystal.generation, crystal.ice_cells()), (914, 4027));
    }
}
