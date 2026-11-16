//! The Mandelbrot set, pixel by pixel. Each pixel stands for a point c of the complex plane,
//! and iterates z -> z² + c from z = 0. It records the first step at which |z| passes 2 (from
//! then on |z| grows without bound, so c is outside the set), or that it hasn't yet.
//!
//! Every pixel keeps its z and its step count, so raising the iteration limit carries on
//! from where the last limit stopped rather than starting again. That is what lets the demo
//! animate the limit one step at a time, like a generation of the other demos.
//!
//! Deep views of the Mandelbrot set (zoomed in past about 3 x 10^9, short of where f64 runs out
//! of digits) are computed by *perturbation*: see [`Reference`].

use crate::dd::Dd;

/// A rectangle of the complex plane, cut into `width` x `height` pixels.
///
/// The plane point of a pixel is computed from a whole-pixel offset, so moving the view by a
/// whole number of pixels gives every pixel exactly the point some pixel had before. That is
/// what lets [`Picture::pan`] keep the work already done.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: usize,
    pub height: usize,
    /// The plane point at the picture's centre, before `offset`, in double-double so a deep
    /// view can be placed precisely. Flat (shallow) views use only `hi`.
    base: [Dd; 2],
    /// Plane units per pixel.
    step: f64,
    /// Whole pixels the view has been moved by since `base` was set.
    offset: [i64; 2],
    /// Whether this view may zoom past f64's precision, using perturbation (the Mandelbrot
    /// set does; a Julia set doesn't).
    deep: bool,
}

/// The smallest step allowed, relative to the size of the coordinates: below about 16 units
/// in the last place, neighbouring pixels would round to the same f64 point and the picture
/// would turn into blocks.
const PRECISION: f64 = 16.0 * f64::EPSILON;

/// The same for a deep view, whose coordinates are double-double: about 2^-104 a unit.
const DEEP_PRECISION: f64 = 16.0 * f64::EPSILON * f64::EPSILON * 2.0;

/// A deep view switches to perturbation once its step is below this, relative to the size of
/// the coordinates: a few hundred times above where f64 itself gives out, so the plain
/// path never gets near its limit.
const PERTURB_BELOW: f64 = 1e-12;

/// The widest view allowed, in plane units across its shorter side. The whole set fits in a
/// disc of radius 2, so there is nothing to see further out.
const WIDEST: f64 = 12.0;

impl Viewport {
    pub fn new(width: usize, height: usize, centre: [f64; 2], step: f64) -> Self {
        Self::new_dd(
            width,
            height,
            [Dd::new(centre[0]), Dd::new(centre[1])],
            step,
        )
    }

    fn new_dd(width: usize, height: usize, centre: [Dd; 2], step: f64) -> Self {
        Self {
            width,
            height,
            base: centre,
            step,
            offset: [0, 0],
            deep: false,
        }
    }

    /// This view, allowed to zoom past f64's precision (or not).
    pub fn with_deep(&self, deep: bool) -> Self {
        Self { deep, ..*self }
    }

    /// Whether pictures of this view are computed by perturbation.
    pub fn perturbed(&self) -> bool {
        self.deep && self.step < PERTURB_BELOW * self.scale_of(self.base_f64())
    }

    fn base_f64(&self) -> [f64; 2] {
        [self.base[0].hi, self.base[1].hi]
    }

    /// The point perturbation is measured from: `base`, which panning leaves alone, so a
    /// panned picture keeps its reference and its pixels' work.
    pub fn reference_point(&self) -> [Dd; 2] {
        self.base
    }

    /// Pixel `(x, y)`'s point less [`Viewport::reference_point`]: small, and exact in f64.
    pub fn offset_from_reference(&self, x: usize, y: usize) -> [f64; 2] {
        let i = (x as i64 + self.offset[0]) as f64 + 0.5 - self.width as f64 * 0.5;
        let j = (y as i64 + self.offset[1]) as f64 + 0.5 - self.height as f64 * 0.5;
        [i * self.step, -j * self.step]
    }

    /// Pixel `(x, y)`'s point in full, for the pointer readout of a deep view.
    pub fn point_dd(&self, x: usize, y: usize) -> [Dd; 2] {
        let [dx, dy] = self.offset_from_reference(x, y);
        [self.base[0] + Dd::new(dx), self.base[1] + Dd::new(dy)]
    }

    /// The view centred on `centre` that just shows `span` (plane width and height).
    pub fn fit(width: usize, height: usize, centre: [f64; 2], span: [f64; 2]) -> Self {
        let step = (span[0] / width.max(1) as f64).max(span[1] / height.max(1) as f64);
        Self::new(width, height, centre, step)
    }

    /// Plane units per pixel.
    pub fn step(&self) -> f64 {
        self.step
    }

    pub fn centre(&self) -> [f64; 2] {
        self.at(self.width as f64 / 2.0, self.height as f64 / 2.0)
    }

    /// The plane point of pixel `(x, y)`: the centre of that pixel.
    pub fn point(&self, x: usize, y: usize) -> [f64; 2] {
        // Whole numbers first, so the same pixel of the plane gets the same bits however the
        // view reached it.
        let i = (x as i64 + self.offset[0]) as f64 + 0.5 - self.width as f64 * 0.5;
        let j = (y as i64 + self.offset[1]) as f64 + 0.5 - self.height as f64 * 0.5;
        let [b0, b1] = self.base_f64();
        [b0 + i * self.step, b1 - j * self.step]
    }

    /// The plane point at `(x, y)` in pixels from the picture's top-left corner, fractions
    /// allowed: pixel `(i, j)` covers `i..i + 1` by `j..j + 1`. The plane's imaginary axis
    /// points up, the picture's y down.
    pub fn at(&self, x: f64, y: f64) -> [f64; 2] {
        let i = x + self.offset[0] as f64 - self.width as f64 * 0.5;
        let j = y + self.offset[1] as f64 - self.height as f64 * 0.5;
        let [b0, b1] = self.base_f64();
        [b0 + i * self.step, b1 - j * self.step]
    }

    /// Where point `(x, y)` of `other` (in its pixels) falls in this view, in pixels: like
    /// `self.pixel_of(other.at(x, y))`, but exact enough for deep views, whose points don't
    /// fit an f64. For drawing an older picture scaled under a newer one.
    pub fn pixel_of_view(&self, other: &Viewport, x: f64, y: f64) -> [f64; 2] {
        let d = [
            (other.base[0] - self.base[0]).to_f64(),
            (other.base[1] - self.base[1]).to_f64(),
        ];
        let i = x + other.offset[0] as f64 - other.width as f64 * 0.5;
        let j = y + other.offset[1] as f64 - other.height as f64 * 0.5;
        let (px, py) = (d[0] + i * other.step, d[1] - j * other.step);
        [
            px / self.step - self.offset[0] as f64 + self.width as f64 * 0.5,
            -py / self.step - self.offset[1] as f64 + self.height as f64 * 0.5,
        ]
    }

    /// Where the plane point `p` falls, in pixels from the top-left corner: the inverse of
    /// [`Viewport::at`].
    pub fn pixel_of(&self, p: [f64; 2]) -> [f64; 2] {
        let [b0, b1] = self.base_f64();
        let x = (p[0] - b0) / self.step - self.offset[0] as f64 + self.width as f64 * 0.5;
        let y = (b1 - p[1]) / self.step - self.offset[1] as f64 + self.height as f64 * 0.5;
        [x, y]
    }

    /// This view magnified by `factor` about `(x, y)` (in pixels), which stays put on screen.
    /// The magnification stops where f64 runs out of precision, and where the view is wide
    /// enough to show everything.
    pub fn zoomed(&self, x: f64, y: f64, factor: f64) -> Self {
        let step = self.clamp_step(self.step / factor, self.at(x, y));
        // The new centre, worked out in double-double: in a deep view its digits run past
        // f64's. It is the point under (x, y), moved back by (x, y)'s distance from the
        // centre at the new step.
        let (cx, cy) = (self.width as f64 * 0.5, self.height as f64 * 0.5);
        let i = x + self.offset[0] as f64 - cx;
        let j = y + self.offset[1] as f64 - cy;
        let (di, dj) = ((x - cx) * step, (y - cy) * step);
        let base = [
            self.base[0] + Dd::new(i * self.step) - Dd::new(di),
            self.base[1] - Dd::new(j * self.step) + Dd::new(dj),
        ];
        Self {
            base,
            step,
            offset: [0, 0],
            ..*self
        }
    }

    /// This view with its contents moved `dx` pixels right and `dy` down.
    pub fn panned(&self, dx: i64, dy: i64) -> Self {
        Self {
            offset: [self.offset[0] - dx, self.offset[1] - dy],
            ..*self
        }
    }

    /// The same centre, `width` x `height` pixels of `step` plane units each.
    pub fn resized(&self, width: usize, height: usize, step: f64) -> Self {
        let centre = [
            self.base[0] + Dd::new(self.offset[0] as f64 * self.step),
            self.base[1] - Dd::new(self.offset[1] as f64 * self.step),
        ];
        let mut view = Self::new_dd(width, height, centre, step).with_deep(self.deep);
        view.step = view.clamp_step(step, view.base_f64());
        view
    }

    /// The same zoom and size, centred on `p`.
    pub fn centred_on(&self, p: [f64; 2]) -> Self {
        Self::new(self.width, self.height, p, self.step).with_deep(self.deep)
    }

    /// Whether zooming in further is blocked by f64's precision.
    pub fn at_precision_limit(&self) -> bool {
        self.step <= self.smallest_step(self.centre()) * 1.0001
    }

    fn smallest_step(&self, near: [f64; 2]) -> f64 {
        let precision = if self.deep { DEEP_PRECISION } else { PRECISION };
        precision * self.scale_of(near)
    }

    /// The size of coordinates near `p`, for steps relative to it: never below 1.
    fn scale_of(&self, p: [f64; 2]) -> f64 {
        p[0].abs().max(p[1].abs()).max(1.0)
    }

    fn clamp_step(&self, step: f64, near: [f64; 2]) -> f64 {
        let widest = WIDEST / self.width.min(self.height).max(1) as f64;
        step.max(self.smallest_step(near)).min(widest)
    }
}

/// What a pixel knows so far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pixel {
    /// Not looked at since the picture was started or the view moved.
    Fresh,
    /// Not escaped after this many steps: perhaps in the set, perhaps not yet decided.
    Running(u32),
    /// In the main cardioid or the large disc to its left, which are known to be inside the
    /// set without iterating.
    Inside,
    /// Settled into a cycle, found at this step: inside the set, so iterating stopped there
    /// (see [`SETTLE_TOLERANCE`]).
    Settled(u32),
    /// |z| first passed 2 at `step`. `smooth` is a fractional version of the step that
    /// varies continuously from pixel to pixel, for colouring without bands: close to `step`,
    /// see [`smooth_count`].
    Escaped { step: u32, smooth: f32 },
}

// `Picture::smooth` codes: NaN while running, these three for fresh, inside and settled,
// and otherwise the escaped pixel's smooth count (never negative, never near f32::MAX).
const FRESH: f32 = f32::NEG_INFINITY;
const INSIDE: f32 = f32::INFINITY;
const SETTLED: f32 = f32::MAX;

/// A point inside the set settles into a cycle, and then never escapes, however long it
/// runs; a pixel that has is stopped. Without this, every pixel inside ran to the stop
/// limit, which near the set's edge is most of the work (on the spike at x8,103, all of it:
/// 13 s natively at 5,000 steps). Each pixel saves its z at steps 1, 2, 4, 8, ... (Brent's
/// method) and is taken to have settled once a later z comes back within this distance of
/// the saved one. Too large, and a point just outside the set, lingering near a cycle before
/// it escapes, would be taken for inside: see the test `settled_pixels_never_escape`.
const SETTLE_TOLERANCE: f64 = 1e-10;

/// The tolerance never exceeds this fraction of a pixel's width (as Fractint did): a fixed
/// 1e-10 stopped points just outside a small copy of the set as inside once a pixel was
/// narrower than that (near the period-10 copy at -1.99998588 at x1e9: 2 of 6 such pixels
/// wrong, at x3e9 10 of 48). A tenth, a hundredth and a thousandth all fixed that; a
/// hundredth leaves a margin and still stops nearly as many of the pixels truly inside.
const SETTLE_PER_STEP: f64 = 1e-2;

/// Which set a picture shows. Both iterate z -> z² + c; they differ in what a pixel's point
/// stands for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Family {
    /// The pixel's point is c, and z starts at 0.
    Mandelbrot,
    /// c is fixed, and the pixel's point is where z starts: the filled Julia set of c.
    Julia([f64; 2]),
}

impl Family {
    /// |z| squared past which z is sure to escape: 2, or |c| if that is bigger (see the
    /// README's "Why 2?", whose argument needs |z| > 2 and |z| ≥ |c|). For the Mandelbrot
    /// set c is the pixel, and a pixel past 2 escapes at step 1 anyway.
    fn escape_squared(self) -> f64 {
        match self {
            Self::Mandelbrot => 4.0,
            Self::Julia([cr, ci]) => (cr * cr + ci * ci).max(4.0),
        }
    }
}

/// The Mandelbrot set, or a Julia set, over a [`Viewport`], computed a budgeted piece at a
/// time.
///
/// Raise the limit with [`Picture::raise_limit`] and call [`Picture::work`] each frame until
/// [`Picture::is_done`]. The work is split between frames by iteration count, not time: the
/// browser has no clock for that (`std::time::Instant` panics in wasm).
pub struct Picture {
    family: Family,
    view: Viewport,
    limit: u32,
    z: Vec<[f64; 2]>,
    steps: Vec<u32>,
    smooth: Vec<f32>,
    /// The pixels still running (or fresh), coarse ones first: every fourth pixel each way,
    /// then every second, then the rest, so a rough picture appears quickly.
    queue: Vec<u32>,
    /// The pass through `queue`: entries before `write` are survivors already brought to
    /// `pass_limit`, from `read` on are still to do, and between the two is junk.
    read: usize,
    write: usize,
    pass_limit: u32,
    /// For a deep picture (see [`Viewport::perturbed`]): the reference orbit its pixels are
    /// measured from, and each pixel's place along it. Each pixel's `z` then holds its
    /// difference from the reference's, not z itself.
    reference: Option<Reference>,
    places: Vec<u32>,
    /// Each pixel's z at the last step that was a power of two, to tell when it has settled
    /// into a cycle (see [`SETTLE_TOLERANCE`]).
    saved: Vec<[f64; 2]>,
    /// [`SETTLE_TOLERANCE`], or less in a view whose pixels are narrower than it.
    tolerance: f64,
    /// For a deep picture, each pixel's place on the reference orbit when `saved` was saved
    /// (`saved` then holds its dz, not its z).
    saved_places: Vec<u32>,
}

/// Perturbation, for views too deep for f64. A pixel's point c is not representable in f64
/// there, but its difference dc from a nearby reference point C is, easily. Iterate C in
/// double-double once, rounding each Z to f64 (|Z| <= 2, so that loses nothing that
/// matters), and every pixel then iterates only its own difference dz = z - Z, in f64:
///
/// > dz -> (2 Z + dz) dz + dc
///
/// which is exactly what z -> z² + c becomes when z = Z + dz and c = C + dc.
///
/// A pixel's orbit can wander away from the reference's, after which dz is no longer small
/// and the rounding in it no longer harmless. *Rebasing* (Zhuoran, 2021) fixes that with no
/// second reference: whenever |z| < |dz|, carry on from Z_0 = 0 instead, with dz = z (which
/// is what Z_0 + dz must then be). The same is done when an escaping reference runs out.
struct Reference {
    c: [Dd; 2],
    /// Z_0 = 0, Z_1 = C, Z_2, ..., as far as the limit, or to where the reference escaped,
    /// or round its cycle once.
    orbit: Vec<[f64; 2]>,
    /// The last Z, in full, to carry on from when the limit is raised.
    z: [Dd; 2],
    escaped: bool,
    /// Z at the last index that was a power of two, in full, and that index: to tell when
    /// the reference has settled into a cycle (as pixels do, see [`SETTLE_TOLERANCE`]).
    saved: [Dd; 2],
    saved_index: usize,
    /// Once settled: the earlier index its last Z matches. A pixel reaching the end of the
    /// orbit carries on from there, so the orbit never needs extending again, and a pixel's
    /// place along it repeats, which its own cycle check relies on.
    cycle_from: Option<usize>,
    /// How close two Zs must be to count as the same: a thousandth of a pixel, so carrying
    /// on from the earlier one moves no pixel visibly.
    tolerance: f64,
}

/// How far a reference's orbit is searched for a cycle when a picture starts (a few
/// milliseconds of double-double arithmetic in the browser). Past this it is extended
/// without searching.
const CYCLE_SEARCH: usize = 4096;

/// What a double-double step of the reference costs, counted in plain iterations.
const REFERENCE_COST: u64 = 6;

impl Reference {
    fn new(c: [Dd; 2], step: f64) -> Self {
        let mut reference = Self {
            c,
            orbit: vec![[0.0, 0.0]],
            z: [Dd::ZERO, Dd::ZERO],
            escaped: false,
            saved: [Dd::ZERO, Dd::ZERO],
            saved_index: 0,
            cycle_from: None,
            tolerance: (step * 1e-3).min(1e-20),
        };
        // Look for a cycle now, over a fixed number of steps, not as the limit rises: which
        // loop the pixels run depends on it (see `advance_perturbed`), and a picture must
        // come out the same however its limit was raised.
        while !reference.escaped
            && reference.cycle_from.is_none()
            && reference.orbit.len() <= CYCLE_SEARCH
        {
            reference.step(true);
        }
        reference
    }

    /// One more Z, looking for a cycle if `search`.
    fn step(&mut self, search: bool) {
        let [zr, zi] = self.z;
        let next = [
            zr * zr - zi * zi + self.c[0],
            (zr * zi).scale(2.0) + self.c[1],
        ];
        let rounded = [next[0].to_f64(), next[1].to_f64()];
        self.z = next;
        self.orbit.push(rounded);
        self.escaped = rounded[0] * rounded[0] + rounded[1] * rounded[1] > 4.0;
        if !search {
            return;
        }
        let index = self.orbit.len() - 1;
        let apart =
            (next[0] - self.saved[0]).to_f64().abs() + (next[1] - self.saved[1]).to_f64().abs();
        if self.saved_index > 0 && apart < self.tolerance {
            self.cycle_from = Some(self.saved_index);
        } else if index.is_power_of_two() {
            (self.saved, self.saved_index) = (next, index);
        }
    }

    /// Compute the orbit on to Z_limit, unless it has escaped. Returns the work done.
    /// Extending it later gives exactly the values computing it all at once would: the
    /// pictures rely on that to come out the same however the limit was raised.
    fn extend(&mut self, limit: u32) -> u64 {
        let mut work = 0;
        while !self.escaped && self.cycle_from.is_none() && self.orbit.len() <= limit as usize {
            self.step(false);
            work += REFERENCE_COST;
        }
        work
    }
}

impl Picture {
    pub fn new(family: Family, view: Viewport, limit: u32) -> Self {
        let n = view.width * view.height;
        let mut picture = Self {
            family,
            view,
            limit,
            z: vec![[0.0; 2]; n],
            steps: vec![0; n],
            smooth: vec![FRESH; n],
            queue: Vec::new(),
            read: 0,
            write: 0,
            pass_limit: limit,
            reference: None,
            places: Vec::new(),
            saved: vec![[0.0; 2]; n],
            tolerance: SETTLE_TOLERANCE.min(view.step() * SETTLE_PER_STEP),
            saved_places: Vec::new(),
        };
        if family == Family::Mandelbrot && view.perturbed() {
            picture.reference = Some(Reference::new(view.reference_point(), view.step()));
            picture.places = vec![0; n];
            picture.saved_places = vec![0; n];
        }
        picture.rebuild_queue();
        picture
    }

    pub fn view(&self) -> &Viewport {
        &self.view
    }

    pub fn family(&self) -> Family {
        self.family
    }

    /// Every pixel is iterated until it escapes or reaches this many steps.
    pub fn limit(&self) -> u32 {
        self.limit
    }

    /// Let every pixel go on to `limit` steps. Lowering it does nothing: the steps already
    /// taken can't be undone (start a new picture instead).
    pub fn raise_limit(&mut self, limit: u32) {
        self.limit = self.limit.max(limit);
    }

    /// Whether every pixel has escaped or reached the limit.
    pub fn is_done(&self) -> bool {
        self.read >= self.queue.len() && self.pass_limit == self.limit
    }

    /// How far through the current pass the work is, from 0 to 1.
    pub fn progress(&self) -> f32 {
        if self.queue.is_empty() {
            1.0
        } else {
            self.read as f32 / self.queue.len() as f32
        }
    }

    pub fn pixel(&self, x: usize, y: usize) -> Pixel {
        let i = y * self.view.width + x;
        let s = self.smooth[i];
        if s == FRESH {
            Pixel::Fresh
        } else if s == INSIDE {
            Pixel::Inside
        } else if s == SETTLED {
            Pixel::Settled(self.steps[i])
        } else if s.is_nan() {
            Pixel::Running(self.steps[i])
        } else {
            Pixel::Escaped {
                step: self.steps[i],
                smooth: s,
            }
        }
    }

    /// Do about `budget` iterations of work, pixel by pixel in the queue's order, and return
    /// how much was done. Each pixel touched also counts a fixed cost for its overhead. A
    /// pixel can be left partway, and carries on next time.
    pub fn work(&mut self, budget: u64) -> u64 {
        let mut spent = match &mut self.reference {
            Some(reference) => reference.extend(self.limit),
            None => 0,
        };
        loop {
            if self.read >= self.queue.len() {
                self.queue.truncate(self.write);
                self.read = self.write;
                if self.pass_limit == self.limit {
                    return spent;
                }
                // The limit was raised: every survivor needs more steps.
                self.pass_limit = self.limit;
                self.read = 0;
                self.write = 0;
                continue;
            }
            if spent >= budget {
                return spent;
            }
            let i = self.queue[self.read] as usize;
            let (cost, finished) = self.advance(i, budget - spent);
            spent += cost;
            if !finished {
                return spent; // the budget ran out partway through this pixel
            }
            if self.smooth[i].is_nan() {
                self.queue[self.write] = i as u32;
                self.write += 1;
            }
            self.read += 1;
        }
    }

    /// Iterate pixel `i` towards the limit, at most `allowance` steps. Returns the work done
    /// (the steps plus the pixel's overhead, or 1 if it needed none) and whether the pixel
    /// is finished: escaped, inside, or at the limit.
    fn advance(&mut self, i: usize, allowance: u64) -> (u64, bool) {
        if self.reference.is_some() {
            return self.advance_perturbed(i, allowance);
        }
        let (x, y) = (i % self.view.width, i / self.view.width);
        let point = self.view.point(x, y);
        let c = match self.family {
            Family::Mandelbrot => point,
            Family::Julia(c) => c,
        };
        if self.smooth[i] == FRESH {
            match self.family {
                Family::Mandelbrot if known_inside(c) => {
                    self.smooth[i] = INSIDE;
                    return (1, true);
                }
                Family::Mandelbrot => {}
                Family::Julia(_) => self.z[i] = point,
            }
            self.saved[i] = self.z[i];
            self.smooth[i] = f32::NAN;
        }
        if !self.smooth[i].is_nan() {
            return (1, true);
        }
        let start = self.steps[i];
        let stop = self
            .limit
            .min(start.saturating_add(allowance.min(u32::MAX as u64) as u32));
        let escape = self.family.escape_squared();
        let [mut zr, mut zi] = self.z[i];
        let [mut sr, mut si] = self.saved[i];
        let mut k = start;
        loop {
            let (zr2, zi2) = (zr * zr, zi * zi);
            if zr2 + zi2 > escape {
                self.smooth[i] = smooth_count(c, [zr, zi], k);
                break;
            }
            if k >= stop {
                break;
            }
            if k > 0 && (zr - sr).abs() + (zi - si).abs() < self.tolerance {
                self.smooth[i] = SETTLED;
                break;
            }
            if k.is_power_of_two() {
                (sr, si) = (zr, zi);
            }
            zi = 2.0 * zr * zi + c[1];
            zr = zr2 - zi2 + c[0];
            k += 1;
        }
        self.z[i] = [zr, zi];
        self.saved[i] = [sr, si];
        self.steps[i] = k;
        let finished = !self.smooth[i].is_nan() || k >= self.limit;
        // A pixel already at the limit only needed checking: charge it as such, not as a
        // pixel's full overhead, or a pass over thousands of them eats the frame's budget.
        let cost = if k == start {
            1
        } else {
            (k - start) as u64 + PIXEL_COST
        };
        (cost, finished)
    }

    /// [`Picture::advance`] for a deep picture: the pixel's `z` holds dz, its difference from
    /// the reference orbit at its place along that orbit (see [`Reference`]).
    fn advance_perturbed(&mut self, i: usize, allowance: u64) -> (u64, bool) {
        let (x, y) = (i % self.view.width, i / self.view.width);
        let dc = self.view.offset_from_reference(x, y);
        if self.smooth[i] == FRESH {
            if known_inside_dd(self.view.point_dd(x, y)) {
                self.smooth[i] = INSIDE;
                return (1, true);
            }
            self.smooth[i] = f32::NAN;
        }
        if !self.smooth[i].is_nan() {
            return (1, true);
        }
        let reference = self.reference.as_ref().expect("a deep picture");
        if reference.cycle_from.is_some() {
            return self.advance_cycling(i, dc, allowance);
        }
        let orbit = &reference.orbit;
        let last = orbit.len() - 1;
        let start = self.steps[i];
        let stop = self
            .limit
            .min(start.saturating_add(allowance.min(u32::MAX as u64) as u32));
        let [mut dr, mut di] = self.z[i];
        let mut m = self.places[i] as usize;
        let mut k = start;
        loop {
            let [zr0, zi0] = orbit[m];
            let (zr, zi) = (zr0 + dr, zi0 + di);
            let size = zr * zr + zi * zi;
            if size > 4.0 {
                let c = self.view.point(x, y); // only to finish the smooth count
                self.smooth[i] = smooth_count(c, [zr, zi], k);
                break;
            }
            if k >= stop {
                break;
            }
            // While the reference hasn't escaped, it reaches the limit, and m <= k < limit;
            // so m + 1 is always there, once an escaped reference's end is rebased away.
            // Selects, not a branch: once an orbit has drifted from the reference this flips
            // unpredictably, and a mispredicted branch every few steps cost about 15%. After
            // a rebase the reference value is Z_0 = 0, so no second load waits on the choice.
            let rebase = (size < dr * dr + di * di) | (reference.escaped & (m == last));
            dr = if rebase { zr } else { dr };
            di = if rebase { zi } else { di };
            m = if rebase { 0 } else { m };
            let (ar, ai) = if rebase { (0.0, 0.0) } else { (zr0, zi0) };
            let (tr, ti) = (2.0 * ar + dr, 2.0 * ai + di);
            (dr, di) = (tr * dr - ti * di + dc[0], tr * di + ti * dr + dc[1]);
            m += 1;
            k += 1;
        }
        self.z[i] = [dr, di];
        self.places[i] = m as u32;
        self.steps[i] = k;
        let finished = !self.smooth[i].is_nan() || k >= self.limit;
        let cost = if k == start {
            1
        } else {
            (k - start) as u64 + PIXEL_COST
        };
        (cost, finished)
    }

    /// [`Picture::advance_perturbed`] when the reference has settled into a cycle: as there,
    /// plus a check that stops pixels settling too. Inside a small copy of the set, the
    /// case a cycling reference usually means, most pixels stop long before the limit
    /// (the timing test's deep view, mostly inside: 32 s down to 3.6 s natively).
    ///
    /// A deep pixel can't compare its z with a saved one in f64, which is far too coarse
    /// there. But at the same place on the reference orbit as when its dz was saved, the
    /// reference values are the same f64s and cancel exactly, so z - z_saved is exactly
    /// dz - dz_saved. The wrap round the reference's cycle makes the places repeat, so such
    /// steps come. And the check runs only while dz is small enough for f64 to resolve it
    /// far below the tolerance (`DZ_RESOLVED`): otherwise two different values could round
    /// alike, so the check is skipped, which is slower but never wrong.
    fn advance_cycling(&mut self, i: usize, dc: [f64; 2], allowance: u64) -> (u64, bool) {
        let (x, y) = (i % self.view.width, i / self.view.width);
        let reference = self.reference.as_ref().expect("a deep picture");
        let orbit = &reference.orbit;
        let last = orbit.len() - 1;
        let cycle_from = reference.cycle_from.expect("a cycling reference");
        let tolerance = self.tolerance;
        let start = self.steps[i];
        let stop = self
            .limit
            .min(start.saturating_add(allowance.min(u32::MAX as u64) as u32));
        let [mut dr, mut di] = self.z[i];
        let [mut sr, mut si] = self.saved[i];
        let mut sm = self.saved_places[i] as usize;
        let mut m = self.places[i] as usize;
        let mut k = start;
        loop {
            let [zr0, zi0] = orbit[m];
            let (zr, zi) = (zr0 + dr, zi0 + di);
            let size = zr * zr + zi * zi;
            if size > 4.0 {
                let c = self.view.point(x, y); // only to finish the smooth count
                self.smooth[i] = smooth_count(c, [zr, zi], k);
                break;
            }
            if k >= stop {
                break;
            }
            if k > 0
                && m == sm
                && dr.abs() + di.abs() < tolerance * DZ_RESOLVED
                && (dr - sr).abs() + (di - si).abs() < tolerance
            {
                self.smooth[i] = SETTLED;
                break;
            }
            if k.is_power_of_two() {
                (sr, si, sm) = (dr, di, m);
            }
            // The end of the orbit matches where its cycle began (to a thousandth of a
            // pixel), so carry on from there; then m + 1 is always there.
            if m == last {
                m = cycle_from;
            }
            let rebase = size < dr * dr + di * di;
            dr = if rebase { zr } else { dr };
            di = if rebase { zi } else { di };
            m = if rebase { 0 } else { m };
            let (ar, ai) = if rebase { (0.0, 0.0) } else { (zr0, zi0) };
            let (tr, ti) = (2.0 * ar + dr, 2.0 * ai + di);
            (dr, di) = (tr * dr - ti * di + dc[0], tr * di + ti * dr + dc[1]);
            m += 1;
            k += 1;
        }
        self.z[i] = [dr, di];
        self.places[i] = m as u32;
        self.saved[i] = [sr, si];
        self.saved_places[i] = sm as u32;
        self.steps[i] = k;
        let finished = !self.smooth[i].is_nan() || k >= self.limit;
        let cost = if k == start {
            1
        } else {
            (k - start) as u64 + PIXEL_COST
        };
        (cost, finished)
    }

    /// Move the picture `dx` pixels right and `dy` down, keeping every pixel still in view
    /// as it is. Only the strips uncovered at the edges start afresh.
    pub fn pan(&mut self, dx: i64, dy: i64) {
        if dx == 0 && dy == 0 {
            return;
        }
        let (w, h) = (self.view.width, self.view.height);
        shift(&mut self.z, w, h, dx, dy, [0.0; 2]);
        shift(&mut self.steps, w, h, dx, dy, 0);
        shift(&mut self.smooth, w, h, dx, dy, FRESH);
        if !self.places.is_empty() {
            shift(&mut self.places, w, h, dx, dy, 0);
            shift(&mut self.saved_places, w, h, dx, dy, 0);
        }
        shift(&mut self.saved, w, h, dx, dy, [0.0; 2]);
        self.view = self.view.panned(dx, dy);
        self.rebuild_queue();
    }

    /// Queue every pixel not yet finished, coarse ones first, and start a new pass.
    ///
    /// Fresh pixels go first. After a pan they are only the strip just uncovered, while the
    /// pixels kept from before are mostly at the limit already; in a view that is mostly
    /// inside the set, there are a great many of those, and putting them first left the new
    /// strip blank until the drag stopped.
    fn rebuild_queue(&mut self) {
        let (w, h) = (self.view.width, self.view.height);
        self.queue.clear();
        for fresh in [true, false] {
            for level in 0..3 {
                for y in 0..h {
                    for x in 0..w {
                        let i = y * w + x;
                        let s = self.smooth[i];
                        if (s == FRESH) == fresh
                            && (s == FRESH || s.is_nan())
                            && coarseness(x, y) == level
                        {
                            self.queue.push(i as u32);
                        }
                    }
                }
            }
        }
        self.read = 0;
        self.write = 0;
        self.pass_limit = self.limit;
    }
}

/// 0 for every fourth pixel each way, 1 for the rest of every second, 2 for the others: the
/// order in which a new picture is filled in.
fn coarseness(x: usize, y: usize) -> u8 {
    if x.is_multiple_of(4) && y.is_multiple_of(4) {
        0
    } else if x.is_multiple_of(2) && y.is_multiple_of(2) {
        1
    } else {
        2
    }
}

/// Whether `c` is in the main cardioid or the period-2 disc (centre -1, radius 1/4). Both
/// are known to lie inside the set, and together they are most of it, so the pixels there
/// are spared iterating to the limit, where they would never escape.
pub fn known_inside(c: [f64; 2]) -> bool {
    let (x, y) = (c[0], c[1]);
    let q = (x - 0.25) * (x - 0.25) + y * y;
    let cardioid = q * (q + (x - 0.25)) <= 0.25 * y * y;
    let disc = (x + 1.0) * (x + 1.0) + y * y <= 1.0 / 16.0;
    cardioid || disc
}

/// The work of starting and finishing a pixel (its point, the queue, the smooth count),
/// counted in iterations: measured at about 8 (36 ns against 4.6 ns an iteration, natively).
const PIXEL_COST: u64 = 8;

/// The path of 0 under z -> z² + c, for showing a reader why a point escapes or stays.
pub struct Orbit {
    /// z₀, z₁, z₂, ...: up to the first point past the escape radius, or `steps` steps.
    pub points: Vec<[f64; 2]>,
    pub fate: Fate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    /// |z| first passed the escape radius (2, for the Mandelbrot set) at this step.
    Escapes(u32),
    /// z has settled into a cycle of this many points (1: a single fixed point).
    Cycle(u32),
    /// Neither, after this many steps: still settling, or wandering for ever.
    Undecided(u32),
}

/// The longest cycle looked for.
const LONGEST_CYCLE: usize = 64;

/// Follow `z0` under z -> z² + c for at most `steps` steps, and say what became of it. For
/// the Mandelbrot set z0 is 0; for a Julia set, the point in question.
///
/// A point inside the Mandelbrot set usually settles into a cycle: in the main cardioid onto
/// a single point, in the disc to its left onto two, in other bulbs onto more. The cycle is
/// found by looking back from the last point for the same point repeated, twice over, to
/// within a tiny distance. Points near the edge settle slowly, and may not show a cycle in
/// time.
pub fn orbit(z0: [f64; 2], c: [f64; 2], steps: u32) -> Orbit {
    let escape = Family::Julia(c).escape_squared();
    let mut points = vec![z0];
    let mut z = z0;
    for n in 0..=steps {
        if z[0] * z[0] + z[1] * z[1] > escape {
            return Orbit {
                points,
                fate: Fate::Escapes(n),
            };
        }
        if n == steps {
            break;
        }
        z = [z[0] * z[0] - z[1] * z[1] + c[0], 2.0 * z[0] * z[1] + c[1]];
        points.push(z);
    }
    let close = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-9;
    let last = points.len() - 1;
    let fate = (1..=LONGEST_CYCLE)
        .find(|&p| {
            last >= 2 * p
                && close(points[last], points[last - p])
                && close(points[last - p], points[last - 2 * p])
        })
        .map_or(Fate::Undecided(steps), |p| Fate::Cycle(p as u32));
    Orbit { points, fate }
}

/// [`known_inside`] in double-double, for deep views: there, rounding c to f64 can move it
/// across the cardioid's edge, which would mark whole stretches of pixels inside wrongly.
fn known_inside_dd(c: [Dd; 2]) -> bool {
    let (x, y) = (c[0], c[1]);
    let a = x - Dd::new(0.25);
    let y2 = y * y;
    let q = a * a + y2;
    let cardioid = (q * (q + a) - y2.scale(0.25)).to_f64() <= 0.0;
    let b = x + Dd::new(1.0);
    let disc = (b * b + y2 - Dd::new(1.0 / 16.0)).to_f64() <= 0.0;
    cardioid || disc
}

/// A deep pixel's cycle check needs its dz no bigger than this many times the tolerance:
/// f64 holds dz to about 1e-16 of itself, and a few thousand steps can grow that error a
/// thousandfold, so this keeps the error a hundredth of the tolerance or less.
const DZ_RESOLVED: f64 = 1e10;

/// The escape radius for smoothing: far enough that z has settled into squaring.
const SMOOTH_RADIUS_SQUARED: f64 = 4_294_967_296.0; // 65,536²

/// A fractional escape count that varies continuously with c, for colouring without bands.
///
/// `z` escaped (first passed 2) at `step`. Iterate on until |z| is huge, k steps in all;
/// from then on each step squares |z|, so log2(log2 |z|) grows by exactly 1 a step, and
/// k - log2(log2 |z|) is the same whichever k it is read at. For a point that escapes with
/// |z| just over 2 it is about `step`; for one that escapes with |z| near 4, about
/// `step` - 1 (only about: the + c of the first few steps shifts it a little either way).
/// It stays within 2 of `step` for points within 2 of 0, and it never jumps.
pub fn smooth_count(c: [f64; 2], z: [f64; 2], step: u32) -> f32 {
    let [mut zr, mut zi] = z;
    let mut k = step as f64;
    for _ in 0..64 {
        let (zr2, zi2) = (zr * zr, zi * zi);
        if zr2 + zi2 > SMOOTH_RADIUS_SQUARED {
            break;
        }
        zi = 2.0 * zr * zi + c[1];
        zr = zr2 - zi2 + c[0];
        k += 1.0;
    }
    let log2_modulus = 0.5 * (zr * zr + zi * zi).log2();
    (k - log2_modulus.log2()).max(0.0) as f32
}

/// Move a `w` x `h` array of values `dx` right and `dy` down, filling what is uncovered.
fn shift<T: Copy>(v: &mut [T], w: usize, h: usize, dx: i64, dy: i64, fill: T) {
    let (wi, hi) = (w as i64, h as i64);
    if dx.abs() >= wi || dy.abs() >= hi {
        v.fill(fill);
        return;
    }
    // Copy rows so that no source row is overwritten before it is read.
    let rows: Box<dyn Iterator<Item = i64>> = if dy > 0 {
        Box::new((0..hi).rev())
    } else {
        Box::new(0..hi)
    };
    let kept = (wi - dx.abs()) as usize;
    let (from, to) = if dx >= 0 {
        (0, dx as usize)
    } else {
        ((-dx) as usize, 0)
    };
    for y in rows {
        let row = y as usize * w;
        let source = y - dy;
        if (0..hi).contains(&source) {
            let src = source as usize * w + from;
            v.copy_within(src..src + kept, row + to);
            let gap = if dx >= 0 { 0..to } else { kept..w };
            v[row + gap.start..row + gap.end].fill(fill);
        } else {
            v[row..row + w].fill(fill);
        }
    }
}

/// The orbit of 0 under z -> z² + c, as a plain loop: the escape step, or `None` if |z|
/// stays within 2 for `limit` steps. The reference the tests check [`Picture`] against.
#[cfg(test)]
fn escape_step(c: [f64; 2], limit: u32) -> Option<u32> {
    let [mut zr, mut zi] = [0.0f64, 0.0];
    for n in 0..=limit {
        if zr * zr + zi * zi > 4.0 {
            return Some(n);
        }
        (zr, zi) = (zr * zr - zi * zi + c[0], 2.0 * zr * zi + c[1]);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home(w: usize, h: usize) -> Viewport {
        Viewport::fit(w, h, [-0.75, 0.0], [3.2, 2.6])
    }

    fn finished(view: Viewport, limit: u32) -> Picture {
        let mut p = Picture::new(Family::Mandelbrot, view, limit);
        p.work(u64::MAX);
        assert!(p.is_done());
        p
    }

    /// Every pixel, as bits, so two pictures can be compared exactly.
    fn state(p: &Picture) -> Vec<(u32, u32, u64, u64)> {
        (0..p.z.len())
            .map(|i| {
                let fresh_or_bits = p.smooth[i].to_bits();
                (
                    p.steps[i],
                    fresh_or_bits,
                    p.z[i][0].to_bits(),
                    p.z[i][1].to_bits(),
                )
            })
            .collect()
    }

    /// Orbits worked by hand. c = 1: 0, 1, 2, 5, so |z| first passes 2 at step 3 (2 itself
    /// does not count). c = 2: 0, 2, 6. c = 3: 0, 3. c = -2: 0, -2, 2, 2, ... stays at 2
    /// forever. c = i: 0, i, -1 + i, -i, -1 + i, ... a cycle. c = 0 and c = -1 (0, -1, 0, -1)
    /// stay put.
    #[test]
    fn hand_traced_orbits() {
        assert_eq!(escape_step([1.0, 0.0], 100), Some(3));
        assert_eq!(escape_step([2.0, 0.0], 100), Some(2));
        assert_eq!(escape_step([3.0, 0.0], 100), Some(1));
        for c in [[-2.0, 0.0], [0.0, 1.0], [0.0, 0.0], [-1.0, 0.0]] {
            assert_eq!(escape_step(c, 10_000), None, "{c:?}");
        }
        // With limit 2, c = 1 is not yet known to escape: |z_2| = 2.
        assert_eq!(escape_step([1.0, 0.0], 2), None);
    }

    /// The picture must give exactly the plain loop's answer for every pixel.
    #[test]
    fn picture_matches_the_plain_loop() {
        let view = home(64, 48);
        let p = finished(view, 200);
        for y in 0..48 {
            for x in 0..64 {
                let expect = escape_step(view.point(x, y), 200);
                match p.pixel(x, y) {
                    Pixel::Escaped { step, .. } => assert_eq!(Some(step), expect),
                    Pixel::Running(n) => assert_eq!((n, expect), (200, None)),
                    Pixel::Inside | Pixel::Settled(_) => assert_eq!(expect, None),
                    Pixel::Fresh => panic!("pixel ({x}, {y}) never computed"),
                }
            }
        }
    }

    /// Raising the limit one step at a time, with the work cut into small, uneven slices,
    /// must leave every pixel exactly as computing straight to the final limit does: this
    /// is what the animation relies on.
    #[test]
    fn step_by_step_equals_all_at_once() {
        let view = home(50, 40);
        let target = finished(view, 120);
        let mut p = Picture::new(Family::Mandelbrot, view, 0);
        let mut budget = 1;
        for limit in 0..=120 {
            p.raise_limit(limit);
            while !p.is_done() {
                p.work(budget);
                budget = budget * 7 % 1_013 + 1;
            }
        }
        assert!(state(&p) == state(&target));
    }

    /// Moving the view by whole pixels keeps the pixels still in view, and must leave the
    /// picture exactly as a fresh computation of the moved view, including a move made
    /// partway through a pass.
    #[test]
    fn panning_keeps_work_and_matches_a_fresh_picture() {
        let view = home(60, 40);
        let mut p = finished(view, 80);
        for (dx, dy) in [(7, 0), (0, -5), (-13, 9), (100, 0), (3, 3)] {
            p.pan(dx, dy);
            p.work(500); // partway
            let fresh = finished(*p.view(), 80);
            p.work(u64::MAX);
            assert!(state(&p) == state(&fresh), "after panning ({dx}, {dy})");
        }
    }

    /// After a pan, the strip just uncovered must be computed before the pixels kept from
    /// before, or in a mostly-inside view (thousands of pixels running at the limit) the
    /// strip stayed blank for the whole drag. So the fresh pixels lead the queue, and a
    /// small budget fills them in first.
    #[test]
    fn panning_computes_the_new_strip_first() {
        // Inside the period-3 bulb above the main cardioid: nearly all pixels run to the
        // limit without escaping, and the shortcut doesn't cover them.
        let view = Viewport::fit(80, 60, [-0.1226, 0.7449], [0.04, 0.03]);
        let mut p = finished(view, 300);
        p.pan(4, 0);
        let fresh = p.smooth.iter().filter(|&&s| s == FRESH).count();
        assert_eq!(fresh, 4 * 60);
        assert!(
            p.queue[..fresh]
                .iter()
                .all(|&i| p.smooth[i as usize] == FRESH)
        );
        // Enough budget for the strip alone fills it.
        p.work(fresh as u64 * (300 + PIXEL_COST));
        assert!(p.smooth.iter().all(|&s| s != FRESH), "strip still blank");
    }

    /// The plain loop in double-double, for checking deep pictures: slow, but exact enough
    /// for any view this demo can show.
    fn escape_step_dd(c: [Dd; 2], limit: u32) -> Option<u32> {
        let (mut zr, mut zi) = (Dd::ZERO, Dd::ZERO);
        for n in 0..=limit {
            let (r, i) = (zr.to_f64(), zi.to_f64());
            if r * r + i * i > 4.0 {
                return Some(n);
            }
            (zr, zi) = (zr * zr - zi * zi + c[0], (zr * zi).scale(2.0) + c[1]);
        }
        None
    }

    /// A deep view: `factor` times the home view's magnification, centred on `centre` plus a
    /// nudge too small for an f64 near it.
    fn deep(w: usize, h: usize, centre: [f64; 2], nudge: [f64; 2], factor: f64) -> Viewport {
        let v = Viewport::fit(w, h, centre, [3.2, 2.6]).with_deep(true);
        let v = v.zoomed(w as f64 / 2.0, h as f64 / 2.0, factor);
        Viewport {
            base: [v.base[0] + Dd::new(nudge[0]), v.base[1] + Dd::new(nudge[1])],
            ..v
        }
    }

    /// Deep pictures, computed by perturbation from one reference, must agree with the
    /// plain loop in double-double, pixel by pixel: at a zoom f64 could still manage, and
    /// far past it, around a point of the boundary (c = i; nudged off it at x1e20, so the
    /// reference escapes and its pixels must rebase off its end), and at the antenna's tip,
    /// c = -2.
    /// Near the boundary, rounding can shift a chaotic orbit's escape by a step or two, so
    /// a few pixels may differ; most must match exactly.
    #[test]
    fn deep_pictures_match_the_plain_loop_in_double_double() {
        let (w, h, limit) = (40, 30, 2_000);
        let cases = [
            ("c = i, x1e12", deep(w, h, [0.0, 1.0], [0.0, 0.0], 1e12)),
            (
                "c = i, x1e20",
                deep(w, h, [0.0, 1.0], [3e-21, -2e-21], 1e20),
            ),
            (
                "c = -2, x1e18",
                deep(w, h, [-2.0, 0.0], [1e-19, 3e-20], 1e18),
            ),
        ];
        for (what, view) in cases {
            assert!(view.perturbed(), "{what} should be deep");
            let mut p = Picture::new(Family::Mandelbrot, view, limit);
            p.work(u64::MAX);
            let escaped = p.reference.as_ref().is_some_and(|r| r.escaped);
            if what == "c = i, x1e20" {
                // Nudged just off the boundary, its reference escapes: this is the case
                // where pixels must rebase off the end of the reference.
                assert!(escaped, "{what}: meant to test a reference that escapes");
            }
            let (mut exact, mut near, mut total) = (0, 0, 0);
            let mut steps = std::collections::BTreeSet::new();
            for y in 0..h {
                for x in 0..w {
                    let expect = escape_step_dd(view.point_dd(x, y), limit);
                    let got = match p.pixel(x, y) {
                        Pixel::Escaped { step, .. } => Some(step),
                        Pixel::Running(_) | Pixel::Inside | Pixel::Settled(_) => None,
                        Pixel::Fresh => panic!("{what}: ({x}, {y}) never computed"),
                    };
                    steps.insert(got);
                    total += 1;
                    match (got, expect) {
                        (a, b) if a == b => exact += 1,
                        (Some(a), Some(b)) if a.abs_diff(b) <= 2 => near += 1,
                        _ => {}
                    }
                }
            }
            eprintln!(
                "{what}: {exact} exact, {near} within 2, of {total}; {} distinct; reference escaped: {escaped}",
                steps.len()
            );
            // At the tip, the reference's orbit lands on z = 2, a repelling fixed point lying
            // exactly on the escape circle, so pixels there are decided by rounding at the
            // boundary, which the two methods do differently; neither is the truer.
            let needed = if what.starts_with("c = -2") { 90 } else { 97 };
            assert!(
                exact * 100 >= total * needed,
                "{what}: only {exact} of {total} exact"
            );
            assert_eq!(exact + near, total, "{what}: some pixels far off");
            assert!(steps.len() > 3, "{what}: a flat picture tests nothing");
        }
    }

    /// The deep path must keep the shallow path's exactness: raising the limit step by step,
    /// in uneven slices, and panning, must leave every pixel as computing in one go does.
    #[test]
    fn deep_step_by_step_and_panned_equal_all_at_once() {
        let view = deep(36, 28, [0.0, 1.0], [3e-21, -2e-21], 1e20);
        let mut target = Picture::new(Family::Mandelbrot, view, 300);
        target.work(u64::MAX);
        let mut p = Picture::new(Family::Mandelbrot, view, 0);
        let mut budget = 1;
        for limit in 0..=300 {
            p.raise_limit(limit);
            while !p.is_done() {
                p.work(budget);
                budget = budget * 7 % 997 + 1;
            }
        }
        assert!(state(&p) == state(&target) && p.places == target.places);
        p.pan(5, -3);
        p.work(700); // partway
        let mut fresh = Picture::new(Family::Mandelbrot, *p.view(), 300);
        fresh.work(u64::MAX);
        p.work(u64::MAX);
        assert!(state(&p) == state(&fresh) && p.places == fresh.places);
    }

    /// A deep view may zoom far past f64's limit, to about 10^28, and still tell
    /// neighbouring pixels apart; a view that isn't deep still stops near 10^12.
    #[test]
    fn deep_views_zoom_past_f64() {
        let mut v = home(300, 200).with_deep(true);
        for _ in 0..40 {
            v = v.zoomed(150.0, 100.0, 10.0);
        }
        assert!(v.at_precision_limit());
        let magnification = home(300, 200).step() / v.step();
        assert!(magnification > 1e26, "only x{magnification:e}");
        let (a, b) = (v.point_dd(0, 0), v.point_dd(1, 0));
        assert!((b[0] - a[0]).to_f64() > 0.0);
        assert!(!home(300, 200).with_deep(true).perturbed());
    }

    /// Drawing an older picture under a newer one maps one view's pixels into another's.
    /// Deep, that must be done relative to the views' bases, or f64 rounding of the points
    /// would put it hundreds of pixels off.
    #[test]
    fn pixel_of_view_is_exact_deep() {
        let a = deep(300, 200, [0.0, 1.0], [3e-21, 0.0], 1e20);
        let b = a.zoomed(70.0, 40.0, 2.0);
        let [x, y] = b.pixel_of_view(&a, 70.0, 40.0);
        assert!(
            (x - 70.0).abs() < 1e-3 && (y - 40.0).abs() < 1e-3,
            "({x}, {y})"
        );
        let shallow = home(300, 200);
        let moved = shallow.panned(5, -3);
        assert_eq!(moved.pixel_of_view(&shallow, 10.0, 20.0), [15.0, 17.0]);
    }

    /// The centre of a small copy of the set of this period on the real axis near -2, by
    /// Newton's method from just right of -2.
    fn copy_centre(period: u32) -> f64 {
        let mut c = -1.99999f64;
        for _ in 0..100 {
            let (mut z, mut dz) = (0.0f64, 0.0f64);
            for _ in 0..period {
                dz = 2.0 * z * dz + 1.0;
                z = z * z + c;
            }
            c -= z / dz;
        }
        c
    }

    /// A pixel stopped as settled must really be inside: the plain loop must not see it
    /// escape even with thirty times the steps. Checked where it is hardest, near the edge of
    /// the set and its small copies, at several zooms; and the stopping must pay: on the
    /// spike at x8,103, where every pixel is inside, most must stop early.
    #[test]
    fn settled_pixels_never_escape() {
        let views = [
            ("home", home(160, 130), 1_000),
            (
                "seahorse x8103",
                Viewport::fit(120, 90, [-0.7453, 0.1127], [3.2, 2.6]).zoomed(60.0, 45.0, 8103.0),
                2_000,
            ),
            (
                "spike x8103",
                Viewport::fit(120, 90, [-1.7549, 0.0], [3.2, 2.6]).zoomed(60.0, 45.0, 8103.0),
                2_000,
            ),
            (
                "period-3 bulb edge x500",
                Viewport::fit(120, 90, [-0.1226, 0.7449], [3.2, 2.6]).zoomed(60.0, 45.0, 500.0),
                2_000,
            ),
            (
                "cardioid edge x1e6",
                Viewport::fit(120, 90, [0.2501, 0.0], [3.2, 2.6]).zoomed(60.0, 45.0, 1e6),
                2_000,
            ),
            (
                "period-8 copy x1e8",
                Viewport::fit(100, 80, [copy_centre(8), 0.0], [3.2, 2.6]).zoomed(50.0, 40.0, 1e8),
                3_000,
            ),
            // Where a fixed tolerance of 1e-10 got pixels wrong.
            (
                "period-10 copy x1e9",
                Viewport::fit(100, 80, [copy_centre(10), 0.0], [3.2, 2.6]).zoomed(50.0, 40.0, 1e9),
                3_000,
            ),
            (
                "period-10 copy x3e9",
                Viewport::fit(100, 80, [copy_centre(10), 0.0], [3.2, 2.6]).zoomed(50.0, 40.0, 3e9),
                3_000,
            ),
        ];
        for (what, view, limit) in views {
            let mut p = Picture::new(Family::Mandelbrot, view, limit);
            p.work(u64::MAX);
            let (mut settled, mut early) = (0, 0);
            for y in 0..view.height {
                for x in 0..view.width {
                    if let Pixel::Settled(k) = p.pixel(x, y) {
                        settled += 1;
                        early += usize::from(k < limit / 2);
                        assert_eq!(
                            escape_step(view.point(x, y), 30 * limit),
                            None,
                            "{what}: ({x}, {y}) settled at step {k} but escapes"
                        );
                    }
                }
            }
            eprintln!("{what}: {settled} settled, {early} before half the limit");
            if what.starts_with("spike") {
                assert!(
                    early * 2 > view.width * view.height,
                    "{what}: too few stop early"
                );
            }
        }
    }

    /// The pixels the shortcut skips must be ones plain iteration never sees escape.
    #[test]
    fn cardioid_and_disc_are_inside() {
        let view = home(160, 130);
        let mut skipped = 0;
        for y in 0..130 {
            for x in 0..160 {
                let c = view.point(x, y);
                if known_inside(c) {
                    skipped += 1;
                    assert_eq!(escape_step(c, 5_000), None, "{c:?}");
                }
            }
        }
        assert!(skipped > 1_000, "the shortcut should cover much of the set");
        assert!(known_inside([0.0, 0.0]) && known_inside([-1.0, 0.0]));
        assert!(!known_inside([0.26, 0.0]) && !known_inside([-1.26, 0.0]));
    }

    /// The set is its own mirror image in the real axis (c and its conjugate have conjugate
    /// orbits), so a picture centred on the axis must be exactly symmetric top to bottom.
    #[test]
    fn mirror_symmetric_about_the_real_axis() {
        let (w, h) = (40, 30);
        let p = finished(home(w, h), 150);
        for y in 0..h {
            for x in 0..w {
                assert_eq!(p.pixel(x, y), p.pixel(x, h - 1 - y), "({x}, {y})");
            }
        }
    }

    /// The smooth count must change gently from point to point, where the whole-number count
    /// jumps, and stay within 2 of the whole-number count.
    #[test]
    fn smooth_count_is_continuous() {
        let mut last: Option<f32> = None;
        let mut jumps = 0;
        let mut last_step = 0;
        for i in 0..15_000 {
            let c = [0.5 + i as f64 * 1e-4, 0.3];
            let step = escape_step(c, 1_000).expect("these points are all outside");
            let mut z = [0.0f64, 0.0];
            for _ in 0..step {
                z = [z[0] * z[0] - z[1] * z[1] + c[0], 2.0 * z[0] * z[1] + c[1]];
            }
            let s = smooth_count(c, z, step);
            assert!((s - step as f32).abs() < 2.0, "{c:?}: {s} against {step}");
            if let Some(prev) = last {
                assert!((s - prev).abs() < 0.01, "jump at {c:?}: {prev} to {s}");
            }
            jumps += usize::from(step != last_step);
            (last, last_step) = (Some(s), step);
        }
        assert!(
            jumps > 3,
            "the whole-number count should change along this line"
        );
    }

    /// Orbits with known fates. c = 1 escapes at step 3 (0, 1, 2, 5). c = 0 stays at 0; c = -2
    /// lands on 2 and stays (0, -2, 2, 2, ...). c = -1 and c = i fall into cycles of two
    /// points (0, -1 and -1 + i, -i). The real period-3 bulb's centre, -1.7549, and the
    /// "rabbit", -0.1226 + 0.7449i, cycle through three points. Along the real axis the
    /// cycles double on the way to the Feigenbaum point: 1 at -0.5, 2 at -1, 4 at -1.3.
    #[test]
    fn orbit_fates() {
        let o = orbit([0.0, 0.0], [1.0, 0.0], 100);
        assert_eq!(o.fate, Fate::Escapes(3));
        assert_eq!(
            o.points,
            vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [5.0, 0.0]]
        );
        let fate = |c| orbit([0.0, 0.0], c, 5_000).fate;
        assert_eq!(fate([0.0, 0.0]), Fate::Cycle(1));
        assert_eq!(fate([-2.0, 0.0]), Fate::Cycle(1));
        assert_eq!(fate([-1.0, 0.0]), Fate::Cycle(2));
        assert_eq!(fate([0.0, 1.0]), Fate::Cycle(2));
        assert_eq!(fate([-1.7549, 0.0]), Fate::Cycle(3));
        assert_eq!(fate([-0.1226, 0.7449]), Fate::Cycle(3));
        assert_eq!(fate([-0.5, 0.0]), Fate::Cycle(1));
        assert_eq!(fate([-1.3, 0.0]), Fate::Cycle(4));
        assert_eq!(orbit([0.0, 0.0], [0.0, 0.0], 0).fate, Fate::Undecided(0));
        // The picture and the orbit must agree on when a point escapes.
        for c in [[0.5, 0.5], [-0.75, 0.1], [0.26, 0.0]] {
            let Fate::Escapes(n) = fate(c) else {
                panic!("{c:?} should escape")
            };
            assert_eq!(escape_step(c, 5_000), Some(n));
        }
    }

    /// For c = 0 the rule is plain squaring, which pulls points inside the unit circle in to
    /// 0 and pushes points outside it away, so the filled Julia set is the unit disc.
    #[test]
    fn julia_of_zero_is_the_unit_disc() {
        let view = Viewport::fit(80, 80, [0.0, 0.0], [3.0, 3.0]);
        let mut p = Picture::new(Family::Julia([0.0, 0.0]), view, 300);
        p.work(u64::MAX);
        for y in 0..80 {
            for x in 0..80 {
                let [a, b] = view.point(x, y);
                let r = a.hypot(b);
                match p.pixel(x, y) {
                    Pixel::Running(_) | Pixel::Settled(_) => {
                        assert!(r < 1.01, "({a}, {b}) should escape")
                    }
                    Pixel::Escaped { .. } => assert!(r > 0.99, "({a}, {b}) should stay"),
                    other => panic!("({a}, {b}) is {other:?}"),
                }
            }
        }
        // And the orbit agrees: 0.5 squares down towards 0, 1.5 runs off.
        assert_eq!(orbit([0.5, 0.0], [0.0, 0.0], 100).fate, Fate::Cycle(1));
        assert_eq!(orbit([1.5, 0.0], [0.0, 0.0], 100).fate, Fate::Escapes(1));
        assert_eq!(orbit([3.0, 0.0], [0.0, 0.0], 100).fate, Fate::Escapes(0));
    }

    /// z and -z square to the same thing, so every Julia set is symmetric through 0: a
    /// picture centred on 0 must match itself turned half round, exactly.
    #[test]
    fn julia_sets_are_symmetric_through_zero() {
        let (w, h) = (41, 31);
        let view = Viewport::fit(w, h, [0.0, 0.0], [3.2, 2.4]);
        let mut p = Picture::new(Family::Julia([-0.1226, 0.7449]), view, 200);
        p.work(u64::MAX);
        for y in 0..h {
            for x in 0..w {
                assert_eq!(p.pixel(x, y), p.pixel(w - 1 - x, h - 1 - y), "({x}, {y})");
            }
        }
    }

    /// Julia pictures, like Mandelbrot ones, must come out the same however the work and
    /// the limit are cut up, and after panning.
    #[test]
    fn julia_step_by_step_and_panned_equal_all_at_once() {
        let family = Family::Julia([-0.8, 0.156]);
        let view = Viewport::fit(40, 30, [0.0, 0.0], [3.2, 2.4]);
        let mut target = Picture::new(family, view, 90);
        target.work(u64::MAX);
        let mut p = Picture::new(family, view, 0);
        let mut budget = 3;
        for limit in 0..=90 {
            p.raise_limit(limit);
            while !p.is_done() {
                p.work(budget);
                budget = budget * 5 % 701 + 1;
            }
        }
        assert!(state(&p) == state(&target));
        p.pan(-6, 4);
        p.work(u64::MAX);
        let mut fresh = Picture::new(family, *p.view(), 90);
        fresh.work(u64::MAX);
        assert!(state(&p) == state(&fresh));
    }

    /// The centre of a small copy of the set of this period on the real axis, to
    /// double-double precision: Newton's method from `guess`, z_period(c) = 0.
    fn copy_centre_dd(period: u32, guess: f64) -> Dd {
        let mut c = Dd::new(guess);
        for _ in 0..60 {
            let (mut z, mut dz) = (Dd::ZERO, Dd::ZERO);
            for _ in 0..period {
                dz = (z * dz).scale(2.0) + Dd::new(1.0);
                z = z * z + c;
            }
            c = c - crate::dd::div(z, dz);
        }
        c
    }

    /// A deep view `factor` times the home view's magnification, centred on `c` (real).
    fn deep_at(w: usize, h: usize, c: Dd, factor: f64) -> Viewport {
        let v = deep(w, h, [c.hi, 0.0], [0.0, 0.0], factor);
        Viewport {
            base: [c, Dd::ZERO],
            ..v
        }
    }

    /// Deep pixels stopped as settled must really be inside: the plain loop in double-double
    /// must not see them escape with ten times the steps. Checked around small copies of the
    /// set found by Newton's method, at zooms where each copy's edge crosses the view, so
    /// stopped and escaping pixels sit side by side; and the stopping must pay where the
    /// view is all inside.
    #[test]
    fn deep_settled_pixels_never_escape() {
        let (w, h, limit) = (60, 45, 2_000);
        let cases = [(12, 1e12), (16, 1e14), (20, 1e16), (24, 1e18)];
        for (period, factor) in cases {
            let view = deep_at(w, h, copy_centre_dd(period, copy_centre(period)), factor);
            assert!(view.perturbed());
            let mut p = Picture::new(Family::Mandelbrot, view, limit);
            p.work(u64::MAX);
            let (mut settled, mut escaped) = (0, 0);
            for y in 0..h {
                for x in 0..w {
                    match p.pixel(x, y) {
                        Pixel::Settled(k) => {
                            settled += 1;
                            assert_eq!(
                                escape_step_dd(view.point_dd(x, y), 10 * limit),
                                None,
                                "period {period} x{factor:e}: ({x}, {y}) settled at {k} but escapes"
                            );
                        }
                        Pixel::Escaped { .. } => escaped += 1,
                        _ => {}
                    }
                }
            }
            eprintln!("period {period} x{factor:e}: {settled} settled, {escaped} escaped");
            assert!(
                settled > 0 && escaped > 0,
                "period {period} x{factor:e}: not an edge"
            );
        }
        // All inside: in the period-3 copy, every pixel stops long before the limit.
        let view = deep_at(w, h, copy_centre_dd(3, -1.7548776662466927), 1e20);
        let mut p = Picture::new(Family::Mandelbrot, view, limit);
        p.work(u64::MAX);
        for y in 0..h {
            for x in 0..w {
                assert!(
                    matches!(p.pixel(x, y), Pixel::Settled(k) if k < 100),
                    "({x}, {y}): {:?}",
                    p.pixel(x, y)
                );
            }
        }
    }

    /// With a reference that cycles (its orbit wraps round), deep pictures must still come
    /// out exactly the same however the limit was raised and the work cut up, and panned.
    #[test]
    fn deep_cycling_reference_step_by_step_and_panned_equal_all_at_once() {
        let view = deep_at(40, 30, copy_centre_dd(12, copy_centre(12)), 1e12);
        let mut target = Picture::new(Family::Mandelbrot, view, 400);
        target.work(u64::MAX);
        assert!(target.reference.as_ref().unwrap().cycle_from.is_some());
        let mut p = Picture::new(Family::Mandelbrot, view, 0);
        let mut budget = 1;
        for limit in 0..=400 {
            p.raise_limit(limit);
            while !p.is_done() {
                p.work(budget);
                budget = budget * 7 % 997 + 1;
            }
        }
        let same = |a: &Picture, b: &Picture| {
            state(a) == state(b)
                && a.places == b.places
                && a.saved_places == b.saved_places
                && a.saved.iter().zip(&b.saved).all(|(x, y)| {
                    x[0].to_bits() == y[0].to_bits() && x[1].to_bits() == y[1].to_bits()
                })
        };
        assert!(same(&p, &target));
        p.pan(4, -3);
        p.work(500);
        let mut fresh = Picture::new(Family::Mandelbrot, *p.view(), 400);
        fresh.work(u64::MAX);
        p.work(u64::MAX);
        assert!(same(&p, &fresh));
    }

    /// The view's two mappings must undo each other, zooming must keep the point under the
    /// pointer where it was, and panning must move the contents by whole pixels.
    #[test]
    fn viewport_mappings() {
        let v = home(300, 200);
        let p = v.at(37.25, 151.5);
        let back = v.pixel_of(p);
        assert!((back[0] - 37.25).abs() < 1e-9 && (back[1] - 151.5).abs() < 1e-9);
        assert_eq!(v.point(10, 20), v.at(10.5, 20.5));
        let z = v.zoomed(37.25, 151.5, 8.0);
        assert!((z.step() - v.step() / 8.0).abs() < 1e-18);
        let q = z.at(37.25, 151.5);
        assert!((q[0] - p[0]).abs() < 1e-12 && (q[1] - p[1]).abs() < 1e-12);
        let moved = v.panned(5, -3);
        assert_eq!(moved.point(15, 17), v.point(10, 20));
        // The imaginary axis points up: the top row is the most positive.
        assert!(v.point(0, 0)[1] > v.point(0, 199)[1]);
    }

    /// Zooming in stops where neighbouring pixels would round to the same point.
    #[test]
    fn zoom_stops_at_f64_precision() {
        let mut v = home(300, 200);
        for _ in 0..200 {
            v = v.zoomed(150.0, 100.0, 10.0);
        }
        assert!(v.at_precision_limit());
        assert_ne!(v.point(0, 0), v.point(1, 0));
        assert!(!home(300, 200).at_precision_limit());
    }

    /// The cost of an iteration, which `ITERATIONS_PER_FRAME` in main.rs is based on: the
    /// home view at about the size of a laptop window's picture, to 500 steps.
    /// `cargo test --release -p mandelbrot timing -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn timing() {
        let views = [
            ("home", home(910, 830), 500),
            (
                "seahorse",
                Viewport::fit(910, 830, [-0.7436, 0.1318], [0.01, 0.01]),
                5_000,
            ),
            // Every pixel inside the set: the case settling into a cycle exists for.
            (
                "spike x8103",
                Viewport::fit(910, 830, [-1.7549, 0.0], [3.2, 2.6]).zoomed(455.0, 415.0, 8103.0),
                5_000,
            ),
            // Perturbation: one double-double reference, every pixel in f64 against it.
            (
                "deep, c = i, x1e20",
                deep(910, 830, [0.0, 1.0], [3e-21, -2e-21], 1e20),
                5_000,
            ),
            // Last: at about 10 s it heats the processor, and a case timed after it ran
            // three times slower.
            // Deep and mostly inside: the period-12 copy near -2 at x1e12, where the deep
            // cycle check (comparing dz at matching places on the reference) pays.
            (
                "deep inside, period 12 x1e12",
                deep_at(910, 830, copy_centre_dd(12, copy_centre(12)), 1e12),
                5_000,
            ),
        ];
        for (name, view, limit) in views {
            let mut p = Picture::new(Family::Mandelbrot, view, limit);
            let start = std::time::Instant::now();
            let mut iterations = 0;
            while !p.is_done() {
                iterations += p.work(5_000_000);
            }
            let took = start.elapsed();
            println!(
                "{name}: {iterations} iterations in {took:.2?}, {:.2} ns each, {:.1} a pixel",
                took.as_nanos() as f64 / iterations as f64,
                iterations as f64 / (view.width * view.height) as f64
            );
        }
    }
}
