//! The flow: the gas's density and velocity, averaged over blocks of nodes and over steps.
//!
//! One node's particles say almost nothing about the flow: 0 to 6 particles, in a few
//! directions. The fluid appears only in averages. d'Humières and Lallemand (Complex Systems
//! 1 (1987) 599–632, §3.3) obtained "macroscopic quantities ... by averaging ... over
//! rectangular regions", and over time steps; this does both.

use crate::sim::{Kind, Lattice, PARTICLES, WALL};

/// Per node byte, four counts packed into 16-bit lanes of a `u64`, so summing a block is one
/// lookup and one addition per node: its particles, its momentum's x and y in
/// [`Kind::momentum`]'s integer units (each offset by 4 to keep it positive), and 1 if it is
/// open. One table per lattice.
/// Walls count as empty: the particles they hold for a step are being turned round, not
/// flowing. A block of up to 8,000 nodes fits: no lane passes 8 per node.
static PACKED: [u64; 256] = packed_table(Kind::Hex.momentum());
static PACKED_SQUARE: [u64; 256] = packed_table(Kind::Square.momentum());

const OFFSET: i64 = 4;

const fn packed_table(momentum: &[(i32, i32); 6]) -> [u64; 256] {
    let mut table = [0; 256];
    let mut s = 0;
    while s < 256 {
        let (mut n, mut jx, mut jy, mut open) = (0, OFFSET, OFFSET, 0);
        if s as u8 & WALL == 0 {
            open = 1;
            let mut i = 0;
            while i < 6 {
                if (s as u8 & PARTICLES) >> i & 1 == 1 {
                    n += 1;
                    jx += momentum[i].0 as i64;
                    jy += momentum[i].1 as i64;
                }
                i += 1;
            }
        }
        table[s] = n as u64 | (jx as u64) << 16 | (jy as u64) << 32 | (open as u64) << 48;
        s += 1;
    }
    table
}

/// Sums of particles and momentum over blocks of `bw` x `bh` nodes, accumulated over any
/// number of steps.
pub struct Blocks {
    kind: Kind,
    pub bw: usize,
    pub bh: usize,
    /// Blocks across and down. Nodes beyond the last whole block are left out. A block may
    /// hold at most 8,000 nodes (see [`PACKED`]).
    pub nx: usize,
    pub ny: usize,
    mass: Vec<u32>,
    jx: Vec<i32>,
    jy: Vec<i32>,
    /// Open (non-wall) nodes in each block, as of the last sample.
    open: Vec<u32>,
    /// Steps accumulated since the last [`Blocks::clear`].
    pub samples: u32,
}

/// One block's averages: particles per open node, and the mean velocity in node spacings
/// per step, x right and y up. A block with no particles has velocity zero.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Average {
    pub density: f64,
    pub ux: f64,
    pub uy: f64,
}

impl Blocks {
    pub fn new(kind: Kind, width: usize, height: usize, bw: usize, bh: usize) -> Self {
        assert!(
            bw * bh <= 8_000,
            "a block's sums would overflow their lanes"
        );
        let (nx, ny) = (width / bw, height / bh);
        Self {
            kind,
            bw,
            bh,
            nx,
            ny,
            mass: vec![0; nx * ny],
            jx: vec![0; nx * ny],
            jy: vec![0; nx * ny],
            open: vec![0; nx * ny],
            samples: 0,
        }
    }

    pub fn clear(&mut self) {
        self.mass.fill(0);
        self.jx.fill(0);
        self.jy.fill(0);
        self.samples = 0;
    }

    /// Add one step's particles.
    pub fn add(&mut self, lattice: &Lattice) {
        let (w, cells) = (lattice.width(), lattice.cells());
        let table = match self.kind {
            Kind::Hex => &PACKED,
            Kind::Square => &PACKED_SQUARE,
        };
        let nodes = (self.bw * self.bh) as i64;
        for by in 0..self.ny {
            for bx in 0..self.nx {
                let mut sum = 0u64;
                for y in by * self.bh..(by + 1) * self.bh {
                    for &c in &cells[y * w + bx * self.bw..][..self.bw] {
                        sum += table[c as usize];
                    }
                }
                let lane = |i: u32| (sum >> (16 * i) & 0xffff) as i64;
                let k = by * self.nx + bx;
                self.mass[k] += lane(0) as u32;
                self.jx[k] += (lane(1) - OFFSET * nodes) as i32;
                self.jy[k] += (lane(2) - OFFSET * nodes) as i32;
                self.open[k] = lane(3) as u32;
            }
        }
        self.samples += 1;
    }

    /// Block `(bx, by)`'s averages over the samples so far.
    pub fn average(&self, bx: usize, by: usize) -> Average {
        let k = by * self.nx + bx;
        let mass = f64::from(self.mass[k]);
        let open = f64::from(self.open[k]) * f64::from(self.samples);
        if mass == 0.0 || open == 0.0 {
            return Average::default();
        }
        let (ux, uy) = self.kind.momentum_unit();
        Average {
            density: mass / open,
            ux: f64::from(self.jx[k]) * ux / mass,
            uy: f64::from(self.jy[k]) * uy / mass,
        }
    }

    /// The centre of block `(bx, by)`: the mean of its nodes' positions (as
    /// `Lattice::position` gives them, y down).
    pub fn centre(&self, bx: usize, by: usize) -> (f64, f64) {
        block_centre(self.kind, self.bw, self.bh, bx, by)
    }
}

/// The mean position of the nodes of block `(bx, by)`, `bw` x `bh` nodes. On the hex
/// lattice half its rows are odd, shifted half a node.
fn block_centre(kind: Kind, bw: usize, bh: usize, bx: usize, by: usize) -> (f64, f64) {
    let (bw, bh) = (bw as f64, bh as f64);
    let shift = if kind.shifted() { 0.25 } else { 0.0 };
    (
        bx as f64 * bw + (bw - 1.0) / 2.0 + shift,
        (by as f64 * bh + (bh - 1.0) / 2.0) * kind.row_height(),
    )
}

/// The flow as drawn: each block's averages, smoothed over time, with the vorticity and
/// values between block centres worked out from them.
pub struct Field {
    kind: Kind,
    pub nx: usize,
    pub ny: usize,
    bw: usize,
    bh: usize,
    pub cells: Vec<Average>,
    /// Set until the first [`Field::blend`], which then takes the averages as they are.
    fresh: bool,
}

impl Field {
    pub fn new(blocks: &Blocks) -> Self {
        Self {
            kind: blocks.kind,
            nx: blocks.nx,
            ny: blocks.ny,
            bw: blocks.bw,
            bh: blocks.bh,
            cells: vec![Average::default(); blocks.nx * blocks.ny],
            fresh: true,
        }
    }

    /// Fold in the averages accumulated in `blocks`, as a moving average over about
    /// `window` steps: the more steps `blocks` holds, the more weight they get.
    pub fn blend(&mut self, blocks: &Blocks, window: f64) {
        if blocks.samples == 0 {
            return;
        }
        let a = if self.fresh {
            1.0
        } else {
            1.0 - (-f64::from(blocks.samples) / window.max(1.0)).exp()
        };
        self.fresh = false;
        for by in 0..self.ny {
            for bx in 0..self.nx {
                let new = blocks.average(bx, by);
                let old = &mut self.cells[by * self.nx + bx];
                old.density += a * (new.density - old.density);
                old.ux += a * (new.ux - old.ux);
                old.uy += a * (new.uy - old.uy);
            }
        }
    }

    pub fn get(&self, bx: usize, by: usize) -> Average {
        self.cells[by * self.nx + bx]
    }

    /// The spacing of block centres, across and down, in node spacings.
    fn spacing(&self) -> (f64, f64) {
        (self.bw as f64, self.bh as f64 * self.kind.row_height())
    }

    /// The vorticity at block `(bx, by)`: how fast the flow there spins, counter-clockwise
    /// positive (`d uy/dx - d ux/dy`, y up), in radians per step times 2. Central
    /// differences, one-sided at the edges.
    pub fn vorticity(&self, bx: usize, by: usize) -> f64 {
        let (dx, dy) = self.spacing();
        let (x0, x1) = (bx.saturating_sub(1), (bx + 1).min(self.nx - 1));
        let (y0, y1) = (by.saturating_sub(1), (by + 1).min(self.ny - 1));
        let duy_dx = if x1 > x0 {
            (self.get(x1, by).uy - self.get(x0, by).uy) / ((x1 - x0) as f64 * dx)
        } else {
            0.0
        };
        // Block rows run down the screen, so y up decreases as `by` grows.
        let dux_dy = if y1 > y0 {
            (self.get(bx, y0).ux - self.get(bx, y1).ux) / ((y1 - y0) as f64 * dy)
        } else {
            0.0
        };
        duy_dx - dux_dy
    }

    /// The field at a lattice position `(px, py)` (as `Lattice::position`, y down),
    /// interpolated between the four nearest block centres, and constant beyond the outer
    /// centres. `value` picks what to interpolate from each block.
    pub fn at(&self, px: f64, py: f64, value: impl Fn(usize, usize) -> f64) -> f64 {
        let (dx, dy) = self.spacing();
        // Block (0, 0)'s centre (see `Blocks::centre`).
        let (cx, cy) = block_centre(self.kind, self.bw, self.bh, 0, 0);
        let fx = ((px - cx) / dx).clamp(0.0, (self.nx - 1) as f64);
        let fy = ((py - cy) / dy).clamp(0.0, (self.ny - 1) as f64);
        let (x0, y0) = (fx as usize, fy as usize);
        let (x1, y1) = ((x0 + 1).min(self.nx - 1), (y0 + 1).min(self.ny - 1));
        let (tx, ty) = (fx - x0 as f64, fy - y0 as f64);
        let top = value(x0, y0) * (1.0 - tx) + value(x1, y0) * tx;
        let bottom = value(x0, y1) * (1.0 - tx) + value(x1, y1) * tx;
        top * (1.0 - ty) + bottom * ty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{E, Inflow, Kind, ROW_HEIGHT, SOUND_SPEED};

    /// A gas with exactly one particle moving east at every node has density 1 and velocity
    /// exactly (1, 0) in every block; walls don't count towards either.
    #[test]
    fn a_uniform_stream_averages_exactly() {
        let mut l = Lattice::new(Kind::Hex, 24, 12, 1);
        for y in 0..12 {
            for x in 0..24 {
                l.set_particles(x, y, E);
            }
        }
        l.set_wall(0, 0, true);
        let mut b = Blocks::new(Kind::Hex, 24, 12, 8, 6);
        b.add(&l);
        b.add(&l);
        assert_eq!((b.nx, b.ny, b.samples), (3, 2, 2));
        for by in 0..2 {
            for bx in 0..3 {
                let a = b.average(bx, by);
                assert_eq!(
                    a,
                    Average {
                        density: 1.0,
                        ux: 1.0,
                        uy: 0.0
                    }
                );
            }
        }
    }

    /// A gas filled at density d moving at speed u averages to about d per direction and u
    /// (d'Humières and Lallemand's equilibrium (2.5)).
    #[test]
    fn an_equilibrium_gas_averages_to_its_density_and_speed() {
        let mut l = Lattice::new(Kind::Hex, 96, 96, 7);
        l.fill(0.2, 0.25);
        let mut b = Blocks::new(Kind::Hex, 96, 96, 48, 48);
        b.add(&l);
        for by in 0..2 {
            for bx in 0..2 {
                let a = b.average(bx, by);
                assert!((a.density / 6.0 - 0.2).abs() < 0.01, "{a:?}");
                assert!((a.ux - 0.25).abs() < 0.03 && a.uy.abs() < 0.03, "{a:?}");
            }
        }
    }

    fn field_of(
        nx: usize,
        ny: usize,
        bw: usize,
        bh: usize,
        f: impl Fn(f64, f64) -> Average,
    ) -> Field {
        let blocks = Blocks::new(Kind::Hex, nx * bw, ny * bh, bw, bh);
        let mut field = Field::new(&blocks);
        for by in 0..ny {
            for bx in 0..nx {
                let (x, y) = blocks.centre(bx, by);
                field.cells[by * nx + bx] = f(x, y);
            }
        }
        field
    }

    /// A flow turning as a solid body at rate W, (ux, uy) = (-W y, W x) with y up, has
    /// vorticity 2W everywhere: a check of the differences' signs and spacings.
    #[test]
    fn solid_rotation_has_vorticity_twice_its_rate() {
        let rate = 0.001;
        let f = field_of(10, 8, 12, 14, |x, y| Average {
            density: 1.0,
            ux: -rate * -y, // y up is minus y down
            uy: rate * x,
        });
        for by in 0..8 {
            for bx in 0..10 {
                assert!(
                    (f.vorticity(bx, by) - 2.0 * rate).abs() < 1e-12,
                    "({bx}, {by})"
                );
            }
        }
    }

    /// Interpolation returns each block's value at its centre, and a linear field
    /// exactly in between.
    #[test]
    fn interpolation_hits_block_centres_and_is_linear_between() {
        let f = field_of(6, 5, 8, 10, |x, y| Average {
            density: 0.5 * x - 0.25 * y,
            ux: 0.0,
            uy: 0.0,
        });
        let blocks = Blocks::new(Kind::Hex, 48, 50, 8, 10);
        let density = |bx: usize, by: usize| f.get(bx, by).density;
        for (bx, by) in [(0, 0), (3, 2), (5, 4)] {
            let (x, y) = blocks.centre(bx, by);
            assert!((f.at(x, y, density) - f.get(bx, by).density).abs() < 1e-12);
        }
        let (x, y) = (21.3, 17.9);
        assert!((f.at(x, y, density) - (0.5 * x - 0.25 * y)).abs() < 1e-9);
    }

    /// The mean particles per node in rings around `(cx, cy)`, `width` node spacings wide,
    /// out to `rings` rings, counting only nodes whose direction from the centre (in
    /// degrees) satisfies `angle`.
    fn sector_profile(
        l: &Lattice,
        (cx, cy): (f64, f64),
        width: f64,
        rings: usize,
        angle: impl Fn(f64) -> bool,
    ) -> Vec<f64> {
        let (mut sum, mut count) = (vec![0.0f64; rings], vec![0.0f64; rings]);
        for y in 0..l.height() {
            for x in 0..l.width() {
                let (px, py) = Kind::Hex.position(x, y);
                let k = ((px - cx).hypot(py - cy) / width) as usize;
                if k < rings && angle((cy - py).atan2(px - cx).to_degrees()) {
                    sum[k] += f64::from((l.get(x, y) & PARTICLES).count_ones());
                    count[k] += 1.0;
                }
            }
        }
        sum.iter()
            .zip(&count)
            .map(|(s, c)| s / c.max(1.0))
            .collect()
    }

    /// The radius of a ring of extra density: the centroid of the excess over
    /// `background` within 6 rings of the densest ring.
    fn ring_radius(profile: &[f64], width: f64, background: f64) -> f64 {
        let peak = (0..profile.len())
            .max_by(|&a, &b| profile[a].total_cmp(&profile[b]))
            .unwrap();
        let near = peak.saturating_sub(6)..(peak + 7).min(profile.len());
        let (mut moment, mut mass) = (0.0, 0.0);
        for k in near {
            let excess = (profile[k] - background).max(0.0);
            moment += excess * (k as f64 + 0.5) * width;
            mass += excess;
        }
        moment / mass
    }

    /// A pulse of dense gas spreads as a ring, and on this lattice the ring is round: as
    /// far out along the lattice's directions as between them (each within 30° sectors
    /// repeating every 60°). Averaged over 16 runs. (The ring's centroid runs about 3% faster
    /// than the speed of sound: a two-dimensional pulse leaves a trailing wake, so the
    /// centroid is no measure of the speed; `sound_travels_at_one_over_root_two` measures
    /// it as the papers did.)
    #[test]
    #[ignore]
    fn a_pulse_spreads_as_a_round_ring() {
        let (w, h) = (600, 600);
        let centre = (300.0, 300.0 * ROW_HEIGHT);
        let (width, rings, runs) = (4.0, 60, 16);
        let along = |a: f64| (a + 15.0).rem_euclid(60.0) < 30.0;
        let between = |a: f64| !along(a);
        // Profiles at steps 100 and 200, all round; at step 200, along and between.
        let mut profiles = [
            vec![0.0; rings],
            vec![0.0; rings],
            vec![0.0; rings],
            vec![0.0; rings],
        ];
        for seed in 0..runs {
            let mut l = Lattice::new(Kind::Hex, w, h, seed);
            l.fill(0.2, 0.0);
            l.fill_disc(centre.0, centre.1, 15.0, 0.35);
            while l.steps < 100 {
                l.step();
            }
            let mut add = |k: usize, p: Vec<f64>| {
                for (a, b) in profiles[k].iter_mut().zip(p) {
                    *a += b / runs as f64;
                }
            };
            add(0, sector_profile(&l, centre, width, rings, |_| true));
            while l.steps < 200 {
                l.step();
            }
            add(1, sector_profile(&l, centre, width, rings, |_| true));
            add(2, sector_profile(&l, centre, width, rings, along));
            add(3, sector_profile(&l, centre, width, rings, between));
        }
        let r = profiles.map(|p| ring_radius(&p, width, 1.2));
        println!("ring radii {r:?}");
        assert!(r[1] - r[0] > 60.0, "the ring moves out: {r:?}");
        assert!((r[2] - r[3]).abs() < 0.02 * r[1], "radii {r:?}");
    }

    /// A standing sound wave: a density ripple `cos(k . r)` over a gas at rest, one
    /// wavelength across a world `nodes` nodes wide (along x) or `nodes` rows high (down y,
    /// so `nodes` must be even). Returns the wavelength and the mean time between its
    /// amplitude's zero crossings, in steps.
    fn sound_half_period(along_x: bool, nodes: usize, seed: u64) -> (f64, f64) {
        let (w, h, wavelength) = if along_x {
            (nodes, 40, nodes as f64)
        } else {
            (40, nodes, nodes as f64 * ROW_HEIGHT)
        };
        let mut l = Lattice::new(Kind::Hex, w, h, seed);
        let k = 2.0 * std::f64::consts::PI / wavelength;
        let phase = |x: usize, y: usize| {
            let (px, py) = Kind::Hex.position(x, y);
            k * if along_x { px } else { py }
        };
        let mut rng = ca_core::Rng::new(seed);
        for y in 0..h {
            for x in 0..w {
                let d = 0.2 * (1.0 + 0.3 * phase(x, y).cos());
                let bits = (0..6).filter(|_| rng.chance(d)).fold(0, |b, i| b | 1 << i);
                l.set_particles(x, y, bits);
            }
        }
        let amplitude = |l: &Lattice| {
            let mut a = 0.0;
            for y in 0..h {
                for x in 0..w {
                    a += f64::from((l.get(x, y) & PARTICLES).count_ones()) * phase(x, y).cos();
                }
            }
            a
        };
        let mut crossings = Vec::new();
        let mut last = amplitude(&l);
        for t in 1..=420 {
            l.step();
            let a = amplitude(&l);
            if (a < 0.0) != (last < 0.0) {
                crossings.push(t as f64 - 1.0 + last / (last - a));
            }
            last = a;
        }
        let half = (crossings[crossings.len() - 1] - crossings[0]) / (crossings.len() - 1) as f64;
        (wavelength, half)
    }

    /// Sound travels at 1/sqrt 2 node spacings per step in FHP-I (Frisch et al. 1987, §7),
    /// the same in every direction: d'Humières and Lallemand (§5) measured it so, from
    /// standing waves, finding the speeds "isotropic" and in agreement with 1/sqrt 2. A
    /// standing wave of wavelength L oscillates with period L / c, so its density crosses
    /// zero every L / 2c steps. Checked along a lattice direction and at 90° to it (between
    /// two), with wavelengths of about 120 nodes, within 2%.
    #[test]
    fn sound_travels_at_one_over_root_two() {
        for along_x in [true, false] {
            let runs: Vec<(f64, f64)> =
                (0..3).map(|s| sound_half_period(along_x, 120, s)).collect();
            let half = runs.iter().map(|r| r.1).sum::<f64>() / 3.0;
            let speed = runs[0].0 / (2.0 * half);
            println!("along x {along_x}: speed {speed}");
            assert!(
                (speed - SOUND_SPEED).abs() < 0.02 * SOUND_SPEED,
                "along x {along_x}: speed {speed}"
            );
        }
    }

    /// The step's cost, for sizing the work done per frame (`--ignored --nocapture`).
    #[test]
    #[ignore]
    fn timing_steps() {
        let (w, h) = (800, 400);
        let mut l = Lattice::new(Kind::Hex, w, h, 1);
        l.fill(0.2, 0.3);
        l.inflow = Some(Inflow {
            width: 4,
            density: 0.2,
            speed: 0.3,
        });
        let mut b = Blocks::new(Kind::Hex, w, h, 16, 18);
        let steps = 500;
        let t = std::time::Instant::now();
        for _ in 0..steps {
            l.step();
        }
        let step_ns = t.elapsed().as_nanos() as f64 / (steps * w * h) as f64;
        let t = std::time::Instant::now();
        for _ in 0..steps {
            b.add(&l);
        }
        let add_ns = t.elapsed().as_nanos() as f64 / (steps * w * h) as f64;
        println!("step {step_ns:.2} ns/node, averaging {add_ns:.2} ns/node");
    }

    /// The flow behind a flat plate across a wind turns back on itself: two eddies form
    /// behind it, the "wake" d'Humières and Lallemand measured (§6.3, figure 8). On the
    /// centre line just behind the plate, the averaged velocity points upstream.
    ///
    /// The plate is 115 node spacings across, the wind 0.3 at density 0.2, so the Reynolds
    /// number u w g / nu is about 19 (FHP-I's g and nu from Frisch et al. 1987, Table 1).
    #[test]
    #[ignore]
    fn a_plate_has_a_recirculating_wake() {
        let (w, h) = (800, 400);
        let mut l = Lattice::new(Kind::Hex, w, h, 3);
        let mid = h as f64 * ROW_HEIGHT / 2.0;
        let half = h as f64 * ROW_HEIGHT / 6.0;
        l.wall_where(|px, py| (px - 200.0).abs() <= 1.0 && (py - mid).abs() <= half);
        l.fill(0.2, 0.3);
        l.inflow = Some(Inflow {
            width: 4,
            density: 0.2,
            speed: 0.3,
        });
        let (bw, bh) = (16, 18);
        let mut b = Blocks::new(Kind::Hex, w, h, bw, bh);
        for _ in 0..1500 {
            l.step();
        }
        for _ in 0..500 {
            l.step();
            b.add(&l);
        }
        // The plate is in block column 200 / 16 = 12. Measured (seed 3): about -0.07 at
        // its lowest, 3 to 5 blocks behind the plate; the wind ahead of it stays near 0.26.
        let row = b.ny / 2;
        let behind = (13..25)
            .map(|bx| b.average(bx, row).ux)
            .fold(f64::MAX, f64::min);
        let ahead = b.average(4, row).ux;
        assert!(behind < -0.03, "lowest u_x behind the plate: {behind}");
        assert!(ahead > 0.2, "u_x ahead of the plate: {ahead}");
    }

    /// The mean u_x on the centre line, `behind` nodes downstream of a plate, over steps
    /// 2,500 to 3,000 of a wind of 0.25 past it.
    fn centre_line_flow_behind_a_plate(kind: Kind, behind: f64) -> f64 {
        let (w, h) = (800, 400);
        let mut l = Lattice::new(kind, w, h, 3);
        let tall = h as f64 * kind.row_height();
        l.wall_where(|px, py| (px - 200.0).abs() <= 1.0 && (py - tall / 2.0).abs() <= tall / 6.0);
        l.fill(0.2, 0.25);
        l.inflow = Some(Inflow {
            width: 4,
            density: 0.2,
            speed: 0.25,
        });
        let bw = 16;
        let bh = if kind == Kind::Hex { 18 } else { 16 };
        let mut b = Blocks::new(kind, w, h, bw, bh);
        for t in 0..3000 {
            l.step();
            if t >= 2500 {
                b.add(&l);
            }
        }
        b.average(((200.0 + behind) / bw as f64) as usize, b.ny / 2)
            .ux
    }

    /// The wake shows HPP's flaw. Each row of HPP keeps its own momentum (see
    /// `hpp_keeps_each_rows_momentum_and_fhp_does_not`), so rows stopped by the plate are
    /// never set moving again by their neighbours: 400 nodes behind it, the centre line is
    /// still almost still. On FHP's lattice momentum spreads between rows, as viscosity
    /// does in a real fluid, and the wake fills in. (Measured: about 0.2 for FHP, under
    /// 0.05 for HPP, against a wind of 0.25.)
    #[test]
    #[ignore]
    fn hpp_wake_never_fills_in() {
        let fhp = centre_line_flow_behind_a_plate(Kind::Hex, 400.0);
        let hpp = centre_line_flow_behind_a_plate(Kind::Square, 400.0);
        println!("u_x 400 behind the plate: FHP {fhp:.3}, HPP {hpp:.3}");
        assert!(fhp > 0.15, "FHP {fhp}");
        assert!(hpp < 0.08, "HPP {hpp}");
    }
}
