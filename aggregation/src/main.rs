//! Diffusion-limited aggregation demo: particles wander at random until they touch a
//! growing cluster and stick, and the cluster grows into a branching fractal.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, GridView, NAVY, Pace, Playback, Plot, Series, Style, eframe, grouped, readout,
    speed_cap_note,
};
use sim::{Aggregate, EMPTY, FIT_FROM, Seed};

fn main() {
    ca_core::run("Diffusion-limited aggregation", AggregationApp::new);
}

/// The colours a cluster shades through as it grows, from the seed (first) to the
/// particles that joined last, like the time-of-arrival colours in Sander's pictures.
const SHADES: [Color32; 5] = [
    Color32::from_rgb(0xe9, 0xc4, 0x6a), // gold
    Color32::from_rgb(0xe7, 0x6f, 0x51), // coral
    Color32::from_rgb(0xd1, 0x6b, 0xa5), // pink
    Color32::from_rgb(0x5a, 0xb4, 0xff), // sky blue
    Color32::from_rgb(0xf4, 0xf1, 0xde), // spark white
];

/// The chart's points and its fitted line.
const POINTS: Color32 = Color32::from_rgb(0x5a, 0xb4, 0xff);
const FIT: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

/// Square worlds, by side.
const SIZES: &[usize] = &[250, 500, 1000];

/// Walker moves a frame can afford. Natively a move (a step or a big step) takes about
/// 70 ns (a 500 x 500 world fills with 12,000 particles of 94 moves each in 0.08 s), so
/// this is about 10 ms, with room left for drawing. The browser is slower and slows down.
const MOVES_PER_FRAME: u64 = 150_000;

struct AggregationApp {
    aggregate: Aggregate,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    /// The seed and world for the next Restart.
    seed: Seed,
    size: usize,
    /// The sticking probability, which applies at once.
    stick: f64,
    /// The random seed of the current cluster; Restart moves on to the next.
    run: u64,
    /// The cell under the pointer, as of the last frame, shown in the controls.
    pointer: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for AggregationApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            // Colours by when each particle joined, as a fraction of the particles so far,
            // through a table of 256 shades.
            let newest = self.aggregate.particles().max(1) as f64;
            let shades: Vec<Color32> = (0..=255).map(|i| shade(f64::from(i) / 255.0)).collect();
            self.view
                .set_image(ui.ctx(), self.aggregate.arrival(), |k| {
                    if k == EMPTY {
                        NAVY
                    } else {
                        shades[(f64::from(k) / newest * 255.0).round().min(255.0) as usize]
                    }
                });
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            // The project's demos show the cell under the pointer as the last row of the
            // readout, not as a tooltip on the grid.
            if shown.hovered != self.pointer {
                self.pointer = shown.hovered;
                ui.ctx().request_repaint(); // the controls were drawn before this
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl AggregationApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (seed, size, stick, run) = (Seed::Point, SIZES[1], 1.0, 1);
        let mut playback = Playback::new(300, 20_000, "particles / second");
        // A lone seed is nothing to look at: the cluster is best seen growing.
        playback.running = true;
        Self {
            aggregate: Aggregate::new(size, size, seed, stick, run),
            view: GridView::new("aggregation"),
            about: About::new(include_str!("../README.md")),
            playback,
            pace: Pace::default(),
            seed,
            size,
            stick,
            run,
            pointer: None,
            dirty: true,
        }
    }

    /// A fresh cluster, from new random walks, with the chosen seed and world.
    fn restart(&mut self) {
        self.run += 1;
        self.aggregate = Aggregate::new(self.size, self.size, self.seed, self.stick, self.run);
        self.dirty = true;
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Diffusion-limited aggregation");
        ui.label("Not quite a cellular automaton: one particle at a time wanders at random over the grid until it lands next to the cluster, and sticks there. The cluster grows into a branching fractal.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        speed_cap_note(ui, "world");
        ui.small("Restart grows a new cluster from new random walks. Colours run from gold at the seed to white for the newest particles.");
        ui.separator();

        ui.label(RichText::new("Seed").strong());
        ui.horizontal_wrapped(|ui| {
            for (seed, name) in [(Seed::Point, "point"), (Seed::Line, "line")] {
                if ui.selectable_label(self.seed == seed, name).clicked() && self.seed != seed {
                    self.seed = seed;
                    self.restart();
                }
            }
        });
        ui.small(match self.seed {
            Seed::Point => {
                "One particle in the middle. Walkers start on a circle just outside the cluster."
            }
            Seed::Line => {
                "The bottom row, its ends joined round. Walkers start just above the highest particle."
            }
        });
        ui.separator();

        ui.label(RichText::new("Sticking probability").strong());
        if ui
            .add(egui::Slider::new(&mut self.stick, 0.1..=1.0))
            .changed()
        {
            self.aggregate.stick = self.stick;
        }
        ui.small("Below 1, a particle next to the cluster may wander on; the branches grow thicker. Takes effect at once.");
        ui.separator();

        ui.label(RichText::new("Size against radius").strong());
        self.chart(ui);
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{size} x {size}");
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.restart();
                }
            }
        });
        ui.separator();

        let a = &self.aggregate;
        readout(ui, |r| {
            r.row("particle", grouped(a.particles()));
            let reach = format!("{:.0} of {:.0}", a.reach(), a.room());
            match a.seed() {
                Seed::Point => r.row("radius", reach),
                Seed::Line => r.row("height", reach),
            };
            r.pointer(self.pointer.map(|(x, y)| {
                let what = match a.arrival().get(x, y) {
                    EMPTY => "empty".to_string(),
                    0 if a.seed() == Seed::Point => "the seed".to_string(),
                    0 => "the line".to_string(),
                    k => format!("particle {}", grouped(u64::from(k))),
                };
                format!("({x}, {y}): {what}")
            }));
        });
        if a.finished() {
            ui.small(
                "Fully grown: the cluster has reached the edge of its world. Restart to grow another.",
            );
        }
    }

    /// The cluster's sites against its radius of gyration, on log scales, with the line
    /// fitted from [`FIT_FROM`] sites on: its slope is the fractal dimension.
    fn chart(&self, ui: &mut egui::Ui) {
        let a = &self.aggregate;
        if a.seed() == Seed::Line {
            ui.small(
                "The chart measures a cluster grown from a point: choose the point seed to see it.",
            );
            return;
        }
        let fit = a.dimension();
        let line: Vec<(f64, f64)> = match fit {
            Some((slope, intercept)) => {
                let from = a.history().iter().find(|p| p.1 >= FIT_FROM).map(|p| p.0);
                let to = a.history().last().map(|p| p.0);
                [from, to]
                    .into_iter()
                    .flatten()
                    .map(|r| (r, (intercept + slope * r.ln()).exp()))
                    .collect()
            }
            None => Vec::new(),
        };
        ca_core::plot(
            ui,
            &Plot {
                x_label: "radius of gyration",
                y_label: "sites",
                log_x: true,
                log_y: true,
                x_range: None,
                y_range: None,
                series: vec![
                    Series {
                        points: a.history(),
                        colour: POINTS,
                        style: Style::Dots,
                    },
                    Series {
                        points: &line,
                        colour: FIT,
                        style: Style::Line,
                    },
                ],
            },
            140.0,
        );
        match fit {
            Some((slope, _)) => ui.label(format!("Slope, the fractal dimension: {slope:.2}")),
            None => ui.label(format!(
                "The slope is shown once there are {} particles.",
                grouped(4 * FIT_FROM as u64)
            )),
        };
        ui.small("Published values are 1.67 to 1.71: see the explanation.");
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.restart();
        }
        let wanted = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => self.pace.steps(per_second as f64, dt, usize::MAX),
        };
        let start = self.aggregate.moves();
        for _ in 0..wanted {
            if self.aggregate.add_particle().is_none() {
                self.playback.running = false;
                break;
            }
            self.dirty = true;
            if self.aggregate.moves() - start > MOVES_PER_FRAME {
                break;
            }
        }
    }
}

/// The colour of a particle that joined a fraction `t` of the way through the growth so
/// far: 0 the seed, 1 the newest.
fn shade(t: f64) -> Color32 {
    let t = t.clamp(0.0, 1.0) * (SHADES.len() - 1) as f64;
    let i = (t.floor() as usize).min(SHADES.len() - 2);
    let f = t - i as f64;
    let (a, b) = (SHADES[i], SHADES[i + 1]);
    let m = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * f).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shades start at the seed's colour and end at the newest's, and no particle
    /// is drawn in the background's navy.
    #[test]
    fn shades_run_from_seed_to_newest() {
        assert_eq!(shade(0.0), SHADES[0]);
        assert_eq!(shade(1.0), SHADES[SHADES.len() - 1]);
        assert_eq!(shade(0.25), SHADES[1]);
        for i in 0..=100 {
            assert_ne!(shade(f64::from(i) / 100.0), NAVY);
        }
    }
}
