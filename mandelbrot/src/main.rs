//! The Mandelbrot set: every point c of the plane iterates z -> z² + c from 0, and is
//! coloured by the step at which |z| passes 2, or drawn as the set if it never does. The
//! iteration limit is raised a step at a time, like the generations of the other demos.

mod colour;
mod sim;
mod view;

use ca_core::egui::{
    self, Color32, Event, MouseWheelSource, MouseWheelUnit, RichText, Sense, Stroke, Vec2,
};
use ca_core::{About, Advance, Pace, Playback, eframe};
use colour::{Colours, Palette};
use sim::{Fate, Orbit, Picture, Pixel, Viewport, orbit};
use view::PlaneView;

fn main() {
    ca_core::run("The Mandelbrot set", MandelbrotApp::new);
}

/// The view Reset returns to: the whole set, a little margin round it.
const HOME_CENTRE: [f64; 2] = [-0.75, 0.0];
const HOME_SPAN: [f64; 2] = [3.2, 2.6];

/// Iterations of z -> z² + c one frame can afford (with each pixel's overhead counted as
/// a few more: see `sim::PIXEL_COST`). Measured natively at about 4.6 ns each (`cargo test
/// --release -p mandelbrot timing -- --ignored --nocapture`), so this is about 11 ms,
/// leaving room in a 16 ms frame for building and drawing the picture. The browser is
/// slower, and there a big job takes more frames.
const ITERATIONS_PER_FRAME: u64 = 2_500_000;

/// The highest limit offered.
const MOST_STEPS: u32 = 1_000_000;

/// The most steps of the pointer's orbit followed (looking for a cycle), and drawn.
const ORBIT_STEPS: u32 = 20_000;
const ORBIT_DRAWN: usize = 1_000;

/// The dots on the cycle an orbit settles into: the site's gold.
const CYCLE: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

struct MandelbrotApp {
    picture: Picture,
    view: PlaneView,
    about: About,
    playback: Playback,
    pace: Pace,
    colours: Colours,
    /// Running stops when the limit gets here.
    stop_at: u32,
    /// One pixel per screen pixel rather than per point: sharper on a high-density screen,
    /// and that much more work.
    sharp: bool,
    /// Pixels per point the picture is computed at.
    scale: f32,
    /// Fit the home view on the next frame, once the space for it is known.
    go_home: bool,
    /// Plane units per point in the home view, for the magnification shown.
    home_step: f64,
    /// Fractions of a pixel dragged but not yet moved: the picture moves by whole pixels.
    pan_carry: Vec2,
    /// The plane point under the pointer, what its pixel knows, and its orbit.
    hover: Option<([f64; 2], Pixel, Orbit)>,
    /// Draw the orbit of the point under the pointer.
    show_orbit: bool,
    /// The picture must be uploaded again.
    redraw: bool,
}

impl eframe::App for MandelbrotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        egui::CentralPanel::default().show(ui, |ui| {
            let (rect, response) =
                ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            let scale = if self.sharp {
                ui.ctx().pixels_per_point()
            } else {
                1.0
            };
            self.fit(rect, scale);
            self.gestures(ui, &response, rect);
            self.advance(dt);

            let spent = self.picture.work(ITERATIONS_PER_FRAME);
            if spent > 0 || self.redraw {
                self.view
                    .set_picture(ui.ctx(), &self.picture, &self.colours);
                self.redraw = false;
            }
            if self.picture.is_done() {
                self.view.drop_backdrop();
            }
            self.view
                .paint(ui.painter(), rect, self.scale, self.picture.view());

            self.hover = response.hover_pos().and_then(|pos| {
                let at = (pos - rect.min) * self.scale;
                let v = self.picture.view();
                let (x, y) = (at.x.floor() as usize, at.y.floor() as usize);
                (at.x >= 0.0 && at.y >= 0.0 && x < v.width && y < v.height).then(|| {
                    let c = v.point(x, y);
                    let steps = self.picture.limit().min(ORBIT_STEPS);
                    (c, self.picture.pixel(x, y), orbit(c, steps))
                })
            });
            if self.show_orbit
                && let Some((_, _, orbit)) = &self.hover
            {
                let to_screen = |p: [f64; 2]| {
                    let [x, y] = self.picture.view().pixel_of(p);
                    rect.min + egui::vec2(x as f32, y as f32) / self.scale
                };
                draw_orbit(&ui.painter().with_clip_rect(rect), orbit, to_screen);
            }
        });

        if self.playback.running || !self.picture.is_done() {
            ui.ctx().request_repaint();
        }
    }
}

impl MandelbrotApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut playback = Playback::new(15, 2_000, "steps / second");
        playback.running = true; // the set is best seen emerging
        let placeholder = Viewport::fit(1, 1, HOME_CENTRE, HOME_SPAN);
        Self {
            picture: Picture::new(placeholder, 0),
            view: PlaneView::new("mandelbrot"),
            about: About::new(include_str!("../README.md")),
            playback,
            pace: Pace::default(),
            colours: Colours::new(Palette::Site),
            stop_at: 500,
            sharp: false,
            scale: 1.0,
            go_home: true,
            home_step: placeholder.step(),
            pan_carry: Vec2::ZERO,
            hover: None,
            show_orbit: true,
            redraw: true,
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("The Mandelbrot set");
        ui.label("Each point c of the plane is a number. Start at z = 0 and repeat z → z² + c. If z ever gets further than 2 from 0, it flies off for ever, and c is outside the set, coloured by the step at which it escaped. The points that never escape are the set.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small("Each step lets every point take one more turn of z → z² + c. Restart goes back to step 0.");
        ui.label(format!("Steps so far: {}", self.picture.limit()));
        ui.horizontal_wrapped(|ui| {
            ui.label("Stop at");
            ui.add(
                egui::DragValue::new(&mut self.stop_at)
                    .range(1..=MOST_STEPS)
                    .speed(5.0),
            );
            ui.label("steps");
            if ui
                .button("Go there now")
                .on_hover_text("Take every point straight to the stop limit")
                .clicked()
            {
                self.picture.raise_limit(self.stop_at);
                self.playback.running = false;
            }
        });
        if self.playback.running && self.picture.limit() >= self.stop_at {
            ui.small("Reached the stop limit: raise it to go on.");
        }
        if !self.picture.is_done() {
            ui.small(format!(
                "Computing: {:.0}%",
                self.picture.progress() * 100.0
            ));
        }
        ui.separator();

        ui.label(RichText::new("View").strong());
        ui.small("Scroll the mouse wheel or pinch to zoom in on the pointer. Drag, or swipe with two fingers, to move.");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Reset view").clicked() {
                self.go_home = true;
            }
            ui.label(format!(
                "magnified {}",
                magnification(self.home_step / (self.picture.view().step() * self.scale as f64))
            ));
        });
        if self.picture.view().at_precision_limit() {
            ui.small("This is as far as the demo can zoom: closer in, 64-bit numbers can't tell neighbouring pixels apart.");
        }
        ui.checkbox(&mut self.sharp, "Sharp: a point for every screen pixel")
            .on_hover_text("On a high-density screen, four times the work for a crisper picture.");
        ui.checkbox(&mut self.show_orbit, "Show the orbit of the point under the pointer")
            .on_hover_text("The path of z from 0, one dot per step: z₀ = 0 (ringed), z₁ = c (the pointer), z₂, ...");
        ui.separator();

        ui.label(RichText::new("Colours").strong());
        ui.horizontal_wrapped(|ui| {
            for palette in Palette::ALL {
                if ui
                    .selectable_label(self.colours.palette() == palette, palette.name())
                    .clicked()
                {
                    self.colours.set_palette(palette);
                    self.redraw = true;
                }
            }
        });
        self.redraw |= ui
            .checkbox(&mut self.colours.bands, "Show escape bands")
            .on_hover_text("Each band is the points that escape at the same step.")
            .changed();
        ui.label("The colours repeat every");
        self.redraw |= ui
            .add(
                egui::Slider::new(&mut self.colours.period, 2.0..=512.0)
                    .logarithmic(true)
                    .integer()
                    .suffix(" steps"),
            )
            .changed();
        ui.separator();

        match &self.hover {
            Some((c, pixel, orbit)) => {
                let digits = (-(self.picture.view().step().log10())).ceil().max(2.0) as usize + 1;
                ui.label(format!("Pointer at c = {}", complex(*c, digits)));
                ui.label(match *pixel {
                    Pixel::Fresh => "not computed yet".to_string(),
                    Pixel::Inside => "in the main cardioid or the disc left of it: known to be inside, without iterating".to_string(),
                    Pixel::Running(n) => format!("still within 2 of 0 after {n} steps"),
                    Pixel::Escaped { step, .. } => format!("escapes at step {step}"),
                });
                let cycle = match orbit.fate {
                    Fate::Cycle(1) => Some("z settles on a single point".to_string()),
                    Fate::Cycle(p) => Some(format!("z settles into a cycle of {p} points")),
                    Fate::Undecided(n) if n > 0 && !matches!(pixel, Pixel::Escaped { .. }) => {
                        Some(format!("no cycle found in {n} steps"))
                    }
                    _ => None,
                };
                if let Some(cycle) = cycle {
                    ui.label(cycle);
                }
            }
            None => {
                ui.label("Point at the picture to see a point's number and fate.");
            }
        }
    }

    /// Make the picture match the space it is drawn in, keeping its centre and zoom.
    fn fit(&mut self, rect: egui::Rect, scale: f32) {
        let w = ((rect.width() * scale).floor() as usize).max(1);
        let h = ((rect.height() * scale).floor() as usize).max(1);
        if self.go_home {
            self.go_home = false;
            self.scale = scale;
            let home = Viewport::fit(w, h, HOME_CENTRE, HOME_SPAN);
            self.home_step = home.step() * scale as f64;
            self.replace(home);
            return;
        }
        let v = *self.picture.view();
        if (v.width, v.height) != (w, h) || scale != self.scale {
            let step = v.step() * self.scale as f64 / scale as f64;
            self.scale = scale;
            self.replace(v.resized(w, h, step));
        }
    }

    /// Start a new picture of `view` at the current limit, keeping the old one as a
    /// backdrop if it was finished (or there is nothing better).
    fn replace(&mut self, view: Viewport) {
        if self.picture.is_done() || !self.view.has_backdrop() {
            self.view.keep_as_backdrop();
        }
        self.picture = Picture::new(view, self.picture.limit());
        self.redraw = true;
    }

    /// Drags and two-finger swipes move the picture; the mouse wheel and pinches zoom.
    fn gestures(&mut self, ui: &egui::Ui, response: &egui::Response, rect: egui::Rect) {
        let mut pan = Vec2::ZERO;
        let mut zoom = 1.0f32;
        if response.dragged() {
            pan += response.drag_delta();
        }
        if response.contains_pointer() {
            ui.input(|i| {
                for event in &i.events {
                    let &Event::MouseWheel {
                        unit,
                        delta,
                        modifiers,
                        source,
                        ..
                    } = event
                    else {
                        continue;
                    };
                    if modifiers.command {
                        continue; // egui reads command-scroll as a zoom: see zoom_delta
                    }
                    let points = match unit {
                        MouseWheelUnit::Point => delta,
                        MouseWheelUnit::Line => delta * 40.0,
                        MouseWheelUnit::Page => delta * rect.height(),
                    };
                    if is_wheel(source, unit, delta) {
                        zoom *= (points.y.clamp(-50.0, 50.0) * 0.0045).exp();
                    } else {
                        pan += points;
                    }
                }
                zoom *= i.zoom_delta();
            });
        }

        if pan != Vec2::ZERO {
            let moved = pan * self.scale + self.pan_carry;
            let whole = moved.round();
            self.pan_carry = moved - whole;
            self.picture.pan(whole.x as i64, whole.y as i64);
            self.redraw = true;
        }
        if zoom != 1.0 {
            let anchor = response.hover_pos().unwrap_or(rect.center());
            let at = (anchor - rect.min) * self.scale;
            let old = *self.picture.view();
            let new = old.zoomed(at.x as f64, at.y as f64, zoom as f64);
            if new.step() != old.step() {
                self.replace(new);
            }
        }
    }

    /// Apply this frame's Restart, Step or Run to the limit.
    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.picture = Picture::new(*self.picture.view(), 0);
            self.view.drop_backdrop();
            self.redraw = true;
        }
        let limit = self.picture.limit();
        match self.playback.advance() {
            Advance::Idle => {}
            Advance::Step => self.picture.raise_limit(limit.saturating_add(1)),
            Advance::Run(per_second) => {
                if limit >= self.stop_at {
                    self.playback.running = false;
                } else if self.picture.is_done() {
                    // Only once the last step is drawn: a slow picture holds the limit back
                    // rather than falling ever further behind it.
                    let steps = self.pace.steps(per_second as f64, dt, MOST_STEPS as usize);
                    let target = limit.saturating_add(steps as u32).min(self.stop_at);
                    self.picture.raise_limit(target);
                }
            }
        }
    }
}

/// Draw an orbit's path over the picture: a line through its points, a dot at each, a ring
/// round z₀ = 0, and gold dots on the cycle it settles into, if it does.
fn draw_orbit(painter: &egui::Painter, orbit: &Orbit, to_screen: impl Fn([f64; 2]) -> egui::Pos2) {
    let points: Vec<egui::Pos2> = orbit
        .points
        .iter()
        .take(ORBIT_DRAWN)
        .map(|&p| to_screen(p))
        .collect();
    let shadow = Color32::from_black_alpha(150);
    let ink = Color32::from_white_alpha(220);
    painter.add(egui::Shape::line(points.clone(), Stroke::new(3.0, shadow)));
    painter.add(egui::Shape::line(points.clone(), Stroke::new(1.25, ink)));
    for &p in &points {
        painter.circle_filled(p, 2.5, ink);
    }
    painter.circle_stroke(points[0], 6.0, Stroke::new(3.0, shadow));
    painter.circle_stroke(points[0], 6.0, Stroke::new(1.5, ink));
    if let Fate::Cycle(p) = orbit.fate {
        for &z in orbit.points.iter().rev().take(p as usize) {
            let at = to_screen(z);
            painter.circle_filled(at, 5.0, shadow);
            painter.circle_filled(at, 4.0, CYCLE);
        }
    }
}

/// Whether a scroll came from a mouse wheel (to zoom) rather than fingers on a trackpad
/// (to move).
fn is_wheel(source: MouseWheelSource, unit: MouseWheelUnit, delta: Vec2) -> bool {
    match source {
        MouseWheelSource::Wheel => true,
        MouseWheelSource::Trackpad | MouseWheelSource::Momentum => false,
        // The browser doesn't say which. A wheel there scrolls by lines, or (Chrome, Safari)
        // by about 100 points a notch, straight up or down; a swipe comes as many small
        // moves, usually a little sideways too.
        _ => unit != MouseWheelUnit::Point || (delta.x == 0.0 && delta.y.abs() >= 50.0),
    }
}

/// A magnification for people: "×3.5", "×1,200", "×4.2e9".
fn magnification(m: f64) -> String {
    if m < 10.0 {
        format!("×{m:.1}")
    } else if m < 1e6 {
        let whole = m.round() as u64;
        let digits = whole.to_string();
        let mut grouped = String::new();
        for (i, d) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                grouped.push(',');
            }
            grouped.push(d);
        }
        format!("×{grouped}")
    } else {
        format!("×{m:.1e}")
    }
}

/// `c` as "a + bi", to `digits` decimal places.
fn complex(c: [f64; 2], digits: usize) -> String {
    let sign = if c[1] < 0.0 { '−' } else { '+' };
    let re = format!("{:.*}", digits, c[0]).replace('-', "−");
    format!("{re} {sign} {:.*}i", digits, c[1].abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magnifications_read_well() {
        assert_eq!(magnification(3.46), "×3.5");
        assert_eq!(magnification(1_234.4), "×1,234");
        assert_eq!(magnification(4.2e9), "×4.2e9");
    }

    /// A mouse wheel zooms and a trackpad moves the picture. In the browser, where the
    /// source is unknown, a big straight-down scroll is taken for a wheel notch.
    #[test]
    fn wheels_and_swipes() {
        use MouseWheelSource::*;
        let notch = Vec2::new(0.0, -100.0);
        let swipe = Vec2::new(2.0, -7.0);
        assert!(is_wheel(Wheel, MouseWheelUnit::Point, swipe));
        assert!(!is_wheel(Trackpad, MouseWheelUnit::Point, notch));
        assert!(is_wheel(Unknown, MouseWheelUnit::Point, notch));
        assert!(is_wheel(Unknown, MouseWheelUnit::Line, Vec2::new(0.0, 3.0)));
        assert!(!is_wheel(Unknown, MouseWheelUnit::Point, swipe));
    }
}
