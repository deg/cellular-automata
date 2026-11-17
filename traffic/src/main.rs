//! Traffic demo: the Nagel–Schreckenberg model of cars on a ring road, drawn as a
//! space-time diagram in which each step is a new row, with the flow measured against the
//! density of cars.

mod sim;

use std::collections::VecDeque;

use ca_core::egui::{self, Color32, CornerRadius, RichText, Sense, pos2, vec2};
use ca_core::{About, Advance, GridView, NAVY, Pace, Playback, Plot, Series, Style, eframe};
use sim::{EMPTY, History, Road, Sweep, flow_without_dawdling};

fn main() {
    ca_core::run("Traffic", TrafficApp::new);
}

const CORAL: [f32; 3] = [231.0, 111.0, 81.0];
const GOLD: [f32; 3] = [233.0, 196.0, 106.0];
const TEAL: [f32; 3] = [42.0, 157.0, 143.0];
/// This road's point on the chart.
const HERE: Color32 = Color32::from_rgb(0xe7, 0x6f, 0x51);
/// The chart's colours, as in the other demos' charts: measured points sky blue, the
/// reference (theory) lines gold.
const MEASURED: Color32 = Color32::from_rgb(0x5a, 0xb4, 0xff);
const REFERENCE: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

/// The colour of a car at speed `v` of `v_max`: coral standing still, gold at half speed,
/// teal at full speed, so jams stand out from free-flowing traffic.
fn speed_colour(v: u8, v_max: u8) -> Color32 {
    let f = f32::from(v.min(v_max)) / f32::from(v_max.max(1));
    let (a, b, t) = if f < 0.5 {
        (CORAL, GOLD, f * 2.0)
    } else {
        (GOLD, TEAL, f * 2.0 - 1.0)
    };
    let mix = |i: usize| (a[i] + (b[i] - a[i]) * t).round() as u8;
    Color32::from_rgb(mix(0), mix(1), mix(2))
}

/// Road lengths offered; the diagram keeps three rows for every four cells, as elementary's.
const LENGTHS: &[usize] = &[200, 400, 800];

const MAX_SPEED: u8 = 8;
const MAX_DAWDLING: f64 = 0.99;

/// The strip under the diagram showing the road now: its height and the space above it.
const STRIP_HEIGHT: f32 = 22.0;
const STRIP_GAP: f32 = 8.0;
/// The strip's tarmac, a little lighter than the empty cells above it.
const ROAD: Color32 = Color32::from_rgb(0x24, 0x29, 0x45);

/// The live road's flow is averaged over at most this many recent steps.
const WINDOW: usize = 200;

/// Car moves the chart's measurement may make each frame: about 1 ms natively (3 ns a car
/// move), a few in the browser; the whole chart takes a few seconds.
const SWEEP_BUDGET: usize = 400_000;

struct Preset {
    name: &'static str,
    v_max: u8,
    p: f64,
    density: f64,
    note: &'static str,
}

const PRESETS: &[Preset] = &[
    Preset {
        name: "Light traffic",
        v_max: 5,
        p: 0.5,
        density: 0.03,
        note: "Few cars: each nearly always drives at 5 or 4, and seldom meets another. Nagel's thesis shows this density.",
    },
    Preset {
        name: "Jams",
        v_max: 5,
        p: 0.5,
        density: 0.1,
        note: "Near the top of the flow curve, which is flat from about 0.08 to 0.10 here: jams start at random and drift backwards.",
    },
    Preset {
        name: "Stop and go",
        v_max: 5,
        p: 0.25,
        density: 0.2,
        note: "Denser, with less dawdling: waves of stopped cars all along the road. One of the review's pictures.",
    },
    Preset {
        name: "No dawdling",
        v_max: 5,
        p: 0.0,
        density: 0.5,
        note: "Nothing random: the jams of the random start settle into a pattern that moves back one cell a step.",
    },
    Preset {
        name: "Rule 184",
        v_max: 1,
        p: 0.0,
        density: 0.5,
        note: "Maximum speed 1 and no dawdling is elementary Rule 184: a car moves when the cell ahead is empty.",
    },
];

struct TrafficApp {
    road: Road,
    history: History,
    sweep: Sweep,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    length: usize,
    density: f64,
    seed: u64,
    /// The sum of speeds in each recent step, newest last.
    recent: VecDeque<usize>,
    /// The diagram's cell under the pointer, for the readout.
    hovered: Option<(usize, usize)>,
    /// The road strip's cell under the pointer.
    hovered_road: Option<usize>,
    dirty: bool,
}

impl eframe::App for TrafficApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if !self.sweep.done() {
            self.sweep.work(SWEEP_BUDGET);
            ui.ctx().request_repaint();
        }
        if self.dirty {
            let v_max = self.road.v_max;
            self.view.set_image(ui.ctx(), self.history.grid(), |c| {
                if c == EMPTY {
                    NAVY
                } else {
                    speed_colour(c, v_max)
                }
            });
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            // Room under the diagram for the road now.
            let size = ui.available_size() - vec2(0.0, STRIP_GAP + STRIP_HEIGHT);
            let shown = ui.allocate_ui(size, |ui| self.view.show(ui)).inner;
            let hovered_road = self.road_strip(ui, &shown);
            // The readout was drawn before the picture this frame: draw again, or the
            // pointer row lags a cell behind while paused.
            if shown.hovered != self.hovered || hovered_road != self.hovered_road {
                self.hovered = shown.hovered;
                self.hovered_road = hovered_road;
                ui.ctx().request_repaint();
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl TrafficApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let preset = &PRESETS[1];
        let length = LENGTHS[1];
        let road = Road::random(length, 0, preset.v_max, preset.p, 1);
        let mut playback = Playback::new(30, 1_000, "steps / second");
        playback.running = true; // jams forming on their own from a random start is the show
        let mut app = Self {
            history: History::new(&road, 1),
            sweep: Sweep::new(preset.v_max, preset.p),
            road,
            view: GridView::new("traffic"),
            about: About::new(include_str!("../README.md")),
            playback,
            pace: Pace::default(),
            length,
            density: preset.density,
            seed: 1,
            recent: VecDeque::with_capacity(WINDOW),
            hovered: None,
            hovered_road: None,
            dirty: true,
        };
        app.restart();
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Traffic");
        ui.label("Cars drive round a one-lane ring road. Each step every car speeds up by one, slows down so as not to hit the car ahead, sometimes dawdles one slower at random, then moves. Each step is drawn as the next row down.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small("Restart scatters the cars afresh at random, all standing still.");
        ui.separator();

        ui.label(RichText::new("Presets").strong());
        ui.horizontal_wrapped(|ui| {
            for p in PRESETS {
                if ui.selectable_label(self.matches(p), p.name).clicked() {
                    self.road.v_max = p.v_max;
                    self.road.p = p.p;
                    self.density = p.density;
                    self.restart();
                }
            }
        });
        if let Some(p) = PRESETS.iter().find(|p| self.matches(p)) {
            ui.small(p.note);
        }
        ui.separator();

        ui.label(RichText::new("Drivers").strong());
        let (v_max, p) = (self.road.v_max, self.road.p);
        ui.add(egui::Slider::new(&mut self.road.v_max, 1..=MAX_SPEED).text("maximum speed"));
        ui.add(
            // At 1 every car pulling away dawdles back to 0, so the road freezes for good
            // (review, Sec. 8.2.2): stop just short of it.
            egui::Slider::new(&mut self.road.p, 0.0..=MAX_DAWDLING)
                .step_by(0.01)
                .fixed_decimals(2)
                .text("dawdling"),
        );
        if ui
            .add(
                egui::Slider::new(&mut self.density, 0.01..=0.99)
                    .step_by(0.01)
                    .fixed_decimals(2)
                    .text("density"),
            )
            .changed()
        {
            self.restart();
        }
        ui.small("Dawdling is the chance that a car slows by one more. Changing the density restarts; the other two carry on.");
        if (self.road.v_max, self.road.p) != (v_max, p) {
            self.sweep = Sweep::new(self.road.v_max, self.road.p);
            self.recent.clear();
            self.dirty = true;
        }
        ui.separator();

        ui.label(RichText::new("Flow against density").strong());
        self.chart(ui);
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("Road length:");
            for &l in LENGTHS {
                if ui
                    .selectable_label(self.length == l, l.to_string())
                    .clicked()
                    && self.length != l
                {
                    self.length = l;
                    self.restart();
                }
            }
        });
        ui.separator();

        self.readout(ui);
    }

    fn matches(&self, p: &Preset) -> bool {
        self.road.v_max == p.v_max
            && (self.road.p - p.p).abs() < 1e-9
            && (self.density - p.density).abs() < 1e-9
    }

    /// The fundamental diagram: flow against density measured on test roads at the current
    /// settings, the flow with no dawdling for comparison, and this road's own point.
    fn chart(&self, ui: &mut egui::Ui) {
        let v_max = self.road.v_max;
        let this_road = self.recent_flow().map(|f| (self.road.density(), f));
        // The measured flows set the height, rounded up to a tenth so it seldom changes; the
        // no-dawdling lines, usually far higher, are cut off at the top.
        let highest = self
            .sweep
            .points()
            .iter()
            .chain(&this_road)
            .fold(0.0_f64, |m, p| m.max(p.1));
        let top = ((highest * 1.15 * 10.0).ceil() / 10.0).max(0.1);
        let peak = 1.0 / (f64::from(v_max) + 1.0);
        let rising_to = (top / f64::from(v_max)).min(peak);
        let falling_from = (1.0 - top).max(peak);
        let rising = [
            (0.0, 0.0),
            (rising_to, flow_without_dawdling(rising_to, v_max).min(top)),
        ];
        let falling = [
            (
                falling_from,
                flow_without_dawdling(falling_from, v_max).min(top),
            ),
            (1.0, 0.0),
        ];
        let line = |points, colour| Series {
            points,
            colour,
            style: Style::Line,
        };
        let mut series = vec![
            line(&rising, REFERENCE),
            line(&falling, REFERENCE),
            line(self.sweep.points(), MEASURED),
        ];
        let here = this_road.map(|point| [point]);
        if let Some(point) = &here {
            series.push(Series {
                points: point,
                colour: HERE,
                style: Style::Marker,
            });
        }
        let plot = Plot {
            x_label: "density",
            y_label: "flow",
            log_x: false,
            log_y: false,
            x_range: Some((0.0, 1.0)),
            y_range: Some((0.0, top)),
            series,
        };
        ca_core::plot(ui, &plot, 130.0);
        let measuring = if self.sweep.done() {
            ""
        } else {
            " (measuring...)"
        };
        ui.small(format!("Blue: measured on test roads at these settings{measuring}. Gold: with no dawdling. Coral dot: this road. Flow is cars passing a point per step."));
    }

    fn readout(&self, ui: &mut egui::Ui) {
        let road = &self.road;
        let cars = road.cars();
        let stopped = cars.iter().filter(|c| c.v == 0).count();
        let mean_speed = if cars.is_empty() {
            0.0
        } else {
            road.moved() as f64 / cars.len() as f64
        };
        let under = self
            .hovered_road
            .map(|x| (x, road.time, self.cell_text(x, None)))
            .or_else(|| {
                self.hovered
                    .map(|(x, y)| match self.history.time_of_row(y) {
                        Some(t) => (x, t, self.cell_text(x, Some(y))),
                        // Only while the diagram is still filling, when row y will hold step y.
                        None => (x, y as u64, "not reached yet".to_string()),
                    })
            })
            .map(|(x, t, text)| format!("({x}, {t}): {text}"));
        ca_core::readout(ui, |r| {
            r.row("step", ca_core::grouped(road.time));
            r.row(
                "density",
                format!(
                    "{:.2} ({} cars, {} cells)",
                    road.density(),
                    ca_core::grouped(cars.len() as u64),
                    ca_core::grouped(road.length() as u64)
                ),
            );
            r.row("mean speed", format!("{mean_speed:.2} cells / step"));
            r.row(
                "stopped",
                format!(
                    "{} ({:.0}%)",
                    ca_core::grouped(stopped as u64),
                    100.0 * stopped as f64 / cars.len().max(1) as f64
                ),
            );
            let flow = match self.recent_flow() {
                Some(f) => format!("{f:.3}, last {} steps", self.recent.len()),
                None => "-".to_string(),
            };
            r.row("flow", flow);
            r.pointer(under);
        });
    }

    /// What is in cell `x` of the diagram's row `y`, or of the road now.
    fn cell_text(&self, x: usize, y: Option<usize>) -> String {
        // The hovered cell is from the last frame, and the road may have changed size since.
        if x >= self.road.length() {
            return "-".to_string();
        }
        let cell = match y {
            Some(y) if y < self.history.grid().height() => self.history.grid().get(x, y),
            Some(_) => return "-".to_string(),
            None => {
                let mut row = vec![EMPTY; self.road.length()];
                self.road.write_row(&mut row);
                row[x]
            }
        };
        match cell {
            EMPTY => "empty".to_string(),
            0 => "stopped car".to_string(),
            v => format!("car at speed {v}"),
        }
    }

    /// The mean flow over the recent steps, if any.
    fn recent_flow(&self) -> Option<f64> {
        (!self.recent.is_empty()).then(|| {
            let total: usize = self.recent.iter().sum();
            total as f64 / (self.recent.len() * self.road.length()) as f64
        })
    }

    /// The road as it is now, drawn just under the diagram with the same columns, each car
    /// a block in its speed's colour. Returns the cell under the pointer.
    fn road_strip(&self, ui: &mut egui::Ui, shown: &ca_core::GridResponse) -> Option<usize> {
        let (length, rows) = (self.road.length(), self.history.grid().height());
        let width = shown.cell_size();
        if width <= 0.0 {
            return None;
        }
        let bottom_left = shown.cell_rect(0, rows - 1).left_bottom();
        let road = egui::Rect::from_min_size(
            bottom_left + vec2(0.0, STRIP_GAP),
            vec2(width * length as f32, STRIP_HEIGHT),
        );
        let painter = ui.painter();
        painter.rect_filled(road, CornerRadius::ZERO, ROAD);
        let v_max = self.road.v_max;
        for car in self.road.cars() {
            let x = road.left() + width * car.x as f32;
            let block = egui::Rect::from_min_max(
                pos2(x, road.top() + 4.0),
                pos2(x + width.max(1.0), road.bottom() - 4.0),
            );
            painter.rect_filled(block, CornerRadius::ZERO, speed_colour(car.v, v_max));
        }
        let response = ui.interact(road, ui.make_persistent_id("road now"), Sense::hover());
        let pos = response.hover_pos()?;
        let x = ((pos.x - road.left()) / width) as usize;
        (x < length).then_some(x)
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.seed += 1;
            self.restart();
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => self.pace.steps(per_second as f64, dt, 200),
        };
        for _ in 0..steps {
            self.road.step();
            self.history.push(&self.road);
            if self.recent.len() == WINDOW {
                self.recent.pop_front();
            }
            self.recent.push_back(self.road.moved());
        }
        self.dirty |= steps > 0;
    }

    /// A fresh random road with the current settings.
    fn restart(&mut self) {
        let count = (self.density * self.length as f64).round() as usize;
        let (v_max, p) = (self.road.v_max, self.road.p);
        self.road = Road::random(self.length, count, v_max, p, self.seed);
        self.history = History::new(&self.road, self.length * 3 / 4);
        if (self.sweep.v_max, self.sweep.p) != (v_max, p) {
            self.sweep = Sweep::new(v_max, p);
        }
        self.recent.clear();
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stopped cars are coral and full-speed ones teal at every maximum speed, and every
    /// speed in between gets a colour of its own.
    #[test]
    fn speeds_run_from_coral_to_teal() {
        for v_max in 1..=MAX_SPEED {
            assert_eq!(speed_colour(0, v_max), Color32::from_rgb(231, 111, 81));
            assert_eq!(speed_colour(v_max, v_max), Color32::from_rgb(42, 157, 143));
            let colours: std::collections::HashSet<_> =
                (0..=v_max).map(|v| speed_colour(v, v_max)).collect();
            assert_eq!(colours.len(), usize::from(v_max) + 1);
        }
    }

    /// Every preset is reachable with the panel's controls.
    #[test]
    fn presets_fit_the_controls() {
        for p in PRESETS {
            assert!((1..=MAX_SPEED).contains(&p.v_max), "{}", p.name);
            assert!((0.0..=MAX_DAWDLING).contains(&p.p) && (0.01..=0.99).contains(&p.density));
        }
    }
}
