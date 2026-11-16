//! The FHP lattice gas (Frisch, Hasslacher and Pomeau, 1986): particles on a triangular
//! lattice whose averages flow like a fluid.
//!
//! Each node of the lattice has six neighbours, one in each direction `i * 60°`. A node
//! holds at most one particle moving in each direction, so its state is six bits. Every step
//! has two phases:
//!
//! 1. **Collision.** At each node, particles meeting there may scatter, by rules that keep
//!    the node's particle count and total momentum. This is model FHP-I, as written out in
//!    Frisch, d'Humières, Hasslacher, Lallemand, Pomeau and Rivet, "Lattice gas
//!    hydrodynamics in two and three dimensions", Complex Systems 1 (1987) 649–707,
//!    equation (3.4) and section 2.2:
//!    - **head-on pairs**: exactly two particles, moving in opposite directions `(i, i+3)`,
//!      leave turned by 60° one way or the other, `(i-1, i+2)` or `(i+1, i+4)`, with equal
//!      probability;
//!    - **triples**: exactly three particles at 120° to each other, `(i, i+2, i+4)`, turn
//!      into the other triple, `(i+1, i+3, i+5)`;
//!    - every other state is left alone.
//!
//!    d'Humières and Lallemand (Complex Systems 1 (1987) 599–632, p. 602) count these as
//!    "five configurations among the 64 possible ones", "the minimal set of rules to
//!    prevent spurious conservation laws".
//! 2. **Streaming.** Every particle hops to the neighbouring node in its direction.
//!
//! Walls are nodes that send every arriving particle straight back (bounce-back), which
//! makes the fluid stick to them: the "no-slip" condition (d'Humières and Lallemand §3.1).
//!
//! **The lattice.** The papers' lattice is triangular; the nodes are laid out in rows, each
//! odd row shifted half a node right of the even rows, so the world is a rectangle (the
//! layout d'Humières and Lallemand used, §3.1 and their figure 1, since a lattice in
//! axial coordinates is a rhombus, "awkward for most situations"). The two kinds of row
//! then have different neighbour offsets above and below. Row 0 is at the top; the world
//! wraps around both ways, which needs an even number of rows.
//!
//! **Speed.** A node's six particle bits, a wall bit and a random bit fit in one byte. A
//! collision is one lookup in a 256-entry table, and streaming gathers each row's new bits
//! from the old rows at fixed offsets, a loop the compiler vectorises.
//!
//! **HPP.** For contrast, the same machinery runs the earlier model of Hardy, Pomeau and de
//! Pazzis (1973) on a square lattice ([`Kind::Square`]): four directions, and one collision,
//! a head-on pair turning through 90° (Frisch et al. 1987, §2.1 and equation (3.3)). Its
//! flaw is visible: it conserves the momentum along every row and column separately, "far
//! too many conserved quantities for physical modeling" (ibid.), so momentum never spreads
//! from one row to the next.

use ca_core::Rng;

/// Bits 0 to 5 of a node: a particle moving in direction `i`, at angle `i * 60°`
/// counter-clockwise from east (y up, as in the papers): east, north-east, north-west,
/// west, south-west, south-east. On the square lattice, bits 0 to 3 are directions
/// `i * 90°`: east, north, west, south.
pub const DIRECTIONS: usize = 6;

/// Which lattice, and so which model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// FHP-I on the triangular lattice, in offset rows.
    Hex,
    /// HPP on the square lattice.
    Square,
}

impl Kind {
    /// The number of directions a particle can move in.
    pub fn directions(self) -> usize {
        match self {
            Self::Hex => 6,
            Self::Square => 4,
        }
    }

    /// Direction `i`'s unit vector, `(cos, sin)`, y up.
    pub fn velocity(self, i: usize) -> (f64, f64) {
        let a = i as f64 * std::f64::consts::TAU / self.directions() as f64;
        (a.cos(), a.sin())
    }

    /// Each direction's momentum in exact integer units (see [`MOMENTUM`]); unused
    /// directions are zero.
    pub const fn momentum(self) -> &'static [(i32, i32); DIRECTIONS] {
        match self {
            Self::Hex => &MOMENTUM,
            Self::Square => &MOMENTUM_SQUARE,
        }
    }

    /// The size of [`Kind::momentum`]'s units, in node spacings per step, across and up.
    pub fn momentum_unit(self) -> (f64, f64) {
        match self {
            Self::Hex => (0.5, 3f64.sqrt() / 4.0),
            Self::Square => (1.0, 1.0),
        }
    }

    /// The spacing of rows, in node spacings.
    pub fn row_height(self) -> f64 {
        match self {
            Self::Hex => ROW_HEIGHT,
            Self::Square => 1.0,
        }
    }

    /// Whether odd rows sit half a node right of even rows.
    pub fn shifted(self) -> bool {
        self == Self::Hex
    }

    /// Where node `(x, y)` sits, in node spacings, x right and y down.
    pub fn position(self, x: usize, y: usize) -> (f64, f64) {
        let shift = if self.shifted() && y % 2 == 1 {
            0.5
        } else {
            0.0
        };
        (x as f64 + shift, y as f64 * self.row_height())
    }

    pub fn collisions(self) -> &'static [u8; 256] {
        match self {
            Self::Hex => &COLLIDE,
            Self::Square => &COLLIDE_SQUARE,
        }
    }
}

/// All six particle bits.
pub const PARTICLES: u8 = 0b0011_1111;

/// The node is a wall: arriving particles are sent back the way they came.
pub const WALL: u8 = 1 << 6;

/// The node's random bit, choosing which way a head-on pair turns. d'Humières and
/// Lallemand (§3.2) kept such a bit in each node's byte, "initially set at random with an
/// average density 1/2 and ... complemented each time a collision occurs", and checked
/// that it did not significantly bias their results. This does the same.
pub const CHIRAL: u8 = 1 << 7;

/// Direction `i`'s unit vector, scaled to integers as in d'Humières and Lallemand's
/// footnote 4: `(2 cx, 4 cy / sqrt 3)`. Momentum sums in these units are exact.
pub const MOMENTUM: [(i32, i32); DIRECTIONS] =
    [(2, 0), (1, 2), (-1, 2), (-2, 0), (-1, -2), (1, -2)];

/// The square lattice's four unit vectors, east, north, west and south.
pub const MOMENTUM_SQUARE: [(i32, i32); DIRECTIONS] =
    [(1, 0), (0, 1), (-1, 0), (0, -1), (0, 0), (0, 0)];

/// A row's vertical spacing, in node spacings: the rows of a triangular lattice are
/// `sqrt(3) / 2` apart.
pub const ROW_HEIGHT: f64 = 0.866_025_403_784_438_6;

/// FHP-I's speed of sound, in node spacings per step: `1 / sqrt 2` (Frisch et al. 1987,
/// §7, after equation (7.6)).
pub const SOUND_SPEED: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// FHP-I's factor `g` on the flow's self-advection, at density `d` per direction: the
/// lattice gas follows the Navier-Stokes equations with time rescaled by it (Frisch et al.
/// 1987, Table 1: `g = (1/2)(1 - 2d)/(1 - d)`).
pub fn advection_factor(d: f64) -> f64 {
    0.5 * (1.0 - 2.0 * d) / (1.0 - d)
}

/// FHP-I's kinematic shear viscosity at density `d`, in node spacings squared per step
/// (Frisch et al. 1987, Table 1, in the lattice Boltzmann approximation:
/// `1 / (12 d (1 - d)^3) - 1/8`).
pub fn viscosity(d: f64) -> f64 {
    1.0 / (12.0 * d * (1.0 - d).powi(3)) - 0.125
}

/// The Reynolds number of a flow at speed `u` past an obstacle `l` node spacings across,
/// at density `d`: `u l g / nu` (Frisch et al. 1987, equation (8.27)).
///
/// Written as Mach number times `l` times `R* = c_s g / nu` (their (8.29)–(8.30)), FHP-I's
/// `R*` is at most 0.387, at d = 0.187 (their (8.32) and Table 1; FHP-III reaches 2.22, at
/// d = 0.285). The wind must stay well below the speed of sound to keep the flow nearly
/// incompressible (§8), so at Mach 0.5 an obstacle 100 nodes wide reaches only about 20:
/// the README's "the Reynolds number stays in the tens".
pub fn reynolds(u: f64, l: f64, d: f64) -> f64 {
    u * l * advection_factor(d) / viscosity(d)
}

/// The collision applied to every possible node byte, built once at compile time.
pub static COLLIDE: [u8; 256] = collision_table();

const fn rotate(bits: u8, by: u32) -> u8 {
    // A six-bit rotation: direction i becomes i + by.
    let by = by % 6;
    ((bits << by) | (bits >> (6 - by))) & PARTICLES
}

/// The FHP-I collision of one node (see the module comment), with bounce-back at walls.
/// A collision that changes the particles flips the node's random bit.
const fn collide(state: u8) -> u8 {
    let n = state & PARTICLES;
    let keep = state & (WALL | CHIRAL);
    if state & WALL != 0 {
        return keep | rotate(n, 3);
    }
    let pair = 0b00_1001; // directions 0 and 3
    let triple = 0b01_0101; // directions 0, 2 and 4
    let out = if n == triple || n == rotate(triple, 1) {
        rotate(n, 1)
    } else if n == pair || n == rotate(pair, 1) || n == rotate(pair, 2) {
        // Equation (3.4): with the random bit set, the pair (i+1, i+4) produces a particle
        // at i, so a pair turns by -60°; with it clear, by +60°.
        if state & CHIRAL != 0 {
            rotate(n, 5)
        } else {
            rotate(n, 1)
        }
    } else {
        n
    };
    if out != n {
        (keep ^ CHIRAL) | out
    } else {
        keep | out
    }
}

/// HPP's collision of one node (Frisch et al. 1987, §2.1): exactly two particles meeting
/// head-on, east-west or north-south, leave along the other axis; walls bounce back.
const fn collide_square(state: u8) -> u8 {
    let n = state & 0b1111;
    let keep = state & (WALL | CHIRAL);
    let turned = ((n << 1) | (n >> 3)) & 0b1111; // direction i becomes i + 1
    if state & WALL != 0 {
        keep | (((n << 2) | (n >> 2)) & 0b1111)
    } else if n == 0b0101 || n == 0b1010 {
        keep | turned
    } else {
        keep | n
    }
}

pub static COLLIDE_SQUARE: [u8; 256] = {
    let mut table = [0; 256];
    let mut s = 0;
    while s < 256 {
        table[s] = collide_square(s as u8);
        s += 1;
    }
    table
};

const fn collision_table() -> [u8; 256] {
    let mut table = [0; 256];
    let mut s = 0;
    while s < 256 {
        table[s] = collide(s as u8);
        s += 1;
    }
    table
}

/// Particles in each node byte (walls count theirs too: they hold them for one step).
const fn count_table() -> [u8; 256] {
    let mut table = [0; 256];
    let mut s = 0;
    while s < 256 {
        table[s] = (s as u8 & PARTICLES).count_ones() as u8;
        s += 1;
    }
    table
}

pub static COUNT: [u8; 256] = count_table();

/// A gas flowing in at the left edge: each step, the leftmost `width` columns are refilled
/// with fresh particles, at `density` particles per direction and moving east at `speed`.
///
/// The world wraps around, so the same band also swallows whatever flows out of the right
/// edge. It is d'Humières and Lallemand's "wind tunnel" (§3.1), with the inlet and outlet
/// in one place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inflow {
    pub width: usize,
    pub density: f64,
    pub speed: f64,
}

/// The lattice: a `width` x `height` rectangle of nodes, wrapping around both ways.
pub struct Lattice {
    kind: Kind,
    width: usize,
    height: usize,
    cells: Vec<u8>,
    next: Vec<u8>,
    pub inflow: Option<Inflow>,
    pub steps: u64,
    rng: Rng,
}

/// The chance of a particle in direction `i` in a gas at density `density` (per direction)
/// moving east at `speed`: d'Humières and Lallemand's equation (2.5), the equilibrium to
/// first order in the velocity, `d (1 + 2 c_i . u)` for FHP-I. The same form gives the
/// square lattice's gas mean velocity `u` too (see the tests).
pub fn equilibrium(kind: Kind, density: f64, speed: f64, i: usize) -> f64 {
    if i >= kind.directions() {
        return 0.0;
    }
    (density * (1.0 + 2.0 * kind.velocity(i).0 * speed)).clamp(0.0, 1.0)
}

impl Lattice {
    /// An empty lattice. On the hex lattice `height` must be even, so that rows still
    /// alternate when the world wraps from the bottom row to the top.
    pub fn new(kind: Kind, width: usize, height: usize, seed: u64) -> Self {
        assert!(
            width >= 3 && height >= 2,
            "a world needs at least 3 x 2 nodes"
        );
        assert!(
            kind == Kind::Square || height.is_multiple_of(2),
            "a wrapping hex world needs an even number of rows"
        );
        let mut rng = Rng::new(seed);
        let cells = (0..width * height)
            .map(|_| if rng.chance(0.5) { CHIRAL } else { 0 })
            .collect();
        Self {
            kind,
            width,
            height,
            cells,
            next: vec![0; width * height],
            inflow: None,
            steps: 0,
            rng,
        }
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> &[u8] {
        &self.cells
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.cells[y * self.width + x]
    }

    /// Set node `(x, y)`'s particles to `particles`, keeping its wall and random bits.
    pub fn set_particles(&mut self, x: usize, y: usize, particles: u8) {
        let c = &mut self.cells[y * self.width + x];
        *c = (*c & !PARTICLES) | (particles & PARTICLES);
    }

    /// Make node `(x, y)` a wall (removing its particles) or open it again.
    pub fn set_wall(&mut self, x: usize, y: usize, wall: bool) {
        let c = &mut self.cells[y * self.width + x];
        *c = if wall {
            (*c & CHIRAL) | WALL
        } else {
            *c & CHIRAL
        };
    }

    /// Make every node whose position (see [`Lattice::position`]) satisfies `inside` a wall.
    pub fn wall_where(&mut self, inside: impl Fn(f64, f64) -> bool) {
        for y in 0..self.height {
            for x in 0..self.width {
                let (px, py) = self.kind.position(x, y);
                if inside(px, py) {
                    self.set_wall(x, y, true);
                }
            }
        }
    }

    pub fn is_wall(&self, x: usize, y: usize) -> bool {
        self.get(x, y) & WALL != 0
    }

    /// Where node `(x, y)` sits, in node spacings, x right and y down: on the hex lattice,
    /// odd rows are shifted half a node right.
    pub fn position(&self, x: usize, y: usize) -> (f64, f64) {
        self.kind.position(x, y)
    }

    /// The neighbour of `(x, y)` in direction `i`, wrapping around the edges: the plain
    /// statement of what streaming does, which the tests check it against.
    #[cfg(test)]
    pub fn neighbour(&self, x: usize, y: usize, i: usize) -> (usize, usize) {
        let (w, h) = (self.width, self.height);
        if self.kind == Kind::Square {
            return match i % 4 {
                0 => ((x + 1) % w, y),
                1 => (x, (y + h - 1) % h),
                2 => ((x + w - 1) % w, y),
                _ => (x, (y + 1) % h),
            };
        }
        let p = y % 2; // odd rows sit half a node right, so their diagonals lean right
        let (left, right) = ((x + w - 1) % w, (x + 1) % w);
        let (up, down) = ((y + h - 1) % h, (y + 1) % h);
        let lean = |q: usize| if q == 0 { left } else { x }; // x - 1 + p
        let lean_right = |q: usize| if q == 0 { x } else { right }; // x + p
        match i % 6 {
            0 => (right, y),
            1 => (lean_right(p), up),
            2 => (lean(p), up),
            3 => (left, y),
            4 => (lean(p), down),
            _ => (lean_right(p), down),
        }
    }

    /// Fill every open node with a gas at `density` particles per direction (between 0
    /// and 1), moving east at `speed`, drawn at random from the equilibrium.
    pub fn fill(&mut self, density: f64, speed: f64) {
        let p: [f64; DIRECTIONS] =
            std::array::from_fn(|i| equilibrium(self.kind, density, speed, i));
        for c in &mut self.cells {
            if *c & WALL == 0 {
                *c = (*c & CHIRAL) | random_node(&mut self.rng, &p);
            }
        }
    }

    /// Fill the open nodes within `radius` (node spacings) of the point `(cx, cy)` (as in
    /// [`Lattice::position`]) with a gas at rest at `density`.
    pub fn fill_disc(&mut self, cx: f64, cy: f64, radius: f64, density: f64) {
        let p: [f64; DIRECTIONS] = std::array::from_fn(|i| equilibrium(self.kind, density, 0.0, i));
        for y in 0..self.height {
            for x in 0..self.width {
                let (px, py) = self.kind.position(x, y);
                if (px - cx).hypot(py - cy) <= radius && !self.is_wall(x, y) {
                    let bits = random_node(&mut self.rng, &p);
                    self.set_particles(x, y, bits);
                }
            }
        }
    }

    /// Total particles, walls included.
    pub fn particles(&self) -> u64 {
        self.cells
            .iter()
            .map(|&c| u64::from(COUNT[c as usize]))
            .sum()
    }

    /// Total momentum in [`Kind::momentum`]'s integer units, walls included (for the
    /// conservation tests).
    #[cfg(test)]
    pub fn momentum(&self) -> (i64, i64) {
        self.momentum_of(0..self.cells.len())
    }

    /// The particles in columns `xs`.
    #[cfg(test)]
    pub fn particles_in_columns(&self, xs: std::ops::Range<usize>) -> u64 {
        (0..self.height)
            .flat_map(|y| xs.clone().map(move |x| (x, y)))
            .map(|(x, y)| u64::from(COUNT[self.get(x, y) as usize]))
            .sum()
    }

    /// The momentum of the nodes at the given indices.
    #[cfg(test)]
    pub fn momentum_of(&self, nodes: impl Iterator<Item = usize>) -> (i64, i64) {
        let (mut px, mut py) = (0, 0);
        for c in nodes.map(|k| self.cells[k]) {
            for (i, &(mx, my)) in self.kind.momentum().iter().enumerate() {
                if c & (1 << i) != 0 {
                    px += i64::from(mx);
                    py += i64::from(my);
                }
            }
        }
        (px, py)
    }

    /// One step: collide at every node, stream every particle to its neighbour, then
    /// refill the inflow band, if any.
    pub fn step(&mut self) {
        let table = self.kind.collisions();
        for c in &mut self.cells {
            *c = table[*c as usize];
        }
        self.stream();
        if let Some(inflow) = self.inflow {
            self.refill(inflow);
        }
        self.steps += 1;
    }

    /// Move every particle one node along its direction, into `next`, then swap.
    ///
    /// A node's new bit `i` comes from its neighbour in direction `i + 3`, so each new row
    /// is gathered from the rows above, below and itself at fixed offsets. Its wall and
    /// random bits stay where they are.
    fn stream(&mut self) {
        let (w, h) = (self.width, self.height);
        for y in 0..h {
            let p = y % 2;
            let up = &self.cells[((y + h - 1) % h) * w..][..w];
            let row = &self.cells[y * w..][..w];
            let down = &self.cells[((y + 1) % h) * w..][..w];
            let out = &mut self.next[y * w..][..w];
            match self.kind {
                Kind::Hex => stream_row(out, up, row, down, p),
                Kind::Square => stream_row_square(out, up, row, down),
            }
        }
        std::mem::swap(&mut self.cells, &mut self.next);
    }

    fn refill(&mut self, inflow: Inflow) {
        let p: [f64; DIRECTIONS] =
            std::array::from_fn(|i| equilibrium(self.kind, inflow.density, inflow.speed, i));
        let w = self.width;
        for y in 0..self.height {
            for c in &mut self.cells[y * w..y * w + inflow.width.min(w)] {
                if *c & WALL == 0 {
                    *c = (*c & CHIRAL) | random_node(&mut self.rng, &p);
                }
            }
        }
    }
}

/// A node's particles, each direction present with its probability in `p`.
fn random_node(rng: &mut Rng, p: &[f64; DIRECTIONS]) -> u8 {
    let mut bits = 0;
    for (i, &pi) in p.iter().enumerate() {
        if rng.chance(pi) {
            bits |= 1 << i;
        }
    }
    bits
}

/// The bit of a particle moving east.
pub const E: u8 = 1;
const NE: u8 = 1 << 1;
const NW: u8 = 1 << 2;
const W: u8 = 1 << 3;
const SW: u8 = 1 << 4;
const SE: u8 = 1 << 5;
const STAY: u8 = WALL | CHIRAL;

/// One row of streaming. `p` is the row's parity: the nodes above and below an even row's
/// node `x` are `x - 1` and `x`; for an odd row, `x` and `x + 1`.
///
/// A particle arriving moving east comes from the west neighbour, one moving north-east
/// from the south-west neighbour (in the row below), and so on round.
fn stream_row(out: &mut [u8], up: &[u8], row: &[u8], down: &[u8], p: usize) {
    let w = row.len();
    // The interior, x = 1 to w - 2, as slices of one length, so the loop has no bounds
    // checks and vectorises. `x - 1 + p` starts at p, `x + p` at 1 + p.
    let n = w - 2;
    let (l, r) = (&row[..n], &row[2..]);
    let (c, o) = (&row[1..=n], &mut out[1..=n]);
    let (dl, dr) = (&down[p..p + n], &down[1 + p..1 + p + n]);
    let (ul, ur) = (&up[p..p + n], &up[1 + p..1 + p + n]);
    for k in 0..n {
        o[k] = (c[k] & STAY)
            | (l[k] & E)
            | (r[k] & W)
            | (dl[k] & NE)
            | (dr[k] & NW)
            | (ur[k] & SW)
            | (ul[k] & SE);
    }
    // The two edge nodes, wrapping around.
    for x in [0, w - 1] {
        let left = (x + w - 1) % w;
        let right = (x + 1) % w;
        let (lean, lean_right) = if p == 0 { (left, x) } else { (x, right) };
        out[x] = (row[x] & STAY)
            | (row[left] & E)
            | (row[right] & W)
            | (down[lean] & NE)
            | (down[lean_right] & NW)
            | (up[lean_right] & SW)
            | (up[lean] & SE);
    }
}

/// One row of streaming on the square lattice: east from the west, north from below,
/// west from the east, south from above.
fn stream_row_square(out: &mut [u8], up: &[u8], row: &[u8], down: &[u8]) {
    const N: u8 = 1 << 1;
    const W: u8 = 1 << 2;
    const S: u8 = 1 << 3;
    let w = row.len();
    for x in 0..w {
        let (left, right) = (
            if x == 0 { w - 1 } else { x - 1 },
            if x + 1 == w { 0 } else { x + 1 },
        );
        out[x] = (row[x] & STAY) | (row[left] & E) | (down[x] & N) | (row[right] & W) | (up[x] & S);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(state: u8, i: usize) -> i32 {
        i32::from(state >> (i % 6) & 1)
    }

    /// Equation (3.4) of Frisch et al. (1987), transcribed term by term: the change in
    /// `n_i` from a collision, given the node's six bits and the random bit `xi`.
    fn delta(s: u8, i: usize, xi: i32) -> i32 {
        let m = |j: usize| n(s, i + j);
        let e = |j: usize| 1 - m(j);
        xi * m(1) * m(4) * e(0) * e(2) * e(3) * e(5)
            + (1 - xi) * m(2) * m(5) * e(0) * e(1) * e(3) * e(4)
            - m(0) * m(3) * e(1) * e(2) * e(4) * e(5)
            + m(1) * m(3) * m(5) * e(0) * e(2) * e(4)
            - m(0) * m(2) * m(4) * e(1) * e(3) * e(5)
    }

    /// The table must be the source's microdynamical equation (3.4), for all 64 states and
    /// both values of the random bit.
    #[test]
    fn collisions_follow_equation_3_4() {
        for s in 0..64u8 {
            for xi in [0, 1] {
                let state = s | if xi == 1 { CHIRAL } else { 0 };
                let out = COLLIDE[state as usize];
                for i in 0..6 {
                    assert_eq!(
                        n(out, i),
                        n(s, i) + delta(s, i, xi),
                        "state {s:06b} xi {xi}"
                    );
                }
            }
        }
    }

    /// d'Humières and Lallemand (1987, p. 602): FHP-I's rules change "five configurations
    /// among the 64 possible ones": three head-on pairs and two triples.
    #[test]
    fn exactly_five_states_collide() {
        let changed = (0..64u8)
            .filter(|&s| COLLIDE[s as usize] & PARTICLES != s)
            .collect::<Vec<_>>();
        assert_eq!(
            changed,
            vec![0b00_1001, 0b01_0010, 0b01_0101, 0b10_0100, 0b10_1010]
        );
    }

    /// Frisch et al. (1987, equation (8.32) and Table 1): for FHP-I the Reynolds number per
    /// unit Mach number and length, `c_s g / nu`, is largest, 0.387, at density d = 0.187.
    #[test]
    fn reynolds_coefficient_peaks_where_the_paper_says() {
        let (best, d) = (1..1000)
            .map(|k| {
                let d = k as f64 / 2000.0;
                (SOUND_SPEED * advection_factor(d) / viscosity(d), d)
            })
            .fold((0.0, 0.0), |a, b| if b.0 > a.0 { b } else { a });
        assert!((best - 0.387).abs() < 0.0005, "{best}");
        assert!((d - 0.187).abs() < 0.001, "{d}");
    }

    /// Every collision keeps the node's particle count and momentum, exactly; a wall keeps
    /// the count and reverses the momentum.
    #[test]
    fn collisions_conserve_mass_and_momentum() {
        let mom = |s: u8| {
            (0..6)
                .filter(|&i| s >> i & 1 == 1)
                .fold((0, 0), |(x, y), i| (x + MOMENTUM[i].0, y + MOMENTUM[i].1))
        };
        for s in 0..=255u8 {
            let out = COLLIDE[s as usize];
            let (a, b) = (s & PARTICLES, out & PARTICLES);
            assert_eq!(a.count_ones(), b.count_ones(), "state {s:08b}");
            assert_eq!(out & WALL, s & WALL, "the wall bit stays");
            let (m, m2) = (mom(a), mom(b));
            if s & WALL != 0 {
                assert_eq!(m2, (-m.0, -m.1), "a wall reverses momentum");
            } else {
                assert_eq!(m2, m, "state {s:08b}");
            }
        }
    }

    /// The random bit flips exactly when a collision changes the particles (d'Humières and
    /// Lallemand §3.2), so a head-on pair at the same node alternates which way it turns.
    #[test]
    fn the_random_bit_flips_on_each_collision() {
        for s in 0..=255u8 {
            let out = COLLIDE[s as usize];
            let collided = s & WALL == 0 && out & PARTICLES != s & PARTICLES;
            assert_eq!(out & CHIRAL != s & CHIRAL, collided, "state {s:08b}");
        }
    }

    /// Each neighbour is one node spacing away, at angle `i * 60°` (y up), from both kinds
    /// of row: the offset-row layout is a true triangular lattice.
    #[test]
    fn neighbours_are_at_unit_distance_and_sixty_degrees() {
        let l = Lattice::new(Kind::Hex, 10, 8, 1);
        for (x, y) in [(4, 4), (4, 5)] {
            let (px, py) = Kind::Hex.position(x, y);
            for i in 0..6 {
                let (nx, ny) = l.neighbour(x, y, i);
                let (qx, qy) = Kind::Hex.position(nx, ny);
                let (dx, dy) = (qx - px, -(qy - py));
                let (cx, cy) = Kind::Hex.velocity(i);
                assert!(
                    (dx - cx).abs() < 1e-12 && (dy - cy).abs() < 1e-12,
                    "({x}, {y}) i {i}"
                );
            }
        }
    }

    /// Streaming moves every particle to its neighbour, including across every edge of
    /// the wrapping world, from rows of both parities. A lone particle sees no collision.
    #[test]
    fn a_lone_particle_moves_to_its_neighbour_everywhere() {
        let (w, h) = (5, 4);
        for y in 0..h {
            for x in 0..w {
                for i in 0..6 {
                    let mut l = Lattice::new(Kind::Hex, w, h, 3);
                    l.set_particles(x, y, 1 << i);
                    let (nx, ny) = l.neighbour(x, y, i);
                    l.step();
                    assert_eq!(l.particles(), 1);
                    assert_eq!(l.get(nx, ny) & PARTICLES, 1 << i, "({x}, {y}) dir {i}");
                }
            }
        }
    }

    fn random_gas(w: usize, h: usize, density: f64, seed: u64) -> Lattice {
        let mut l = Lattice::new(Kind::Hex, w, h, seed);
        l.fill(density, 0.0);
        l
    }

    /// Streaming is a permutation of particles: after streaming a random gas once (no
    /// collision phase), every particle is at the neighbour in its direction of the node
    /// it left.
    #[test]
    fn streaming_is_a_permutation() {
        let mut l = random_gas(9, 6, 0.4, 5);
        let before = l.cells().to_vec();
        l.stream();
        for y in 0..6 {
            for x in 0..9 {
                for i in 0..6 {
                    let (nx, ny) = l.neighbour(x, y, i);
                    assert_eq!(
                        before[y * 9 + x] >> i & 1,
                        l.get(nx, ny) >> i & 1,
                        "({x}, {y}) dir {i}"
                    );
                }
            }
        }
    }

    /// On a wrapping world with no walls, every step conserves the particle count and the
    /// momentum exactly, at any density, including a flowing gas.
    #[test]
    fn steps_conserve_mass_and_momentum() {
        for (density, speed, seed) in [(0.2, 0.0, 1), (0.5, 0.0, 2), (0.3, 0.3, 3), (0.8, 0.1, 4)] {
            let mut l = Lattice::new(Kind::Hex, 40, 30, seed);
            l.fill(density, speed);
            let (mass, mom) = (l.particles(), l.momentum());
            for _ in 0..50 {
                l.step();
                assert_eq!((l.particles(), l.momentum()), (mass, mom));
            }
        }
    }

    /// Walls keep the particle count (they bounce particles, never absorb them).
    #[test]
    fn walls_conserve_particles() {
        let mut l = random_gas(30, 20, 0.3, 9);
        for y in 5..15 {
            l.set_wall(12, y, true);
        }
        let mass = l.particles();
        for _ in 0..40 {
            l.step();
            assert_eq!(l.particles(), mass);
        }
    }

    /// A hand trace of bounce-back: a particle sent east into a wall two nodes away
    /// reaches the wall, is turned round there, and is back where it started, moving west,
    /// after 4 steps.
    #[test]
    fn a_particle_bounces_back_off_a_wall() {
        let mut l = Lattice::new(Kind::Hex, 8, 4, 1);
        l.set_wall(4, 2, true);
        l.set_particles(2, 2, E);
        l.step();
        assert_eq!(l.get(3, 2) & PARTICLES, E);
        l.step(); // streams into the wall node
        assert_eq!(l.get(4, 2) & PARTICLES, E);
        l.step(); // reversed there, streams back out
        assert_eq!(l.get(3, 2) & PARTICLES, W);
        l.step();
        assert_eq!(l.get(2, 2) & PARTICLES, W);
        assert_eq!(l.particles(), 1);
    }

    /// A hand trace of a head-on collision: two particles two nodes apart, moving at each
    /// other along a row, meet at the middle node after one step, turn by 60° there (one
    /// way or the other, by the node's random bit), and leave along the pair of directions
    /// the turn chose.
    #[test]
    fn a_head_on_pair_turns_sixty_degrees() {
        for chiral in [false, true] {
            let mut l = Lattice::new(Kind::Hex, 8, 6, 1);
            l.set_particles(2, 2, E);
            l.set_particles(4, 2, W);
            // Fix the middle node's random bit.
            let mid = 2 * 8 + 3;
            l.cells[mid] = if chiral { CHIRAL } else { 0 };
            l.step();
            assert_eq!(l.get(3, 2) & PARTICLES, E | W);
            l.step();
            // Random bit set: (0, 3) turns by -60° to (5, 2); clear: by +60° to (1, 4).
            let (a, b) = if chiral { (5, 2) } else { (1, 4) };
            let (ax, ay) = l.neighbour(3, 2, a);
            let (bx, by) = l.neighbour(3, 2, b);
            assert_eq!(l.get(ax, ay) & PARTICLES, 1 << a, "chiral {chiral}");
            assert_eq!(l.get(bx, by) & PARTICLES, 1 << b, "chiral {chiral}");
            assert_eq!(l.particles(), 2);
            assert_eq!(l.get(3, 2) & CHIRAL != 0, !chiral, "the bit flipped");
        }
    }

    /// A hand trace of a triple collision: three particles converging on one node from
    /// 120° apart, moving in directions 0, 2 and 4, leave it in directions 1, 3 and 5.
    #[test]
    fn a_triple_collision_turns_the_triple() {
        let mut l = Lattice::new(Kind::Hex, 8, 6, 1);
        let (cx, cy) = (4, 3);
        for i in [0, 2, 4] {
            // Start one node behind the centre, moving towards it.
            let (sx, sy) = l.neighbour(cx, cy, i + 3);
            l.set_particles(sx, sy, 1 << i);
        }
        l.step();
        assert_eq!(l.get(cx, cy) & PARTICLES, 0b01_0101);
        l.step();
        for i in [1, 3, 5] {
            let (nx, ny) = l.neighbour(cx, cy, i);
            assert_eq!(l.get(nx, ny) & PARTICLES, 1 << i, "dir {i}");
        }
    }

    /// Collisions drive the gas to equilibrium, which at zero momentum has the same
    /// population in all six directions (Frisch et al. 1987, section 4: the equilibrium
    /// populations depend only on the conserved mass and momentum). A gas started with
    /// particles moving only east and west must spread evenly over all six. Each node starts
    /// with an east-west pair or nothing, so the momentum is exactly zero. With about 16,000
    /// particles, a direction's share has a sampling spread of about 0.003, so 0.01 is
    /// three of those.
    #[test]
    fn a_gas_relaxes_to_equal_populations() {
        let n = 128;
        let mut l = Lattice::new(Kind::Hex, n, n, 11);
        let mut rng = Rng::new(4);
        for y in 0..n {
            for x in 0..n {
                if rng.chance(0.5) {
                    l.set_particles(x, y, E | W);
                }
            }
        }
        assert_eq!(l.momentum(), (0, 0));
        for _ in 0..200 {
            l.step();
        }
        let total = l.particles() as f64;
        for i in 0..6 {
            let count = l.cells().iter().filter(|&&c| c >> i & 1 == 1).count() as f64;
            let share = count / total;
            assert!((share - 1.0 / 6.0).abs() < 0.01, "direction {i}: {share}");
        }
    }

    /// One step of an empty lattice with a wind: only the band is refilled, at density d
    /// per direction and mean velocity u, and a wall inside the band stays empty.
    #[test]
    fn the_inflow_band_refills_at_its_density_and_speed() {
        let (w, h, band) = (20, 4000, 4);
        let mut l = Lattice::new(Kind::Hex, w, h, 6);
        l.set_wall(1, 7, true);
        l.inflow = Some(Inflow {
            width: band,
            density: 0.2,
            speed: 0.25,
        });
        l.step();
        assert_eq!(l.get(1, 7) & PARTICLES, 0, "the wall in the band");
        assert_eq!(
            l.particles(),
            l.particles_in_columns(0..band),
            "nothing outside"
        );
        let nodes = (band * h - 1) as f64; // less the wall
        let n = l.particles() as f64;
        let (px, py) = l.momentum();
        let (ux, uy) = (px as f64 / 2.0 / n, py as f64 * 3f64.sqrt() / 4.0 / n);
        assert!(
            (n / nodes / 6.0 - 0.2).abs() < 0.005,
            "density {}",
            n / nodes / 6.0
        );
        assert!(
            (ux - 0.25).abs() < 0.02 && uy.abs() < 0.02,
            "u = ({ux}, {uy})"
        );
    }

    /// Filling (and the inflow band, which refills the same way) draws from the
    /// equilibrium of d'Humières and Lallemand's (2.5): at density d and speed u, the
    /// populations are d(1 + 2 c_i . u), so the mean velocity per particle is u.
    #[test]
    fn a_fill_carries_its_speed() {
        let mut l = Lattice::new(Kind::Hex, 400, 200, 2);
        l.fill(0.2, 0.25);
        let (px, py) = l.momentum();
        let ux = px as f64 / 2.0 / l.particles() as f64;
        let uy = py as f64 * 3f64.sqrt() / 4.0 / l.particles() as f64;
        assert!(
            (ux - 0.25).abs() < 0.01 && uy.abs() < 0.01,
            "u = ({ux}, {uy})"
        );
    }

    /// HPP's table must be Frisch et al.'s microdynamical equation (3.3), for all 16
    /// states: `Delta_i = n_{i+1} n_{i+3} (1 - n_i)(1 - n_{i+2})
    /// - n_i n_{i+2} (1 - n_{i+1})(1 - n_{i+3})`, indices modulo 4.
    #[test]
    fn hpp_collisions_follow_equation_3_3() {
        let n = |s: u8, i: usize| i32::from(s >> (i % 4) & 1);
        for s in 0..16u8 {
            let out = COLLIDE_SQUARE[s as usize];
            for i in 0..4 {
                let m = |j: usize| n(s, i + j);
                let e = |j: usize| 1 - m(j);
                let delta = m(1) * m(3) * e(0) * e(2) - m(0) * m(2) * e(1) * e(3);
                assert_eq!(n(out, i), n(s, i) + delta, "state {s:04b}");
            }
        }
    }

    /// HPP's collisions keep each node's particle count and momentum; walls reverse it.
    #[test]
    fn hpp_collisions_conserve_mass_and_momentum() {
        let mom = |s: u8| {
            (0..4)
                .filter(|&i| s >> i & 1 == 1)
                .fold((0, 0), |(x, y), i| {
                    (x + MOMENTUM_SQUARE[i].0, y + MOMENTUM_SQUARE[i].1)
                })
        };
        for s in (0..=255u8).filter(|s| s & 0b11_0000 == 0) {
            let out = COLLIDE_SQUARE[s as usize];
            assert_eq!((s & 15).count_ones(), (out & 15).count_ones(), "{s:08b}");
            assert_eq!(out & 0b11_0000, 0, "no hex bits appear");
            let (m, m2) = (mom(s), mom(out));
            if s & WALL != 0 {
                assert_eq!(m2, (-m.0, -m.1));
            } else {
                assert_eq!(m2, m, "{s:08b}");
            }
        }
    }

    /// On the square lattice, streaming moves a lone particle to its neighbour, across
    /// every edge of the wrapping world.
    #[test]
    fn hpp_a_lone_particle_moves_to_its_neighbour_everywhere() {
        let (w, h) = (4, 3);
        for y in 0..h {
            for x in 0..w {
                for i in 0..4 {
                    let mut l = Lattice::new(Kind::Square, w, h, 3);
                    l.set_particles(x, y, 1 << i);
                    let (nx, ny) = l.neighbour(x, y, i);
                    l.step();
                    assert_eq!(l.get(nx, ny) & PARTICLES, 1 << i, "({x}, {y}) dir {i}");
                }
            }
        }
    }

    /// Frisch et al. (1987, §2.1): in HPP, "momentum is conserved along each horizontal and
    /// vertical line, resulting in far too many conserved quantities for physical modeling".
    /// Every row's x-momentum and every column's y-momentum stay exactly as they were. FHP
    /// has no such invariant: its rows trade momentum.
    #[test]
    fn hpp_keeps_each_rows_momentum_and_fhp_does_not() {
        let (w, h) = (30, 20);
        let rows = |l: &Lattice| -> Vec<i64> {
            (0..h)
                .map(|y| l.momentum_of(y * w..(y + 1) * w).0)
                .collect()
        };
        let columns = |l: &Lattice| -> Vec<i64> {
            (0..w)
                .map(|x| l.momentum_of((0..h).map(|y| y * w + x)).1)
                .collect()
        };
        let mut hpp = Lattice::new(Kind::Square, w, h, 5);
        hpp.fill(0.3, 0.2);
        let (r0, c0) = (rows(&hpp), columns(&hpp));
        for _ in 0..50 {
            hpp.step();
            assert_eq!((rows(&hpp), columns(&hpp)), (r0.clone(), c0.clone()));
        }
        let mut fhp = Lattice::new(Kind::Hex, w, h, 5);
        fhp.fill(0.3, 0.2);
        let r0 = rows(&fhp);
        for _ in 0..5 {
            fhp.step();
        }
        assert_ne!(rows(&fhp), r0);
    }

    /// The square lattice's equilibrium fill moves at the speed asked for, as the hex one
    /// does.
    #[test]
    fn hpp_fill_carries_its_speed() {
        let mut l = Lattice::new(Kind::Square, 300, 300, 2);
        l.fill(0.2, 0.25);
        let (px, py) = l.momentum();
        let n = l.particles() as f64;
        let (ux, uy) = (px as f64 / n, py as f64 / n);
        assert!(
            (ux - 0.25).abs() < 0.01 && uy.abs() < 0.01,
            "u = ({ux}, {uy})"
        );
    }
}
