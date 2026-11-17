//! Forest-fire demo: trees grow, lightning strikes, and fires of every size burn out.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, Grid, GridView, NAVY, Pace, PaintStroke, Playback, Plot, Series, Style, eframe,
    grouped, readout, speed_cap_note,
};
use sim::{Cell, Forest};

fn main() {
    ca_core::run("Forest fire", FireApp::new);
}

const TREE: Color32 = Color32::from_rgb(0x2a, 0x9d, 0x8f); // teal
const FLAME: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a); // gold
const SCAR: Color32 = Color32::from_rgb(0xe7, 0x6f, 0x51); // coral
const DOTS: Color32 = Color32::from_rgb(0x5a, 0xb4, 0xff); // sky blue, as elsewhere
const LINE: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

/// Trees grown per lightning, θ = p/f: the values in Grassberger's Table 1 (2002), so the
/// mean density can be compared with his.
const THETAS: &[u32] = &[125, 250, 500, 1_000, 2_000, 4_000, 8_000, 16_000];

const SIZES: &[(usize, usize)] = &[(320, 200), (640, 400), (1280, 800)];

/// The largest θ a world of `cells` sites is offered. The biggest fires grow faster than θ
/// (s_max ∼ θ^λ, λ about 1.1), and must stay well short of the whole world, which would
/// otherwise cut them off; Grassberger's lattices had 2^20 sites or more for θ ≥ 500.
fn max_theta(cells: usize) -> u32 {
    (cells / 64) as u32
}

/// τ in two dimensions, from Clar, Drossel and Schwabl's Table 1 (1996): 2.14(3).
const TAU: f64 = 2.14;

/// Sites visited per frame (growth attempts and burning trees). A unit took 8.4 ns natively
/// (3,000 lightnings at p/f = 1,000 on 640 x 400, on a busy machine), so this is about 8 ms.
const WORK_PER_FRAME: usize = 1_000_000;

struct FireApp {
    forest: Forest,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    size: (usize, usize),
    seed: u64,
    stroke: PaintStroke,
    hovered: Option<(usize, usize)>,
    /// Lightnings a second, smoothed: scars fade over about half a second of them.
    rate: f64,
    /// The picture, reused from frame to frame.
    image: Grid<Color32>,
    dirty: bool,
}

impl eframe::App for FireApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            self.draw();
            self.view.set_image(ui.ctx(), &self.image, |c| c);
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            if shown.hovered != self.hovered {
                // Redraw the "under the pointer" row now, not at the next event.
                self.hovered = shown.hovered;
                ui.ctx().request_repaint();
            }
            for (x, y) in self.stroke.cells(shown.pressed) {
                self.forest.light(x, y);
                self.dirty = true;
            }
        });

        if self.playback.running || self.dirty || self.forest.is_burning() {
            ui.ctx().request_repaint();
        }
    }
}

impl FireApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let mut app = Self {
            forest: Forest::new(size.0, size.1, 1_000, 1),
            view: GridView::new("forest"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(1_000, 1_000_000, "steps / second"),
            pace: Pace::default(),
            size,
            seed: 1,
            stroke: PaintStroke::default(),
            hovered: None,
            rate: 0.0,
            image: Grid::new(size.0, size.1, NAVY),
            dirty: true,
        };
        app.restart();
        // A forest growing from empty, and its first fire sweeping most of it away, is the
        // opening worth seeing.
        app.playback.running = true;
        app
    }

    /// An empty forest again, with the current θ and size, and new random choices.
    fn restart(&mut self) {
        let theta = self.forest.theta();
        self.seed += 1;
        self.forest = Forest::new(self.size.0, self.size.1, theta, self.seed);
        self.image = Grid::new(self.size.0, self.size.1, NAVY);
        self.dirty = true;
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Forest fire");
        ui.label("Trees grow, lightning strikes now and then, and fire spreads from tree to neighbouring tree. Fires of every size break out.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        speed_cap_note(ui, "world");
        ui.small("A step burns one ring of a fire, or grows trees and strikes once. Restart plants an empty forest and empties the chart.");
        ui.separator();

        ui.label(RichText::new("Trees grown per lightning, p/f").strong());
        ui.horizontal_wrapped(|ui| {
            for &theta in THETAS {
                if ui
                    .selectable_label(self.forest.theta() == theta, grouped(u64::from(theta)))
                    .clicked()
                    && self.forest.theta() != theta
                {
                    self.forest.set_theta(theta);
                    if max_theta(self.size.0 * self.size.1) < theta {
                        // The smallest world big enough for it.
                        self.size = *SIZES
                            .iter()
                            .find(|s| max_theta(s.0 * s.1) >= theta)
                            .unwrap_or(&SIZES[SIZES.len() - 1]);
                        self.restart();
                    }
                }
            }
        });
        ui.small("p: the chance a site grows a tree; f: that lightning strikes it. Fires burn out before anything else happens, so only p/f matters. A change empties the chart.");
        ui.separator();

        self.chart(ui);
        ui.separator();

        ui.label(RichText::new("Light a fire").strong());
        ui.small("Click or drag over trees to set them alight. Fires lit by hand are left out of the chart.");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{} x {}", size.0, size.1);
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    let most = max_theta(size.0 * size.1);
                    if self.forest.theta() > most {
                        let theta = *THETAS.iter().rfind(|&&t| t <= most).unwrap_or(&THETAS[0]);
                        self.forest.set_theta(theta);
                    }
                    self.restart();
                }
            }
        });
        ui.small("The world wraps round at the edges. A bigger p/f needs a bigger world.");
        ui.separator();

        let stats = &self.forest.stats;
        let grid = self.forest.grid();
        readout(ui, |r| {
            r.row("lightning", grouped(self.forest.lightnings));
            r.row("tree density", format!("{:.3}", self.forest.density()));
            r.row(
                "mean density",
                match stats.mean_density() {
                    Some(d) if stats.skip == 0 => format!("{d:.4}"),
                    _ => "-".to_string(),
                },
            );
            r.row("fire counted", grouped(stats.fires));
            r.row("largest fire", format!("{} trees", grouped(stats.largest)));
            r.pointer(self.hovered.map(|(x, y)| {
                let what = match grid.get(x, y) {
                    Cell::Empty => "empty",
                    Cell::Tree => "tree",
                    Cell::Burning => "burning",
                };
                format!("({x}, {y}): {what}")
            }));
        });
    }

    /// The fire-size distribution so far, on log-log axes, with the power law the
    /// simulations of the 1990s reported.
    fn chart(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Fire sizes").strong());
        let stats = &self.forest.stats;
        let points = stats.distribution();
        let line = reference_line(&points, self.forest.theta());
        let mut series = vec![Series {
            points: &points,
            colour: DOTS,
            style: Style::Dots,
        }];
        if let Some(line) = &line {
            series.push(Series {
                points: line,
                colour: LINE,
                style: Style::Line,
            });
        }
        ca_core::plot(
            ui,
            &Plot {
                x_label: "trees burnt, s",
                y_label: "share of fires, P(s)",
                log_x: true,
                log_y: true,
                x_range: None,
                y_range: None,
                series,
            },
            120.0,
        );
        if stats.skip > 0 {
            ui.small(format!(
                "The chart starts in {} lightnings, once the forest planted from empty has burnt and grown back many times over: its first fires are far bigger than later ones.",
                grouped(stats.skip)
            ));
        } else {
            ui.small(format!(
                "Dots: the share of fires burning s trees. Line: the power law, slope 1 \u{2212} τ, with τ = {TAU} as measured in the 1990s."
            ));
        }
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.restart();
        }
        let before = self.forest.lightnings;
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => self.pace.steps(per_second as f64, dt, usize::MAX),
        };
        let mut work = 0;
        for _ in 0..steps {
            work += self.forest.step();
            if work >= WORK_PER_FRAME {
                break;
            }
        }
        if dt > 0.0 {
            let now = (self.forest.lightnings - before) as f64 / dt;
            self.rate += (now - self.rate) * (dt * 2.0).min(1.0);
        }
        self.dirty |= steps > 0;
    }

    /// The picture: trees, the burning front, and the scars of recent fires fading back to
    /// the background over about half a second of lightnings. At least the last few, but at
    /// most as many as burn about a quarter of the world between them (each burns about
    /// 0.6 p/f trees), or at top speed every empty site is a scar.
    fn draw(&mut self) {
        let most = 0.25 * self.image.len() as f64 / f64::from(self.forest.theta());
        let fade = (self.rate * 0.5).min(most).max(4.0);
        let now = self.forest.lightnings;
        let grid = self.forest.grid();
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let colour = match grid.get(x, y) {
                    Cell::Tree => TREE,
                    Cell::Burning => FLAME,
                    Cell::Empty => match self.forest.burnt_at(x, y) {
                        0 => NAVY,
                        at => {
                            let age = now.saturating_sub(at) as f64 / fade;
                            if age >= 1.0 {
                                NAVY
                            } else {
                                mix(SCAR, NAVY, age as f32)
                            }
                        }
                    },
                };
                self.image.set(x, y, colour);
            }
        }
    }
}

/// `a` moved a fraction `t` of the way to `b`.
fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// The line P(s) ∝ s^(1 − τ) across the measured sizes, placed through the points between
/// s = 3 and θ/10 (below the cut-off, where the papers see the power law), by their mean in
/// log-log space. None until there are such points.
fn reference_line(points: &[(f64, f64)], theta: u32) -> Option<[(f64, f64); 2]> {
    let slope = 1.0 - TAU;
    let fit: Vec<f64> = points
        .iter()
        .filter(|&&(s, _)| s >= 3.0 && s <= f64::from(theta) / 10.0)
        .map(|&(s, p)| p.ln() - slope * s.ln())
        .collect();
    if fit.is_empty() {
        return None;
    }
    let c = fit.iter().sum::<f64>() / fit.len() as f64;
    let lo = points.first()?.0;
    let hi = points.last()?.0;
    let at = |s: f64| (s, (c + slope * s.ln()).exp());
    Some([at(lo), at(hi)])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each θ offered fits some world, and the default fits the default world.
    #[test]
    fn every_theta_has_a_world_big_enough() {
        for &t in THETAS {
            assert!(SIZES.iter().any(|s| max_theta(s.0 * s.1) >= t), "{t}");
        }
        assert!(max_theta(SIZES[1].0 * SIZES[1].1) >= 1_000);
    }

    /// The reference line has the slope 1 − τ, and runs through points that lie on it.
    #[test]
    fn the_reference_line_has_the_papers_slope() {
        let points: Vec<(f64, f64)> = [1.0, 4.0, 16.0, 64.0, 256.0]
            .iter()
            .map(|&s: &f64| (s, 0.3 * s.powf(1.0 - TAU)))
            .collect();
        let [a, b] = reference_line(&points, 1_000).unwrap();
        let slope = (b.1.ln() - a.1.ln()) / (b.0.ln() - a.0.ln());
        assert!((slope - (1.0 - TAU)).abs() < 1e-12);
        assert!((a.1 - 0.3).abs() < 1e-9);
        assert!(reference_line(&points[..1], 1_000).is_none());
    }
}
