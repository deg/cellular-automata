//! One picture on screen with its own view of the plane: the Mandelbrot set, or a Julia set.

use crate::colour::Colours;
use crate::sim::{Family, Picture, Pixel, Viewport};
use crate::view::PlaneView;
use ca_core::egui::{self, Pos2, Rect, Vec2};
use web_time::Instant;

pub struct Pane {
    picture: Picture,
    view: PlaneView,
    /// What Reset view shows: a centre and the span around it to fit.
    home: ([f64; 2], [f64; 2]),
    /// Pixels per point the picture is computed at.
    scale: f32,
    /// Fit the home view on the next frame, once the space for it is known.
    go_home: bool,
    /// Plane units per point in the home view, for the magnification shown.
    home_step: f64,
    /// Fractions of a pixel dragged but not yet moved: the picture moves by whole pixels.
    pan_carry: Vec2,
    /// The picture must be uploaded again.
    redraw: bool,
}

impl Pane {
    pub fn new(name: &'static str, family: Family, centre: [f64; 2], span: [f64; 2]) -> Self {
        let placeholder = Viewport::fit(1, 1, centre, span);
        Self {
            picture: Picture::new(family, placeholder, 0),
            view: PlaneView::new(name),
            home: (centre, span),
            scale: 1.0,
            go_home: true,
            home_step: placeholder.step(),
            pan_carry: Vec2::ZERO,
            redraw: true,
        }
    }

    pub fn picture(&self) -> &Picture {
        &self.picture
    }

    /// Return to the home view on the next frame.
    pub fn go_home(&mut self) {
        self.go_home = true;
    }

    /// Make the home view a different one, and go there.
    pub fn set_home(&mut self, centre: [f64; 2], span: [f64; 2]) {
        if self.home != (centre, span) {
            self.home = (centre, span);
            self.go_home = true;
        }
    }

    /// How much bigger than the home view things look now.
    pub fn magnification(&self) -> f64 {
        self.home_step / (self.picture.view().step() * self.scale as f64)
    }

    /// Make the picture fill `rect` at `scale` pixels per point, keeping its centre and zoom
    /// (or fitting the home view, if asked to).
    pub fn fit(&mut self, rect: Rect, scale: f32) {
        let w = ((rect.width() * scale).floor() as usize).max(1);
        let h = ((rect.height() * scale).floor() as usize).max(1);
        if self.go_home {
            self.go_home = false;
            self.scale = scale;
            let home = Viewport::fit(w, h, self.home.0, self.home.1);
            self.home_step = home.step() * scale as f64;
            self.replace(self.picture.family(), home);
            return;
        }
        let v = *self.picture.view();
        if (v.width, v.height) != (w, h) || scale != self.scale {
            let step = v.step() * self.scale as f64 / scale as f64;
            self.scale = scale;
            self.replace(self.picture.family(), v.resized(w, h, step));
        }
    }

    /// Show a different set in the same view, at the same limit.
    pub fn set_family(&mut self, family: Family) {
        if family != self.picture.family() {
            self.replace(family, *self.picture.view());
        }
    }

    /// Start a new picture at the current limit, keeping the old one as a backdrop if it was
    /// finished (or there is nothing better).
    fn replace(&mut self, family: Family, view: Viewport) {
        if self.picture.is_done() || !self.view.has_backdrop() {
            self.view.keep_as_backdrop();
        }
        self.picture = Picture::new(family, view, self.picture.limit());
        self.redraw = true;
    }

    /// Move the contents by `delta` points.
    pub fn pan(&mut self, delta: Vec2) {
        let moved = delta * self.scale + self.pan_carry;
        let whole = moved.round();
        self.pan_carry = moved - whole;
        self.picture.pan(whole.x as i64, whole.y as i64);
        self.redraw = true;
    }

    /// Magnify by `factor` about `at`, in points from the picture's top-left corner.
    pub fn zoom(&mut self, at: Vec2, factor: f32) {
        let at = at * self.scale;
        let old = *self.picture.view();
        let new = old.zoomed(at.x as f64, at.y as f64, factor as f64);
        if new.step() != old.step() {
            self.replace(self.picture.family(), new);
        }
    }

    /// Back to step 0, in the same view.
    pub fn restart(&mut self) {
        self.restart_at(0);
    }

    /// Start again in the same view at `limit`, which may be lower than the current one.
    pub fn restart_at(&mut self, limit: u32) {
        self.picture = Picture::new(self.picture.family(), *self.picture.view(), limit);
        self.view.drop_backdrop();
        self.redraw = true;
    }

    pub fn raise_limit(&mut self, limit: u32) {
        self.picture.raise_limit(limit);
    }

    /// Recolour on the next upload.
    pub fn recolour(&mut self) {
        self.redraw = true;
    }

    /// Do up to `budget` iterations of work, and upload the picture if it changed. Returns
    /// the work done and the seconds it took. Only the work is timed, not the upload: the
    /// upload's cost is fixed, and counted in would make the work look slower than it is.
    pub fn update(&mut self, ctx: &egui::Context, colours: &Colours, budget: u64) -> (u64, f64) {
        let started = Instant::now();
        let spent = self.picture.work(budget);
        let seconds = started.elapsed().as_secs_f64();
        if spent > 0 || self.redraw {
            self.view.set_picture(ctx, &self.picture, colours);
            self.redraw = false;
        }
        if self.picture.is_done() {
            self.view.drop_backdrop();
        }
        (spent, seconds)
    }

    pub fn paint(&self, painter: &egui::Painter, rect: Rect) {
        self.view
            .paint(painter, rect, self.scale, self.picture.view());
    }

    /// The plane point at `pos`, drawn in `rect`, and what its pixel knows.
    pub fn pixel_at(&self, pos: Pos2, rect: Rect) -> Option<([f64; 2], Pixel)> {
        let at = (pos - rect.min) * self.scale;
        let v = self.picture.view();
        let (x, y) = (at.x.floor() as usize, at.y.floor() as usize);
        (at.x >= 0.0 && at.y >= 0.0 && x < v.width && y < v.height)
            .then(|| (v.point(x, y), self.picture.pixel(x, y)))
    }

    /// Where the plane point `p` is drawn, in `rect`.
    pub fn to_screen(&self, p: [f64; 2], rect: Rect) -> Pos2 {
        let [x, y] = self.picture.view().pixel_of(p);
        rect.min + egui::vec2(x as f32, y as f32) / self.scale
    }
}
