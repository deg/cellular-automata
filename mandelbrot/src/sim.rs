//! The Mandelbrot set, pixel by pixel. Each pixel stands for a point c of the complex plane,
//! and iterates z -> z² + c from z = 0. It records the first step at which |z| passes 2 (from
//! then on |z| grows without bound, so c is outside the set), or that it hasn't yet.
//!
//! Every pixel keeps its z and its step count, so raising the iteration limit carries on
//! from where the last limit stopped rather than starting again. That is what lets the demo
//! animate the limit one step at a time, like a generation of the other demos.

/// A rectangle of the complex plane, cut into `width` x `height` pixels.
///
/// The plane point of a pixel is computed from a whole-pixel offset, so moving the view by a
/// whole number of pixels gives every pixel exactly the point some pixel had before. That is
/// what lets [`Picture::pan`] keep the work already done.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: usize,
    pub height: usize,
    /// The plane point at the picture's centre, before `offset`.
    base: [f64; 2],
    /// Plane units per pixel.
    step: f64,
    /// Whole pixels the view has been moved by since `base` was set.
    offset: [i64; 2],
}

/// The smallest step allowed, relative to the size of the coordinates: below about 16 units
/// in the last place, neighbouring pixels would round to the same f64 point and the picture
/// would turn into blocks.
const PRECISION: f64 = 16.0 * f64::EPSILON;

/// The widest view allowed, in plane units across its shorter side. The whole set fits in a
/// disc of radius 2, so there is nothing to see further out.
const WIDEST: f64 = 12.0;

impl Viewport {
    pub fn new(width: usize, height: usize, centre: [f64; 2], step: f64) -> Self {
        Self {
            width,
            height,
            base: centre,
            step,
            offset: [0, 0],
        }
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
        [self.base[0] + i * self.step, self.base[1] - j * self.step]
    }

    /// The plane point at `(x, y)` in pixels from the picture's top-left corner, fractions
    /// allowed: pixel `(i, j)` covers `i..i + 1` by `j..j + 1`. The plane's imaginary axis
    /// points up, the picture's y down.
    pub fn at(&self, x: f64, y: f64) -> [f64; 2] {
        let i = x + self.offset[0] as f64 - self.width as f64 * 0.5;
        let j = y + self.offset[1] as f64 - self.height as f64 * 0.5;
        [self.base[0] + i * self.step, self.base[1] - j * self.step]
    }

    /// Where the plane point `p` falls, in pixels from the top-left corner: the inverse of
    /// [`Viewport::at`].
    pub fn pixel_of(&self, p: [f64; 2]) -> [f64; 2] {
        let x = (p[0] - self.base[0]) / self.step - self.offset[0] as f64 + self.width as f64 * 0.5;
        let y =
            (self.base[1] - p[1]) / self.step - self.offset[1] as f64 + self.height as f64 * 0.5;
        [x, y]
    }

    /// This view magnified by `factor` about `(x, y)` (in pixels), which stays put on screen.
    /// The magnification stops where f64 runs out of precision, and where the view is wide
    /// enough to show everything.
    pub fn zoomed(&self, x: f64, y: f64, factor: f64) -> Self {
        let q = self.at(x, y);
        let step = self.clamp_step(self.step / factor, q);
        let base = [
            q[0] - (x - self.width as f64 * 0.5) * step,
            q[1] + (y - self.height as f64 * 0.5) * step,
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
        let mut view = Self::new(width, height, self.centre(), step);
        view.step = view.clamp_step(step, view.base);
        view
    }

    /// Whether zooming in further is blocked by f64's precision.
    pub fn at_precision_limit(&self) -> bool {
        self.step <= self.smallest_step(self.centre()) * 1.0001
    }

    fn smallest_step(&self, near: [f64; 2]) -> f64 {
        PRECISION * near[0].abs().max(near[1].abs()).max(1.0)
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
    /// |z| first passed 2 at `step`. `smooth` is a fractional version of the step that
    /// varies continuously from pixel to pixel, for colouring without bands: close to `step`,
    /// see [`smooth_count`].
    Escaped { step: u32, smooth: f32 },
}

// `Picture::smooth` codes: NaN while running, these two for fresh and inside, and otherwise
// the escaped pixel's smooth count (never negative).
const FRESH: f32 = f32::NEG_INFINITY;
const INSIDE: f32 = f32::INFINITY;

/// The Mandelbrot set over a [`Viewport`], computed a budgeted piece at a time.
///
/// Raise the limit with [`Picture::raise_limit`] and call [`Picture::work`] each frame until
/// [`Picture::is_done`]. The work is split between frames by iteration count, not time: the
/// browser has no clock for that (`std::time::Instant` panics in wasm).
pub struct Picture {
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
}

impl Picture {
    pub fn new(view: Viewport, limit: u32) -> Self {
        let n = view.width * view.height;
        let mut picture = Self {
            view,
            limit,
            z: vec![[0.0; 2]; n],
            steps: vec![0; n],
            smooth: vec![FRESH; n],
            queue: Vec::new(),
            read: 0,
            write: 0,
            pass_limit: limit,
        };
        picture.rebuild_queue();
        picture
    }

    pub fn view(&self) -> &Viewport {
        &self.view
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
        let mut spent = 0;
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
    /// (the steps plus the pixel's overhead, or 1 if it needed none) and whether the pixel is finished: escaped, inside, or at the limit.
    fn advance(&mut self, i: usize, allowance: u64) -> (u64, bool) {
        let (x, y) = (i % self.view.width, i / self.view.width);
        let c = self.view.point(x, y);
        if self.smooth[i] == FRESH {
            if known_inside(c) {
                self.smooth[i] = INSIDE;
                return (1, true);
            }
            self.smooth[i] = f32::NAN;
        }
        if !self.smooth[i].is_nan() {
            return (1, true);
        }
        let start = self.steps[i];
        let stop = self
            .limit
            .min(start.saturating_add(allowance.min(u32::MAX as u64) as u32));
        let [mut zr, mut zi] = self.z[i];
        let mut k = start;
        loop {
            let (zr2, zi2) = (zr * zr, zi * zi);
            if zr2 + zi2 > 4.0 {
                self.smooth[i] = smooth_count(c, [zr, zi], k);
                break;
            }
            if k >= stop {
                break;
            }
            zi = 2.0 * zr * zi + c[1];
            zr = zr2 - zi2 + c[0];
            k += 1;
        }
        self.z[i] = [zr, zi];
        self.steps[i] = k;
        let finished = !self.smooth[i].is_nan() || k >= self.limit;
        ((k - start) as u64 + PIXEL_COST, finished)
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
        self.view = self.view.panned(dx, dy);
        self.rebuild_queue();
    }

    /// Queue every pixel not yet finished, coarse ones first, and start a new pass.
    fn rebuild_queue(&mut self) {
        let (w, h) = (self.view.width, self.view.height);
        self.queue.clear();
        for level in 0..3 {
            for y in 0..h {
                for x in 0..w {
                    let i = y * w + x;
                    let open = self.smooth[i] == FRESH || self.smooth[i].is_nan();
                    if open && coarseness(x, y) == level {
                        self.queue.push(i as u32);
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
    /// z₀ = 0, z₁ = c, z₂, ...: up to the first point past 2, or `steps` steps.
    pub points: Vec<[f64; 2]>,
    pub fate: Fate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    /// |z| first passed 2 at this step.
    Escapes(u32),
    /// z has settled into a cycle of this many points (1: a single fixed point).
    Cycle(u32),
    /// Neither, after this many steps: still settling, or wandering for ever.
    Undecided(u32),
}

/// The longest cycle looked for.
const LONGEST_CYCLE: usize = 64;

/// Follow 0 under z -> z² + c for at most `steps` steps, and say what became of it.
///
/// A point inside the set usually settles into a cycle: in the main cardioid onto a single
/// point, in the disc to its left onto two, in other bulbs onto more. The cycle is found by
/// looking back from the last point for the same point repeated, twice over, to within a
/// tiny distance. Points near the edge settle slowly, and may not show a cycle in time.
pub fn orbit(c: [f64; 2], steps: u32) -> Orbit {
    let mut points = vec![[0.0f64, 0.0]];
    let mut z = [0.0f64, 0.0];
    for n in 1..=steps {
        z = [z[0] * z[0] - z[1] * z[1] + c[0], 2.0 * z[0] * z[1] + c[1]];
        points.push(z);
        if z[0] * z[0] + z[1] * z[1] > 4.0 {
            return Orbit {
                points,
                fate: Fate::Escapes(n),
            };
        }
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
        let mut p = Picture::new(view, limit);
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
                    Pixel::Inside => assert_eq!(expect, None),
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
        let mut p = Picture::new(view, 0);
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
        let o = orbit([1.0, 0.0], 100);
        assert_eq!(o.fate, Fate::Escapes(3));
        assert_eq!(
            o.points,
            vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [5.0, 0.0]]
        );
        let fate = |c| orbit(c, 5_000).fate;
        assert_eq!(fate([0.0, 0.0]), Fate::Cycle(1));
        assert_eq!(fate([-2.0, 0.0]), Fate::Cycle(1));
        assert_eq!(fate([-1.0, 0.0]), Fate::Cycle(2));
        assert_eq!(fate([0.0, 1.0]), Fate::Cycle(2));
        assert_eq!(fate([-1.7549, 0.0]), Fate::Cycle(3));
        assert_eq!(fate([-0.1226, 0.7449]), Fate::Cycle(3));
        assert_eq!(fate([-0.5, 0.0]), Fate::Cycle(1));
        assert_eq!(fate([-1.3, 0.0]), Fate::Cycle(4));
        assert_eq!(orbit([0.0, 0.0], 0).fate, Fate::Undecided(0));
        // The picture and the orbit must agree on when a point escapes.
        for c in [[0.5, 0.5], [-0.75, 0.1], [0.26, 0.0]] {
            let Fate::Escapes(n) = fate(c) else {
                panic!("{c:?} should escape")
            };
            assert_eq!(escape_step(c, 5_000), Some(n));
        }
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
        ];
        for (name, view, limit) in views {
            let mut p = Picture::new(view, limit);
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
