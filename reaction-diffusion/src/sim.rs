//! The Gray–Scott model, as John Pearson ran it in "Complex patterns in a simple system",
//! Science 261 (1993) 189–192 (arXiv:patt-sol/9304003; page numbers below are the arXiv
//! preprint's).
//!
//! Two chemicals, U and V, react by `U + 2V -> 3V` and `V -> P` (Pearson's equation 1).
//! U is fed in and both are removed at the feed rate F; k is the rate of the second
//! reaction. In dimensionless units (equation 2):
//!
//! ```text
//! dU/dt = Du ∇²U − U V² + F (1 − U)
//! dV/dt = Dv ∇²V + U V² − (F + k) V
//! ```
//!
//! Pearson's numbers (p. 3): a 2.5 x 2.5 square on a 256 x 256 mesh, so a mesh spacing of
//! `2.5 / 256`; `Du = 2e-5`, `Dv = 1e-5`; periodic boundaries; forward Euler with a time
//! step of 1. The paper says only "discretization of the diffusion operator"; the five-point
//! Laplacian here is the usual choice, not one it names.
//!
//! In mesh units the diffusion enters only as `D Δt / Δx²`: 0.2097152 for U and 0.1048576
//! for V. Every world size keeps Pearson's spacing, so a bigger world is a bigger dish, not
//! a finer mesh, and patterns keep their size in cells. (A finer mesh would also break
//! forward Euler: it is stable only while `4 D Δt / Δx²` is at most 1, and U is at 0.84.)
//!
//! **Speed.** U and V are separate `f32` arrays, each with a second buffer the step writes
//! into. Each row's interior is a loop over equal-length slices, with no wrapping and no
//! bounds checks, which the compiler turns into SIMD (four cells at once in the browser,
//! with the `simd128` feature `.cargo/config.toml` turns on); the two edge columns, which
//! wrap, are done one by one with the same arithmetic.
//!
//! **Exact symmetry.** The Laplacian is summed as `(left + right) + (up + down)`. Floating
//! point addition is commutative though not associative, so with that grouping a mirror
//! image or a quarter turn of the world gives bit-for-bit the mirrored or turned result,
//! and the symmetry tests can compare exactly.

use ca_core::Rng;

/// Pearson's mesh spacing: 2.5 units across 256 cells.
pub const DX: f64 = 2.5 / 256.0;
/// Pearson's diffusion coefficients and time step.
pub const DU: f64 = 2e-5;
pub const DV: f64 = 1e-5;
pub const DT: f64 = 1.0;

/// `D Δt / Δx²` for each chemical: the diffusion per step, in mesh units.
const RATE_U: f32 = (DU * DT / (DX * DX)) as f32;
const RATE_V: f32 = (DV * DT / (DX * DX)) as f32;
// Forward Euler with the five-point Laplacian is stable only while 4 D Δt / Δx² <= 1.
const _: () = assert!(4.0 * RATE_U <= 1.0 && 4.0 * RATE_V <= 1.0);

/// Pearson's opening (p. 3): the 20 x 20 square at the centre set to this, the rest at the
/// trivial state `U = 1, V = 0`. Drawing seeds the same mixture.
pub const SEED_UV: (f32, f32) = (0.5, 0.25);
pub const SEED_SIDE: usize = 20;
/// "perturbed with ±1% random noise": read here as every value multiplied by a factor
/// between 0.99 and 1.01, so V stays 0 outside the square (V can't be negative).
const NOISE: f64 = 0.01;

/// The feed rate F and the rate constant k (Karl Sims's "kill" rate).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rates {
    pub f: f32,
    pub k: f32,
}

/// Pearson's twelve pattern classes, named by Greek letter as in his figure 2, each at a
/// point of his figure 3 where he found it. Pearson gives F and k in his text only for ε
/// (F = 0.02, k = 0.059, p. 5); the others are read off figure 3, whose letters are text in
/// the arXiv PDF: each letter's left edge lies on its k and its middle on its F, as the
/// tick labels show, which gives ε's point to within 0.0001. [`FIGURE_3`] has every point.
pub struct Class {
    pub letter: &'static str,
    pub rates: Rates,
    /// What Pearson says of it (pp. 4–5), shortened.
    pub note: &'static str,
}

pub const CLASSES: [Class; 12] = [
    Class {
        letter: "α",
        rates: Rates { f: 0.020, k: 0.050 },
        note: "Fledgling spirals that keep colliding and annihilating each other: full spirals never form.",
    },
    Class {
        letter: "β",
        rates: Rates { f: 0.015, k: 0.040 },
        note: "Phase turbulence: the dish oscillates, but out of step, so the phase varies from place to place.",
    },
    Class {
        letter: "γ",
        rates: Rates { f: 0.025, k: 0.055 },
        note: "Mostly stripes, with small patches that oscillate fast; they die out, but new ones keep appearing.",
    },
    Class {
        letter: "δ",
        rates: Rates { f: 0.030, k: 0.055 },
        note: "Regular hexagons, apart from defects that seem to be stable.",
    },
    Class {
        letter: "ε",
        rates: Rates { f: 0.020, k: 0.059 },
        note: "Spots that grow and divide, and die when crowded; the holes fill again. No order in time or space: chaos.",
    },
    Class {
        letter: "ζ",
        rates: Rates { f: 0.025, k: 0.060 },
        note: "Dividing spots that settle into order across the dish, apart from patches of activity that wander.",
    },
    Class {
        letter: "η",
        rates: Rates {
            f: 0.035,
            k: 0.0625,
        },
        note: "Stripes that stand still, except a few that oscillate without dying down.",
    },
    Class {
        letter: "θ",
        rates: Rates { f: 0.040, k: 0.060 },
        note: "Waves spread outwards and stop where they meet, like those in Lee and colleagues' experiments.",
    },
    Class {
        letter: "ι",
        rates: Rates { f: 0.050, k: 0.060 },
        note: "A pattern that stands still. Pearson saw it at this one point only.",
    },
    Class {
        letter: "κ",
        rates: Rates {
            f: 0.050,
            k: 0.0625,
        },
        note: "Stripes spread outwards, and stop where they meet.",
    },
    Class {
        letter: "λ",
        rates: Rates { f: 0.040, k: 0.065 },
        note: "Spots that divide until they fill the dish, then stand still.",
    },
    Class {
        letter: "μ",
        rates: Rates { f: 0.050, k: 0.065 },
        note: "Stripes grow at their tips, and turn aside rather than run into each other.",
    },
];

/// What Pearson found at a point of figure 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Found {
    /// The dish went back to the trivial state everywhere ("R", red in his pictures).
    Red,
    /// It went to the other uniform state, U about 0.3 ("B", blue).
    Blue,
    /// One of the pattern classes, by its index in [`CLASSES`].
    Pattern(usize),
}

/// Every point of Pearson's figure 3, as `(k, F, what he found)`, read off the arXiv PDF as
/// [`Class`] describes.
pub const FIGURE_3: &[(f32, f32, Found)] = {
    use Found::{Blue as B, Pattern as P, Red as R};
    &[
        (0.0400, 0.0200, B),
        (0.0450, 0.0240, B),
        (0.0500, 0.0300, B),
        (0.0550, 0.0350, B),
        (0.0575, 0.0440, B),
        (0.0585, 0.0500, B),
        (0.0600, 0.0520, B),
        (0.0600, 0.0550, B),
        (0.0600, 0.0600, B),
        (0.0400, 0.0050, R),
        (0.0400, 0.0100, R),
        (0.0450, 0.0040, R),
        (0.0450, 0.0120, R),
        (0.0450, 0.0160, R),
        (0.0500, 0.0070, R),
        (0.0550, 0.0100, R),
        (0.0590, 0.0150, R),
        (0.0600, 0.0200, R),
        (0.0625, 0.0200, R),
        (0.0625, 0.0700, R),
        (0.0650, 0.0200, R),
        (0.0650, 0.0250, R),
        (0.0650, 0.0300, R),
        (0.0650, 0.0600, R),
        (0.0675, 0.0350, R),
        (0.0675, 0.0400, R),
        (0.0675, 0.0450, R),
        (0.0675, 0.0500, R),
        (0.0675, 0.0550, R),
        (0.0450, 0.0080, P(0)),
        (0.0500, 0.0150, P(0)),
        (0.0500, 0.0200, P(0)),
        (0.0400, 0.0150, P(1)),
        (0.0450, 0.0200, P(1)),
        (0.0500, 0.0250, P(1)),
        (0.0550, 0.0250, P(2)),
        (0.0550, 0.0300, P(3)),
        (0.0500, 0.0100, P(4)),
        (0.0550, 0.0150, P(4)),
        (0.0550, 0.0200, P(4)),
        (0.0590, 0.0200, P(4)),
        (0.0625, 0.0250, P(4)),
        (0.0625, 0.0300, P(4)),
        (0.0600, 0.0250, P(5)),
        (0.0600, 0.0300, P(6)),
        (0.0625, 0.0350, P(6)),
        (0.0590, 0.0300, P(7)),
        (0.0590, 0.0350, P(7)),
        (0.0590, 0.0400, P(7)),
        (0.0600, 0.0400, P(7)),
        (0.0600, 0.0450, P(7)),
        (0.0600, 0.0500, P(8)),
        (0.0600, 0.0350, P(9)),
        (0.0625, 0.0400, P(9)),
        (0.0625, 0.0450, P(9)),
        (0.0625, 0.0500, P(9)),
        (0.0625, 0.0550, P(9)),
        (0.0625, 0.0625, P(9)),
        (0.0650, 0.0350, P(10)),
        (0.0650, 0.0400, P(10)),
        (0.0650, 0.0450, P(11)),
        (0.0650, 0.0500, P(11)),
        (0.0650, 0.0550, P(11)),
    ]
};

/// The two chemicals on a `width` x `height` grid that wraps around both ways.
pub struct Dish {
    width: usize,
    height: usize,
    u: Vec<f32>,
    v: Vec<f32>,
    /// The buffers the next step writes; after a step, the state before it.
    u_next: Vec<f32>,
    v_next: Vec<f32>,
    /// Whether `u_next` still holds the state one step back (no step yet, or the dish has
    /// been drawn on since).
    has_previous: bool,
    pub rates: Rates,
    pub steps: u64,
}

impl Dish {
    /// A dish in the trivial state, `U = 1, V = 0` everywhere.
    pub fn new(width: usize, height: usize, rates: Rates) -> Self {
        assert!(width >= 3 && height >= 3, "the stencil needs 3 x 3 cells");
        let n = width * height;
        Self {
            width,
            height,
            u: vec![1.0; n],
            v: vec![0.0; n],
            u_next: vec![1.0; n],
            v_next: vec![0.0; n],
            has_previous: false,
            rates,
            steps: 0,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn u(&self) -> &[f32] {
        &self.u
    }

    #[cfg(test)]
    pub fn v(&self) -> &[f32] {
        &self.v
    }

    pub fn get(&self, x: usize, y: usize) -> (f32, f32) {
        let i = y * self.width + x;
        (self.u[i], self.v[i])
    }

    pub fn set(&mut self, x: usize, y: usize, uv: (f32, f32)) {
        let i = y * self.width + x;
        (self.u[i], self.v[i]) = uv;
        self.has_previous = false;
    }

    /// Back to the trivial state everywhere, at step 0.
    pub fn clear(&mut self) {
        self.u.fill(1.0);
        self.v.fill(0.0);
        self.has_previous = false;
        self.steps = 0;
    }

    /// The centred square of [`SEED_SIDE`] cells set to [`SEED_UV`], with no noise.
    pub fn seed_square(&mut self) {
        let (x0, y0) = (
            (self.width - SEED_SIDE.min(self.width)) / 2,
            (self.height - SEED_SIDE.min(self.height)) / 2,
        );
        for y in y0..(y0 + SEED_SIDE).min(self.height) {
            for x in x0..(x0 + SEED_SIDE).min(self.width) {
                self.set(x, y, SEED_UV);
            }
        }
    }

    /// Pearson's opening: the trivial state, the centred square, then ±1% noise everywhere
    /// "to break the square symmetry".
    pub fn pearson_opening(&mut self, rng: &mut Rng) {
        self.clear();
        self.seed_square();
        let mut wobble = || (1.0 + NOISE * (2.0 * rng.next_f64() - 1.0)) as f32;
        for (u, v) in self.u.iter_mut().zip(self.v.iter_mut()) {
            *u *= wobble();
            *v *= wobble();
        }
    }

    /// Set every cell within `radius` of `(cx, cy)` (wrapping round the edges) to `uv`.
    pub fn paint_disc(&mut self, cx: usize, cy: usize, radius: f32, uv: (f32, f32)) {
        let r = radius.max(0.5);
        let reach = r.ceil() as isize;
        let (w, h) = (self.width as isize, self.height as isize);
        for dy in -reach..=reach {
            for dx in -reach..=reach {
                if ((dx * dx + dy * dy) as f32) <= r * r {
                    let x = (cx as isize + dx).rem_euclid(w) as usize;
                    let y = (cy as isize + dy).rem_euclid(h) as usize;
                    self.set(x, y, uv);
                }
            }
        }
    }

    /// The mean of |ΔU| over the last step, or `None` if there was no last step (or the dish
    /// has been drawn on since): how fast the pattern is still changing.
    pub fn change(&self) -> Option<f64> {
        self.has_previous.then(|| {
            let total: f64 = self
                .u
                .iter()
                .zip(&self.u_next)
                .map(|(a, b)| f64::from((a - b).abs()))
                .sum();
            total / self.u.len() as f64
        })
    }

    /// One forward-Euler step of equation 2.
    pub fn step(&mut self) {
        let (w, h) = (self.width, self.height);
        let Rates { f, k } = self.rates;
        for y in 0..h {
            let above = if y == 0 { h - 1 } else { y - 1 };
            let below = if y == h - 1 { 0 } else { y + 1 };
            fn row(a: &[f32], r: usize, w: usize) -> &[f32] {
                &a[r * w..(r + 1) * w]
            }
            let row = |a, r| row(a, r, w);
            let rows = Rows {
                u: [row(&self.u, above), row(&self.u, y), row(&self.u, below)],
                v: [row(&self.v, above), row(&self.v, y), row(&self.v, below)],
            };
            let out_u = &mut self.u_next[y * w..(y + 1) * w];
            let out_v = &mut self.v_next[y * w..(y + 1) * w];
            step_row(&rows, out_u, out_v, f, k);
        }
        std::mem::swap(&mut self.u, &mut self.u_next);
        std::mem::swap(&mut self.v, &mut self.v_next);
        self.has_previous = true;
        self.steps += 1;
    }

    /// Totals of U and of V over the dish, in `f64`.
    #[cfg(test)]
    pub fn totals(&self) -> (f64, f64) {
        let sum = |a: &[f32]| a.iter().map(|&x| f64::from(x)).sum::<f64>();
        (sum(&self.u), sum(&self.v))
    }
}

/// A row and the rows above and below it, of each chemical: `[above, row, below]`.
struct Rows<'a> {
    u: [&'a [f32]; 3],
    v: [&'a [f32]; 3],
}

/// One cell's new `(U, V)`, from its own values and its Laplacians. The edge columns and the
/// SIMD interior both use this, so they compute alike.
#[inline(always)]
fn update(u: f32, lap_u: f32, v: f32, lap_v: f32, f: f32, k: f32) -> (f32, f32) {
    let uvv = u * v * v;
    (
        u + RATE_U * lap_u - uvv + f * (1.0 - u),
        flush(v + RATE_V * lap_v + uvv - (f + k) * v),
    )
}

/// `v`, or 0 if it is smaller than the smallest normal `f32` (in size: a negative `v` is kept,
/// or V wouldn't be conserved where it goes negative). Where V dies away it would otherwise
/// sink through the subnormal numbers, which many processors handle many times slower.
#[inline(always)]
fn flush(v: f32) -> f32 {
    if v.abs() < f32::MIN_POSITIVE { 0.0 } else { v }
}

/// The five-point Laplacian, grouped for exact symmetry (see the module comment).
#[inline(always)]
fn laplacian(left: f32, right: f32, up: f32, down: f32, centre: f32) -> f32 {
    (left + right) + (up + down) - 4.0 * centre
}

fn step_row(rows: &Rows<'_>, out_u: &mut [f32], out_v: &mut [f32], f: f32, k: f32) {
    let w = out_u.len();
    let [ua, um, ub] = rows.u;
    let [va, vm, vb] = rows.v;
    // The edge columns wrap.
    for x in [0, w - 1] {
        let (l, r) = (if x == 0 { w - 1 } else { x - 1 }, (x + 1) % w);
        let lap_u = laplacian(um[l], um[r], ua[x], ub[x], um[x]);
        let lap_v = laplacian(vm[l], vm[r], va[x], vb[x], vm[x]);
        (out_u[x], out_v[x]) = update(um[x], lap_u, vm[x], lap_v, f, k);
    }
    // The interior, x = 1 to w - 2, as slices of one length, which the compiler vectorises.
    let n = w - 2;
    let (ul, uc, ur, uu, ud) = (&um[..n], &um[1..=n], &um[2..], &ua[1..=n], &ub[1..=n]);
    let (vl, vc, vr, vu, vd) = (&vm[..n], &vm[1..=n], &vm[2..], &va[1..=n], &vb[1..=n]);
    let (ou, ov) = (&mut out_u[1..=n], &mut out_v[1..=n]);
    for i in 0..n {
        let lap_u = laplacian(ul[i], ur[i], uu[i], ud[i], uc[i]);
        let lap_v = laplacian(vl[i], vr[i], vu[i], vd[i], vc[i]);
        (ou[i], ov[i]) = update(uc[i], lap_u, vc[i], lap_v, f, k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOME: Rates = Rates { f: 0.04, k: 0.06 };

    /// The coefficients are Pearson's: `D Δt / Δx²` with his numbers, worked by hand:
    /// `(2.5 / 256)² = 9.5367431640625e-5`, so `2e-5 / 9.5367...e-5 = 0.2097152`.
    #[test]
    fn diffusion_rates_are_pearsons() {
        assert!((RATE_U - 0.209_715_2).abs() < 1e-7);
        assert!((RATE_V - 0.104_857_6).abs() < 1e-7);
    }

    /// The trivial state, U = 1 and V = 0, is a fixed point for every F and k (Pearson p. 3:
    /// "A trivial steady state U = 1, V = 0 exists ... for all F and k"): nothing reacts, and
    /// a uniform field doesn't diffuse. It must stay exactly as it is.
    #[test]
    fn the_trivial_state_stays_put() {
        for rates in CLASSES.iter().map(|c| c.rates).chain([SOME]) {
            let mut d = Dish::new(17, 12, rates);
            for _ in 0..200 {
                d.step();
            }
            assert!(d.u().iter().all(|&u| u == 1.0), "{rates:?}");
            assert!(d.v().iter().all(|&v| v == 0.0), "{rates:?}");
        }
    }

    /// The other uniform steady state, from equation 2 with the diffusion dropped: `U V² =
    /// F (1 − U)` and `U V² = (F + k) V` give `U V = F + k`, so `U (1 − U) = (F + k)² / F` and
    /// `V = F (1 − U) / (F + k)`; the smaller root of U is Pearson's blue state.
    fn blue_state(rates: Rates) -> Option<(f64, f64)> {
        let (f, k) = (f64::from(rates.f), f64::from(rates.k));
        let disc = 1.0 - 4.0 * (f + k).powi(2) / f;
        (disc >= 0.0).then(|| {
            let u = (1.0 - disc.sqrt()) / 2.0;
            (u, f * (1.0 - u) / (f + k))
        })
    }

    /// At a "B" point of figure 3 (k = 0.05, F = 0.03), a dish filled with the blue state
    /// stays put, and that state is what Pearson describes: "roughly (U = .3, V = .25)" (p. 4).
    #[test]
    fn the_blue_state_stays_put() {
        let rates = Rates { f: 0.03, k: 0.05 };
        let (u, v) = blue_state(rates).unwrap();
        assert!(
            (u - 0.3).abs() < 0.05 && (v - 0.25).abs() < 0.05,
            "({u}, {v})"
        );
        let mut d = Dish::new(16, 16, rates);
        for y in 0..16 {
            for x in 0..16 {
                d.set(x, y, (u as f32, v as f32));
            }
        }
        for _ in 0..1_000 {
            d.step();
        }
        for (&du, &dv) in d.u().iter().zip(d.v()) {
            assert!((f64::from(du) - u).abs() < 1e-5 && (f64::from(dv) - v).abs() < 1e-5);
        }
    }

    /// A dish of uniform random values with the given chemical held at 0 everywhere.
    fn random_dish(w: usize, h: usize, rates: Rates, seed: u64) -> Dish {
        let mut d = Dish::new(w, h, rates);
        let mut rng = Rng::new(seed);
        for y in 0..h {
            for x in 0..w {
                d.set(x, y, (rng.next_f64() as f32, rng.next_f64() as f32));
            }
        }
        d
    }

    /// Diffusion only moves a chemical around a dish that wraps, so it keeps the total. With
    /// V = 0 and F = 0, U's equation is diffusion alone (every other term has a factor V or
    /// F); with U = 0 and F = k = 0, so is V's.
    #[test]
    fn diffusion_conserves_each_chemical() {
        let off = Rates { f: 0.0, k: 0.0 };
        let mut d = random_dish(23, 19, off, 1);
        d.v.fill(0.0);
        let before = d.totals().0;
        for _ in 0..2_000 {
            d.step();
        }
        assert!((d.totals().0 - before).abs() < 1e-3, "U total moved");
        assert!(d.v().iter().all(|&v| v == 0.0));
        let spread = |a: &[f32]| {
            a.iter().fold(0f32, |m, &x| m.max(x)) - a.iter().fold(1f32, |m, &x| m.min(x))
        };
        assert!(spread(d.u()) < 0.01, "U should have evened out");

        let mut d = random_dish(23, 19, off, 2);
        d.u.fill(0.0);
        let before = d.totals().1;
        for _ in 0..2_000 {
            d.step();
        }
        assert!((d.totals().1 - before).abs() < 1e-3, "V total moved");
        assert!(d.u().iter().all(|&u| u == 0.0));
    }

    /// With no feed and no second reaction, the only reaction is `U + 2V -> 3V`, which turns
    /// U into V one for one: the total of U + V stays as it was, while V grows.
    #[test]
    fn without_feed_and_removal_u_plus_v_is_conserved() {
        let mut d = random_dish(20, 20, Rates { f: 0.0, k: 0.0 }, 3);
        let (u0, v0) = d.totals();
        for _ in 0..2_000 {
            d.step();
        }
        let (u1, v1) = d.totals();
        assert!(
            ((u1 + v1) - (u0 + v0)).abs() < 1e-3,
            "{} -> {}",
            u0 + v0,
            u1 + v1
        );
        assert!(v1 > v0 + 10.0, "U should have turned into V");
    }

    /// One step worked by hand. On a 4 x 4 dish in the trivial state, one cell holds
    /// (U, V) = (0.5, 0.25); F = 0.04, k = 0.06.
    /// - That cell: ∇²U = 4 − 2 = 2 and ∇²V = −1, U V² = 0.03125, so
    ///   U = 0.5 + 0.2097152 × 2 − 0.03125 + 0.04 × 0.5 = 0.9081804 and
    ///   V = 0.25 − 0.1048576 + 0.03125 − 0.1 × 0.25 = 0.1513924.
    /// - Each of its four neighbours: ∇²U = −0.5 and ∇²V = 0.25, nothing reacts, so
    ///   U = 1 − 0.1048576 = 0.8951424 and V = 0.0262144.
    /// - Every other cell is unchanged.
    #[test]
    fn a_first_step_by_hand() {
        let mut d = Dish::new(4, 4, SOME);
        d.set(1, 2, (0.5, 0.25));
        d.step();
        let close = |(u, v): (f32, f32), (eu, ev): (f32, f32)| {
            (u - eu).abs() < 1e-6 && (v - ev).abs() < 1e-6
        };
        for y in 0..4 {
            for x in 0..4 {
                let expected = match (x, y) {
                    (1, 2) => (0.908_180_4, 0.151_392_4),
                    (0, 2) | (2, 2) | (1, 1) | (1, 3) => (0.895_142_4, 0.026_214_4),
                    _ => (1.0, 0.0),
                };
                assert!(
                    close(d.get(x, y), expected),
                    "({x}, {y}): {:?}",
                    d.get(x, y)
                );
            }
        }
        assert_eq!(d.steps, 1);
    }

    /// The opening without noise is symmetric under the square's mirrors and quarter turns,
    /// and so the model keeps it so: exactly, since the Laplacian's grouping makes every
    /// cell's arithmetic the mirror of its image's.
    #[test]
    fn a_symmetric_seed_stays_symmetric() {
        let n = 64;
        let mut d = Dish::new(n, n, CLASSES[10].rates);
        d.seed_square();
        for _ in 0..1_500 {
            d.step();
        }
        let (u, v) = (d.u(), d.v());
        assert!(v.iter().any(|&v| v > 0.1), "the seed should still be there");
        for y in 0..n {
            for x in 0..n {
                let at = |x: usize, y: usize| (u[y * n + x], v[y * n + x]);
                let here = at(x, y);
                assert_eq!(here, at(n - 1 - x, y), "left-right at ({x}, {y})");
                assert_eq!(here, at(x, n - 1 - y), "up-down at ({x}, {y})");
                assert_eq!(here, at(y, x), "diagonal at ({x}, {y})");
            }
        }
    }

    /// Pearson's opening: 400 cells at (1/2, 1/4) give or take 1%, the rest at (1, 0) give or
    /// take 1% in U, and fresh noise each time.
    #[test]
    fn the_opening_is_pearsons() {
        let mut d = Dish::new(256, 256, SOME);
        d.pearson_opening(&mut Rng::new(1));
        let seeded: Vec<(f32, f32)> = (0..256 * 256)
            .map(|i| (d.u()[i], d.v()[i]))
            .filter(|&(_, v)| v > 0.0)
            .collect();
        assert_eq!(seeded.len(), SEED_SIDE * SEED_SIDE);
        assert!(
            seeded
                .iter()
                .all(|&(u, v)| (u - 0.5).abs() <= 0.005 && (v - 0.25).abs() <= 0.0025)
        );
        assert!(d.get(118, 118).1 > 0.0 && d.get(137, 137).1 > 0.0 && d.get(117, 128).1 == 0.0);
        assert!(d.u().iter().all(|&u| u > 0.49 && u <= 1.01));
        let first = d.u().to_vec();
        d.pearson_opening(&mut Rng::new(2));
        assert_ne!(first, d.u(), "the noise should be drawn afresh");
        assert_eq!(d.steps, 0);
    }

    /// Drawing near an edge wraps round to the far side, like the dish.
    #[test]
    fn a_disc_wraps_round_the_edges() {
        let mut d = Dish::new(10, 10, SOME);
        d.paint_disc(0, 0, 1.0, SEED_UV);
        let painted: Vec<(usize, usize)> = (0..100)
            .map(|i| (i % 10, i / 10))
            .filter(|&(x, y)| d.get(x, y) == SEED_UV)
            .collect();
        assert_eq!(painted, [(0, 0), (1, 0), (9, 0), (0, 1), (0, 9)]);
    }

    /// The change per step is only known just after a step.
    #[test]
    fn change_is_known_only_after_a_step() {
        let mut d = Dish::new(8, 8, SOME);
        assert_eq!(d.change(), None);
        d.step();
        assert_eq!(d.change(), Some(0.0));
        d.set(3, 3, SEED_UV);
        assert_eq!(d.change(), None);
        d.step();
        assert!(d.change().unwrap() > 0.0);
    }

    /// The sizes of the patches where V is above `level`, joined across edges as the dish is.
    fn patches(d: &Dish, level: f32) -> Vec<usize> {
        let (w, h) = (d.width(), d.height());
        let mut seen: Vec<bool> = d.v().iter().map(|&v| v <= level).collect();
        let mut sizes = Vec::new();
        for start in 0..w * h {
            if seen[start] {
                continue;
            }
            seen[start] = true;
            let (mut stack, mut size) = (vec![start], 0);
            while let Some(i) = stack.pop() {
                size += 1;
                let (x, y) = (i % w, i / w);
                for (nx, ny) in [
                    ((x + 1) % w, y),
                    ((x + w - 1) % w, y),
                    (x, (y + 1) % h),
                    (x, (y + h - 1) % h),
                ] {
                    let j = ny * w + nx;
                    if !seen[j] {
                        seen[j] = true;
                        stack.push(j);
                    }
                }
            }
            sizes.push(size);
        }
        sizes
    }

    /// Pearson's run: his opening on a 256 x 256 dish, `steps` steps at `rates`.
    fn pearson_run(rates: Rates, steps: u64) -> Dish {
        let mut d = Dish::new(256, 256, rates);
        d.pearson_opening(&mut Rng::new(1993));
        for _ in 0..steps {
            d.step();
        }
        d
    }

    /// Figure 3's uniform outcomes: from Pearson's opening, at every "R" point the dish goes
    /// back to the trivial state, and at every "B" point it fills with the blue state. On a
    /// 64 x 64 dish (the outcome needs no room to show), for 30,000 steps.
    ///
    /// One point is left out: the R at k = 0.045, F = 0.016, between β above and α below.
    /// There the outcome depends on the noise: on Pearson's 256 x 256 dish for his 200,000
    /// steps, three of five random openings died away, as he found, and two lived on.
    #[test]
    #[ignore]
    fn figure_3_uniform_outcomes() {
        for &(k, f, found) in FIGURE_3 {
            if matches!(found, Found::Pattern(_)) {
                continue;
            }
            if (k, f) == (0.045, 0.016) {
                continue;
            }
            let rates = Rates { f, k };
            let mut d = Dish::new(64, 64, rates);
            d.pearson_opening(&mut Rng::new(1993));
            for _ in 0..30_000 {
                d.step();
            }
            match found {
                Found::Red => {
                    let most = d.v().iter().fold(0f32, |m, &v| m.max(v));
                    assert!(most < 1e-3, "R at k {k}, F {f}: V up to {most}");
                }
                Found::Blue => {
                    let (u, v) = blue_state(rates).unwrap();
                    let off = d
                        .u()
                        .iter()
                        .zip(d.v())
                        .map(|(&du, &dv)| (f64::from(du) - u).abs().max((f64::from(dv) - v).abs()))
                        .fold(0f64, f64::max);
                    assert!(off < 0.01, "B at k {k}, F {f}: {off} from ({u:.3}, {v:.3})");
                }
                Found::Pattern(_) => unreachable!(),
            }
        }
    }

    /// λ, at k = 0.065, F = 0.04: "blue spots on a red or yellow background", which
    /// "increase in number until they fill the system", and then the pattern is "time
    /// independent" (Pearson p. 5). After his 200,000 steps: many separate spots, each small,
    /// and hardly changing.
    #[test]
    #[ignore]
    fn lambda_fills_the_dish_with_spots_that_stand_still() {
        let d = pearson_run(CLASSES[10].rates, 200_000);
        let spots = patches(&d, 0.2);
        let biggest = spots.iter().max().copied().unwrap_or(0);
        assert!(spots.len() > 100, "{} spots", spots.len());
        assert!(biggest < 100, "a patch of {biggest} cells is no spot");
        assert!(d.change().unwrap() < 1e-5, "change {:?}", d.change());
    }

    /// μ, at k = 0.065, F = 0.05: "long stripes grew in length" (p. 4). After 200,000 steps
    /// the blue patches are few and long, not spots.
    #[test]
    #[ignore]
    fn mu_grows_stripes_not_spots() {
        let d = pearson_run(CLASSES[11].rates, 200_000);
        let stripes = patches(&d, 0.2);
        let biggest = stripes.iter().max().copied().unwrap_or(0);
        assert!(biggest > 1_000, "the biggest patch is only {biggest} cells");
        assert!(stripes.len() < 60, "{} patches", stripes.len());
    }

    /// ε, at k = 0.059, F = 0.02: spots that divide and die "with no long-range order either
    /// in time or space" (p. 5); Pearson measured a positive Liapunov exponent there. After
    /// 200,000 steps it is still spots, and still changing.
    #[test]
    #[ignore]
    fn epsilon_never_settles() {
        let d = pearson_run(CLASSES[4].rates, 200_000);
        assert!(patches(&d, 0.2).len() > 20);
        assert!(d.change().unwrap() > 1e-4, "change {:?}", d.change());
    }
}
