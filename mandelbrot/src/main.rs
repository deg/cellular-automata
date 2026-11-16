//! The Mandelbrot set: every point c of the plane iterates z -> z² + c from 0, and is
//! coloured by the step at which |z| passes 2, or drawn as the set if it never does. The
//! iteration limit is raised a step at a time, like the generations of the other demos. An
//! inset shows the Julia set of the point under the pointer, and can be enlarged.

mod colour;
mod dd;
mod pane;
mod sim;
mod view;

use ca_core::egui::{
    self, Align2, Color32, Event, FontId, MouseWheelSource, MouseWheelUnit, Rect, RichText, Sense,
    Stroke, Vec2,
};
use ca_core::{About, Advance, Pace, Playback, eframe};
use colour::{Colours, Palette};
use dd::Dd;
use pane::Pane;
use sim::{Family, Fate, Orbit, Pixel, orbit};

fn main() {
    ca_core::run("The Mandelbrot set", MandelbrotApp::new);
}

/// The views Reset returns to: the whole set, and a whole Julia set, with a little margin.
const MANDELBROT_HOME: ([f64; 2], [f64; 2]) = ([-0.75, 0.0], [3.2, 2.6]);
const JULIA_HOME: ([f64; 2], [f64; 2]) = ([0.0, 0.0], [3.4, 3.4]);

/// The Julia set shown before the pointer has been over the picture: one whose orbit
/// settles into a cycle of three (see the tests in sim.rs).
const FIRST_JULIA: [f64; 2] = [-0.1226, 0.7449];

/// Milliseconds of each frame given to computing the main picture, the Julia inset and the
/// overview map, leaving the rest of a 16 ms frame for building and drawing the pictures.
/// They are turned into iterations by [`Budget`], from the speed measured as it goes: the
/// browser runs at a different speed from a native build, and machines differ.
const MAIN_MS: f64 = 7.0;
const INSET_MS: f64 = 2.0;
const MAP_MS: f64 = 1.0;

/// Running speeds up as the steps add up, from this step on: see [`run_rate`].
const SPEED_UP_FROM: f64 = 15.0;

/// The inset's limit stops here even when the main picture's goes higher: its set is
/// recomputed every time the pointer moves, and needs to keep up.
const INSET_MOST_STEPS: u32 = 2_000;

/// The inset's side, as a fraction of the main picture's shorter side.
const INSET_SIZE: f32 = 0.36;

/// The highest limit offered.
const MOST_STEPS: u32 = 1_000_000;

/// The most steps of the pointer's orbit followed (looking for a cycle), and drawn.
const ORBIT_STEPS: u32 = 20_000;
const ORBIT_DRAWN: usize = 1_000;

/// The dots on the cycle an orbit settles into, and the pinned point: the site's gold.
const GOLD: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

/// The overview map's limit: it is computed once (until the window or the limit changes),
/// so it needn't go deep.
const MAP_MOST_STEPS: u32 = 1_000;

/// The least work a frame is given, and the least `Budget::record` learns from.
const BUDGET_FLOOR: u64 = 50_000;
const LEARN_FROM: u64 = 20_000;

/// The shortest time `Budget::record` believes, in seconds: about the step of a browser's
/// clock.
const CLOCK_STEP: f64 = 1e-4;

/// How many iterations a frame can afford, from the speed measured as the pictures are
/// computed. A fixed count, measured natively, made frames of 60 ms and more in the browser.
struct Budget {
    /// Iterations (each pixel's overhead counted as a few more: see `sim::PIXEL_COST`) a
    /// second, smoothed.
    per_second: f64,
}

impl Budget {
    fn new() -> Self {
        // A guess at a browser's speed, about 10 ns an iteration; measured from the first
        // frame on.
        Self { per_second: 1e8 }
    }

    /// The iterations that fit in `ms` milliseconds. Never less than [`BUDGET_FLOOR`], which
    /// is more than the least `record` learns from, so a budget that has fallen too low still
    /// gives samples to learn from and climbs back: with the floor below that threshold, it
    /// once could stay stuck there for good.
    fn for_ms(&self, ms: f64) -> u64 {
        ((self.per_second * ms / 1000.0) as u64).max(BUDGET_FLOOR)
    }

    /// Learn from `spent` iterations taking `seconds`. Small amounts are timed too coarsely
    /// to learn from. A time under [`CLOCK_STEP`] is taken as that: browsers round their
    /// clock to about a tenth of a millisecond, so a fast machine's small budget can read as
    /// no time at all. (Ignoring short times instead would trap a fast machine's budget at
    /// the floor, its work always too quick to time.)
    fn record(&mut self, spent: u64, seconds: f64) {
        if spent >= LEARN_FROM {
            let rate = spent as f64 / seconds.max(CLOCK_STEP);
            self.per_second = 0.8 * self.per_second + 0.2 * rate;
        }
    }
}

/// The small pictures drawn over the main one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Overlay {
    /// The Julia set of c, bottom right.
    Julia,
    /// The whole set, with the main view marked on it, top right.
    Map,
}

/// What is under the pointer: its plane point, what its pixel knows, and its orbit.
struct Hover {
    point: [f64; 2],
    /// The point in full, in a view too deep for `point` to hold all its digits.
    precise: Option<[Dd; 2]>,
    pixel: Pixel,
    orbit: Orbit,
}

struct MandelbrotApp {
    mandelbrot: Pane,
    julia: Pane,
    /// The overview map: the whole of whichever set fills the view.
    map: Pane,
    about: About,
    playback: Playback,
    pace: Pace,
    colours: Colours,
    /// Running stops when the limit gets here.
    stop_at: u32,
    /// One pixel per screen pixel rather than per point: sharper on a high-density screen,
    /// and that much more work.
    sharp: bool,
    /// The c of the Julia set shown: the point last under the pointer, unless pinned.
    julia_c: [f64; 2],
    pinned: bool,
    show_julia: bool,
    /// The Julia set fills the view, in place of the Mandelbrot set.
    julia_full: bool,
    show_orbit: bool,
    show_map: bool,
    hover: Option<Hover>,
    budget: Budget,
}

impl eframe::App for MandelbrotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        egui::CentralPanel::default().show(ui, |ui| self.picture_area(ui, dt));

        if self.playback.running || !self.active().picture().is_done() {
            ui.ctx().request_repaint();
        }
    }
}

impl MandelbrotApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut playback = Playback::new(15, 2_000, "steps / second");
        playback.running = true; // the set is best seen emerging
        let (m_centre, m_span) = MANDELBROT_HOME;
        let (j_centre, j_span) = JULIA_HOME;
        Self {
            mandelbrot: Pane::new("mandelbrot", Family::Mandelbrot, m_centre, m_span),
            julia: Pane::new("julia", Family::Julia(FIRST_JULIA), j_centre, j_span),
            map: Pane::new("map", Family::Mandelbrot, m_centre, m_span),
            about: About::new(include_str!("../README.md")),
            playback,
            pace: Pace::default(),
            colours: Colours::new(Palette::Site),
            stop_at: 500,
            sharp: false,
            julia_c: FIRST_JULIA,
            pinned: false,
            show_julia: true,
            julia_full: false,
            show_orbit: true,
            show_map: true,
            hover: None,
            budget: Budget::new(),
        }
    }

    /// The pane filling the view.
    fn active(&self) -> &Pane {
        if self.julia_full {
            &self.julia
        } else {
            &self.mandelbrot
        }
    }

    fn active_mut(&mut self) -> &mut Pane {
        if self.julia_full {
            &mut self.julia
        } else {
            &mut self.mandelbrot
        }
    }

    /// The insets drawn over the picture in `rect` this frame.
    fn overlays(&self, rect: Rect) -> Vec<(Overlay, Rect)> {
        let mut found = Vec::new();
        if self.show_julia && !self.julia_full {
            found.push((Overlay::Julia, inset_rect(rect)));
        }
        if self.show_map {
            let (_, span) = self.map_home();
            found.push((Overlay::Map, map_rect(rect, span)));
        }
        found
    }

    /// The overview map's view: the whole of the set filling the picture.
    fn map_home(&self) -> ([f64; 2], [f64; 2]) {
        if self.julia_full {
            JULIA_HOME
        } else {
            MANDELBROT_HOME
        }
    }

    fn picture_area(&mut self, ui: &mut egui::Ui, dt: f64) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let scale = if self.sharp {
            ui.ctx().pixels_per_point()
        } else {
            1.0
        };
        let overlay_at = |app: &Self, pos: Option<egui::Pos2>| {
            let pos = pos?;
            app.overlays(rect)
                .into_iter()
                .find(|(_, r)| r.contains(pos))
                .map(|(o, _)| o)
        };
        // A click first: one on the Julia inset enlarges it, and the rest of the frame must
        // then treat the Julia set as filling the view.
        if response.clicked() {
            let pos = response.interact_pointer_pos();
            self.click(pos, overlay_at(self, pos), rect);
        }
        let pointer = response.hover_pos();
        let over = overlay_at(self, pointer);

        self.active_mut().fit(rect, scale);
        let (pan, zoom) = gestures(ui, &response, rect);
        if pan != Vec2::ZERO && (response.dragged() || over.is_none()) {
            self.active_mut().pan(pan);
        }
        if zoom != 1.0 && over.is_none() {
            let anchor = pointer.unwrap_or(rect.center()) - rect.min;
            self.active_mut().zoom(anchor, zoom);
        }
        self.advance(dt);

        // What is under the pointer, and the Julia set to show.
        self.hover = pointer.filter(|_| over.is_none()).and_then(|pos| {
            let (point, pixel) = self.active().pixel_at(pos, rect)?;
            let steps = self.active().picture().limit().min(ORBIT_STEPS);
            let orbit = match self.julia_full {
                false => orbit([0.0, 0.0], point, steps),
                true => orbit(point, self.julia_c, steps),
            };
            Some(Hover {
                point,
                precise: self.active().precise_point_at(pos, rect),
                pixel,
                orbit,
            })
        });
        if let Some(hover) = &self.hover
            && !self.julia_full
            && !self.pinned
        {
            self.julia_c = hover.point;
        }
        self.julia.set_family(Family::Julia(self.julia_c));

        let ctx = ui.ctx().clone();
        let painter = ui.painter().with_clip_rect(rect);
        let budget = self.budget.for_ms(MAIN_MS);
        let active = if self.julia_full {
            &mut self.julia
        } else {
            &mut self.mandelbrot
        };
        let (spent, seconds) = active.update(&ctx, &self.colours, budget);
        self.budget.record(spent, seconds);
        if self.julia_full {
            self.julia.paint(&painter, rect);
        } else {
            self.mandelbrot.paint(&painter, rect);
            if self.pinned {
                let at = self.mandelbrot.to_screen(self.julia_c, rect);
                painter.circle_stroke(at, 7.0, Stroke::new(3.0, Color32::from_black_alpha(150)));
                painter.circle_stroke(at, 7.0, Stroke::new(1.5, GOLD));
            }
        }
        // An orbit is drawn only where it belongs: in the Julia set's plane. Over the
        // Mandelbrot set, it is drawn in the inset instead (see draw_julia_inset).
        if self.julia_full
            && self.show_orbit
            && let Some(hover) = &self.hover
        {
            draw_orbit(&painter, &hover.orbit, 1.0, |p| {
                self.julia.to_screen(p, rect)
            });
        }

        for (overlay, inner) in self.overlays(rect) {
            let hovered = over == Some(overlay);
            let inner_painter = painter.with_clip_rect(inner);
            painter.rect_filled(inner, 0.0, self.colours.palette().inside());
            let caption = match overlay {
                Overlay::Julia => {
                    self.draw_julia_inset(&ctx, &inner_painter, inner, scale);
                    "Julia set"
                }
                Overlay::Map => {
                    self.draw_map(&ctx, &inner_painter, inner, scale);
                    "Overview"
                }
            };
            let edge = Stroke::new(
                1.5,
                Color32::from_white_alpha(if hovered { 230 } else { 140 }),
            );
            painter.rect_stroke(inner, 0.0, edge, egui::StrokeKind::Outside);
            painter.text(
                inner.left_top() + egui::vec2(6.0, 4.0),
                Align2::LEFT_TOP,
                caption,
                FontId::proportional(13.0),
                Color32::from_white_alpha(220),
            );
            if hovered {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
        // Only for insets on screen: a hidden one is never worked on, so waiting for it
        // would repaint for ever.
        let inset_busy = self.show_julia && !self.julia_full && !self.julia.picture().is_done();
        let map_busy = self.show_map && !self.map.picture().is_done();
        if inset_busy || map_busy {
            ui.ctx().request_repaint();
        }
    }

    /// The Julia set of c in its inset, with the orbit of 0, which belongs to its plane.
    fn draw_julia_inset(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        inset: Rect,
        scale: f32,
    ) {
        self.julia.fit(inset, scale);
        self.julia
            .update(ctx, &self.colours, self.budget.for_ms(INSET_MS));
        self.julia.paint(painter, inset);
        if self.show_orbit {
            // The whole of a bounded orbit is always in view here, however far the main
            // picture is zoomed.
            let steps = self.mandelbrot.picture().limit().min(ORBIT_STEPS);
            let path = orbit([0.0, 0.0], self.julia_c, steps);
            let julia = &self.julia;
            draw_orbit(painter, &path, 0.6, |p| julia.to_screen(p, inset));
        }
    }

    /// The whole set in the overview map, with the main view marked on it: a rectangle, or
    /// cross hairs once that is too small to see.
    fn draw_map(&mut self, ctx: &egui::Context, painter: &egui::Painter, inner: Rect, scale: f32) {
        let (centre, span) = self.map_home();
        let family = self.active().picture().family();
        self.map.set_family(family);
        self.map.set_home(centre, span);
        self.map.fit(inner, scale);
        self.map
            .raise_limit(self.active().picture().limit().min(MAP_MOST_STEPS));
        self.map
            .update(ctx, &self.colours, self.budget.for_ms(MAP_MS));
        self.map.paint(painter, inner);

        let view = self.active().picture().view();
        let corner = |x: f64, y: f64| self.map.to_screen(view.at(x, y), inner);
        let shown = Rect::from_two_pos(
            corner(0.0, 0.0),
            corner(view.width as f64, view.height as f64),
        );
        let shadow = Stroke::new(3.0, Color32::from_black_alpha(150));
        let mark = Stroke::new(1.5, GOLD);
        if shown.width().max(shown.height()) >= 8.0 {
            for stroke in [shadow, mark] {
                painter.rect_stroke(shown, 0.0, stroke, egui::StrokeKind::Middle);
            }
        } else {
            let c = shown.center();
            for stroke in [shadow, mark] {
                painter.line_segment([c - egui::vec2(9.0, 0.0), c - egui::vec2(3.0, 0.0)], stroke);
                painter.line_segment([c + egui::vec2(3.0, 0.0), c + egui::vec2(9.0, 0.0)], stroke);
                painter.line_segment([c - egui::vec2(0.0, 9.0), c - egui::vec2(0.0, 3.0)], stroke);
                painter.line_segment([c + egui::vec2(0.0, 3.0), c + egui::vec2(0.0, 9.0)], stroke);
            }
        }
        // A Julia set's map is its own plane, where the orbit of a starting point belongs.
        if self.julia_full
            && self.show_orbit
            && let Some(hover) = &self.hover
        {
            let map = &self.map;
            draw_orbit(painter, &hover.orbit, 0.6, |p| map.to_screen(p, inner));
        }
    }

    /// A click on the Julia inset enlarges it, and one on the map moves the main view there.
    /// A click on the Mandelbrot set pins the Julia set's c, so the pointer can move away.
    fn click(&mut self, pos: Option<egui::Pos2>, on: Option<Overlay>, rect: Rect) {
        match on {
            Some(Overlay::Julia) => self.enlarge_julia(true),
            Some(Overlay::Map) => {
                let (_, span) = self.map_home();
                let inner = map_rect(rect, span);
                if let Some((point, _)) = pos.and_then(|p| self.map.pixel_at(p, inner)) {
                    self.active_mut().centre_on(point);
                }
            }
            None => {
                if !self.julia_full
                    && let Some((point, _)) = pos.and_then(|p| self.mandelbrot.pixel_at(p, rect))
                {
                    self.julia_c = point;
                    self.pinned = true;
                }
            }
        }
    }

    /// Show the Julia set full size (pinning its c, since the pointer is about to move over
    /// it), or go back to the inset.
    fn enlarge_julia(&mut self, full: bool) {
        self.julia_full = full;
        self.pinned |= full;
        self.julia.go_home();
        let limit = self.mandelbrot.picture().limit();
        if full {
            self.julia.raise_limit(limit);
        } else {
            // Back in the inset, its cap applies again. Limits only ever rise, so without
            // starting afresh it would keep the full view's (perhaps 50,000), and never
            // finish while the pointer moves.
            self.julia.restart_at(limit.min(INSET_MOST_STEPS));
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("The Mandelbrot set");
        ui.label("Each point c of the plane is a number. Start at z = 0 and repeat z → z² + c. If z ever gets further than 2 from 0, it flies off for ever, and c is outside the set, coloured by the step at which it escaped. The points that never escape are the set.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small("Each step lets every point take one more turn of z → z² + c. Running speeds up as it goes, since later steps change less; set a low speed to watch it slowly. Restart goes back to step 0.");
        let limit = self.active().picture().limit();
        ui.label(format!("Steps so far: {limit}"));
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
                self.raise_limits(self.stop_at);
                self.playback.running = false;
            }
        });
        if self.playback.running && limit >= self.stop_at {
            ui.small("Reached the stop limit: raise it to go on.");
        }
        if !self.active().picture().is_done() {
            ui.small(format!(
                "Computing: {:.0}%",
                self.active().picture().progress() * 100.0
            ));
        }
        ui.separator();

        ui.label(RichText::new("View").strong());
        ui.small("Scroll the mouse wheel or pinch to zoom in on the pointer. Drag, or swipe with two fingers, to move.");
        ui.horizontal_wrapped(|ui| {
            if ui.button("Reset view").clicked() {
                self.active_mut().go_home();
            }
            ui.label(format!(
                "magnified {}",
                magnification(self.active().magnification())
            ));
        });
        if self.active().picture().view().at_precision_limit() {
            ui.small(if self.julia_full {
                "This is as far as the demo can zoom into a Julia set: closer in, 64-bit numbers can't tell neighbouring pixels apart."
            } else {
                "This is as far as the demo can zoom: closer in, even its double-length numbers can't tell neighbouring pixels apart."
            });
        }
        ui.checkbox(&mut self.sharp, "Sharp: a point for every screen pixel")
            .on_hover_text("On a high-density screen, four times the work for a crisper picture.");
        ui.checkbox(&mut self.show_map, "Show the overview map")
            .on_hover_text(
                "The whole set, with the view marked on it in gold. Click it to move there.",
            );
        ui.separator();

        ui.label(RichText::new("Julia set").strong());
        if self.julia_full {
            ui.label(format!(
                "The Julia set of c = {}: the starting points z₀ whose orbit stays near 0.",
                complex(self.julia_c, 4)
            ));
            ui.checkbox(
                &mut self.show_orbit,
                "Show the orbit of the point under the pointer",
            )
            .on_hover_text("The path of z, one dot per step, from z₀ (ringed) at the pointer.");
            if ui.button("Back to the Mandelbrot set").clicked() {
                self.enlarge_julia(false);
            }
        } else {
            ui.checkbox(&mut self.show_julia, "Show the Julia set of c");
            ui.small("Each c also has a Julia set: the starting points z₀ whose orbit under z → z² + c stays near 0. The inset shows it for the c under the pointer. Click the picture to pin c, and click the inset to enlarge it.");
            ui.horizontal_wrapped(|ui| {
                if self.pinned {
                    ui.label(format!("Pinned at c = {}", complex(self.julia_c, 4)));
                    if ui.button("Follow the pointer").clicked() {
                        self.pinned = false;
                    }
                } else {
                    ui.label("Following the pointer");
                }
                if self.show_julia && ui.button("Enlarge").clicked() {
                    self.enlarge_julia(true);
                }
            });
            if self.show_julia {
                ui.checkbox(&mut self.show_orbit, "Show the orbit of 0 in the inset")
                    .on_hover_text("The path of z under z → z² + c, one dot per step, from z₀ = 0 (ringed). Its points belong to the Julia set's plane, so this is where it is drawn.");
            }
        }
        ui.separator();

        ui.label(RichText::new("Colours").strong());
        ui.horizontal_wrapped(|ui| {
            for palette in Palette::ALL {
                if ui
                    .selectable_label(self.colours.palette() == palette, palette.name())
                    .clicked()
                {
                    self.colours.set_palette(palette);
                    self.recolour();
                }
            }
        });
        let bands = ui
            .checkbox(&mut self.colours.bands, "Show escape bands")
            .on_hover_text("Each band is the points that escape at the same step.")
            .changed();
        ui.label("The colours repeat every");
        let period = ui
            .add(
                egui::Slider::new(&mut self.colours.period, 2.0..=512.0)
                    .logarithmic(true)
                    .integer()
                    .suffix(" steps"),
            )
            .changed();
        if bands || period {
            self.recolour();
        }
        ui.separator();

        self.pointer_readout(ui);
    }

    /// What the point under the pointer is, and what becomes of it.
    fn pointer_readout(&self, ui: &mut egui::Ui) {
        let Some(hover) = &self.hover else {
            ui.label("Point at the picture to see a point's number and fate.");
            return;
        };
        let step = self.active().picture().view().step();
        let digits = (-(step.log10())).ceil().max(2.0) as usize + 1;
        let name = if self.julia_full { "z₀" } else { "c" };
        let shown = match hover.precise {
            Some(p) => complex_dd(p, digits),
            None => complex(hover.point, digits),
        };
        ui.label(format!("Pointer at {name} = {}", shown));
        ui.label(match hover.pixel {
            Pixel::Fresh => "not computed yet".to_string(),
            Pixel::Inside => {
                "in the main cardioid or the disc left of it: known to be inside, without iterating"
                    .to_string()
            }
            Pixel::Settled(n) => format!("settled into a cycle by step {n}: inside the set"),
            Pixel::Running(n) => format!("still near 0 after {n} steps"),
            Pixel::Escaped { step, .. } => format!("escapes at step {step}"),
        });
        let cycle = match hover.orbit.fate {
            Fate::Cycle(1) => Some("z settles on a single point".to_string()),
            Fate::Cycle(p) => Some(format!("z settles into a cycle of {p} points")),
            Fate::Undecided(n) if n > 0 && !matches!(hover.pixel, Pixel::Escaped { .. }) => {
                Some(format!("no cycle found in {n} steps"))
            }
            _ => None,
        };
        if let Some(cycle) = cycle {
            ui.label(cycle);
        }
    }

    fn recolour(&mut self) {
        self.mandelbrot.recolour();
        self.julia.recolour();
        self.map.recolour();
    }

    /// Let both pictures go on to `limit` steps, the inset only as far as it can keep up.
    fn raise_limits(&mut self, limit: u32) {
        self.mandelbrot.raise_limit(limit);
        let julia_most = if self.julia_full {
            limit
        } else {
            limit.min(INSET_MOST_STEPS)
        };
        self.julia.raise_limit(julia_most);
    }

    /// Apply this frame's Restart, Step or Run to the limit.
    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.mandelbrot.restart();
            self.julia.restart();
            self.map.restart();
        }
        let limit = self.active().picture().limit();
        match self.playback.advance() {
            Advance::Idle => {}
            Advance::Step => self.raise_limits(limit.saturating_add(1)),
            Advance::Run(per_second) => {
                if limit >= self.stop_at {
                    self.playback.running = false;
                } else if self.active().picture().is_done() {
                    // Only once the last step is drawn: a slow picture holds the limit back
                    // rather than falling ever further behind it.
                    let rate = run_rate(per_second as f64, limit);
                    let steps = self.pace.steps(rate, dt, MOST_STEPS as usize);
                    let target = limit.saturating_add(steps as u32).min(self.stop_at);
                    self.raise_limits(target);
                }
            }
        }
    }
}

/// Steps a second while running, at `slider` steps a second on the speed slider, with `limit`
/// steps taken. Later steps change the picture less and less, and at a constant 15 a second
/// the first 500 took over half a minute, so past [`SPEED_UP_FROM`] steps the rate grows in
/// proportion to the count: at the default 15, the count grows e-fold each second, and 500
/// comes in about 5 seconds. The slider scales the whole thing, so a slow setting still
/// gives a slow reveal (to watch the bands appear) and a fast one goes faster.
fn run_rate(slider: f64, limit: u32) -> f64 {
    slider * (limit as f64 / SPEED_UP_FROM).max(1.0)
}

/// The inset's place: a square in the bottom-right corner of the picture.
fn inset_rect(rect: Rect) -> Rect {
    let side = (rect.width().min(rect.height()) * INSET_SIZE).round();
    let margin = 10.0;
    Rect::from_min_size(
        rect.right_bottom() - egui::vec2(side + margin, side + margin),
        egui::vec2(side, side),
    )
}

/// The overview map's place: as wide as the Julia inset, in the top-right corner, and as tall
/// as the whole set's `span` needs.
fn map_rect(rect: Rect, span: [f64; 2]) -> Rect {
    let width = (rect.width().min(rect.height()) * INSET_SIZE).round();
    let height = (width * (span[1] / span[0]) as f32).round();
    let margin = 10.0;
    Rect::from_min_size(
        rect.right_top() + egui::vec2(-(width + margin), margin),
        egui::vec2(width, height),
    )
}

/// This frame's moves and zooms over the picture, in points and as a factor. Drags and
/// two-finger swipes move it; the mouse wheel and pinches zoom.
fn gestures(ui: &egui::Ui, response: &egui::Response, rect: Rect) -> (Vec2, f32) {
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
    (pan, zoom)
}

/// Draw an orbit's path over the picture: a line through its points, a dot at each, a ring
/// round z₀, and gold dots on the cycle it settles into, if it does. `size` scales the marks
/// (1 for the main picture).
fn draw_orbit(
    painter: &egui::Painter,
    orbit: &Orbit,
    size: f32,
    to_screen: impl Fn([f64; 2]) -> egui::Pos2,
) {
    let points: Vec<egui::Pos2> = orbit
        .points
        .iter()
        .take(ORBIT_DRAWN)
        .map(|&p| to_screen(p))
        .collect();
    let shadow = Color32::from_black_alpha(150);
    let ink = Color32::from_white_alpha(220);
    painter.add(egui::Shape::line(
        points.clone(),
        Stroke::new(3.0 * size, shadow),
    ));
    painter.add(egui::Shape::line(
        points.clone(),
        Stroke::new(1.25 * size, ink),
    ));
    for &p in &points {
        painter.circle_filled(p, 2.5 * size, ink);
    }
    painter.circle_stroke(points[0], 6.0 * size, Stroke::new(3.0 * size, shadow));
    painter.circle_stroke(points[0], 6.0 * size, Stroke::new(1.5 * size, ink));
    if let Fate::Cycle(p) = orbit.fate {
        for &z in orbit.points.iter().rev().take(p as usize) {
            let at = to_screen(z);
            painter.circle_filled(at, 5.0 * size, shadow);
            painter.circle_filled(at, 4.0 * size, GOLD);
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

/// [`complex`] for a point held in double-double, with all its digits.
fn complex_dd(c: [Dd; 2], digits: usize) -> String {
    let sign = if c[1].to_f64() < 0.0 { '−' } else { '+' };
    let re = c[0].to_decimal(digits).replace('-', "−");
    let im = if c[1].to_f64() < 0.0 { -c[1] } else { c[1] };
    format!("{re} {sign} {}i", im.to_decimal(digits))
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

    /// A budget slowed by a run of slow frames (say, Sharp on with a big window) must climb
    /// back once frames are fast again. It once could not: its floor was below the least it
    /// learned from, so once low it never learned again.
    #[test]
    fn budget_recovers_from_slow_frames() {
        let mut b = Budget::new();
        for _ in 0..200 {
            let spent = b.for_ms(MAIN_MS);
            b.record(spent, 0.050); // every frame's work took 50 ms
        }
        assert_eq!(
            b.for_ms(MAIN_MS),
            BUDGET_FLOOR,
            "slow frames shrink it to the floor"
        );
        let fast = 4e8; // iterations a second
        for _ in 0..200 {
            let spent = b.for_ms(MAIN_MS);
            b.record(spent, spent as f64 / fast);
        }
        let expected = fast * MAIN_MS / 1000.0;
        assert!(
            (b.for_ms(MAIN_MS) as f64 / expected - 1.0).abs() < 0.05,
            "back up to the fast rate: {} of {expected}",
            b.for_ms(MAIN_MS)
        );
    }

    /// The speed-up must not override the slider: a low setting must stay slow (it once
    /// did nothing past step 15), the default must still reach 500 quickly, and a high
    /// setting must go faster still.
    #[test]
    fn running_speeds_up_in_proportion_to_the_slider() {
        assert_eq!(
            run_rate(15.0, 0),
            15.0,
            "the slider's own speed at the start"
        );
        assert_eq!(
            run_rate(15.0, 300),
            300.0,
            "the default: the count again each second"
        );
        assert_eq!(run_rate(2.0, 300), 40.0, "a slow slider stays slow");
        assert_eq!(run_rate(100.0, 300), 2000.0, "a fast one goes faster");
        assert!(run_rate(2.0, 300) < run_rate(15.0, 300));
    }

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
