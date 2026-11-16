//! Snowflakes demo: Clifford Reiter's local model of snow crystal growth on a hex grid.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, GridView, Layout, Pace, PaintStroke, Playback, eframe, generations_per_frame,
};
use sim::{Crystal, PRESETS, Params};

fn main() {
    ca_core::run("Snowflakes", SnowflakeApp::new);
}

/// No water: the project's navy.
const DRY: Color32 = Color32::from_rgb(0x14, 0x18, 0x30);
/// Vapour just short of freezing; water in between shades from `DRY` towards it.
const VAPOUR: Color32 = Color32::from_rgb(0x6c, 0x8c, 0xc4);
/// Ice just frozen, at the crystal's growing edge.
const NEW_ICE: Color32 = Color32::from_rgb(0xf4, 0xf1, 0xde);
/// The ice holding the most water, at the oldest parts of the crystal.
const OLD_ICE: Color32 = Color32::from_rgb(0x8f, 0xc1, 0xe3);

/// Room to grow: the radius of the circle held at the background, in cells. Reiter showed
/// growth limited to radii 100, 200 and 400 on his auxiliary page.
const RADII: &[usize] = &[100, 200, 300];

struct SnowflakeApp {
    crystal: Crystal,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    /// The parameters for the next Restart; alpha and gamma also apply at once.
    params: Params,
    radius: usize,
    stroke: PaintStroke,
    dirty: bool,
}

impl eframe::App for SnowflakeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            let most = self.crystal.most_water();
            self.view.set_image_cropped(
                ui.ctx(),
                self.crystal.water(),
                |w| colour(w, most),
                |_| false,
            );
            self.dirty = false;
        }

        // The far field is held at the background, so filling the panel with its colour
        // hides the edges of the picture (the hex grid's corners are transparent).
        let background = colour(self.crystal.beta(), 1.0);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(background))
            .show(ui, |ui| {
                let shown = self.view.show(ui);
                for (x, y) in self.stroke.cells(shown.pressed) {
                    self.crystal.freeze(x, y);
                    self.dirty = true;
                }
                if let Some((x, y)) = shown.hovered
                    && self.crystal.is_inside(x, y)
                {
                    let w = self.crystal.water().get(x, y);
                    let state = if w >= 1.0 { "ice" } else { "vapour" };
                    shown
                        .response
                        .on_hover_text(format!("water {w:.4} ({state})"));
                }
            });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl SnowflakeApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let params = PRESETS[1].params;
        let radius = RADII[1];
        let mut view = GridView::new("snowflake");
        view.set_layout(Layout::Hex);
        let mut playback = Playback::new(1_000, 5_000, "steps / second");
        playback.running = true;
        Self {
            crystal: Crystal::new(radius, params),
            view,
            about: About::new(include_str!("../README.md")),
            playback,
            pace: Pace::default(),
            params,
            radius,
            stroke: PaintStroke::default(),
            dirty: true,
        }
    }

    fn restart(&mut self) {
        self.crystal = Crystal::new(self.radius, self.params);
        self.dirty = true;
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Snowflakes");
        ui.label("Each hex cell holds some water; at 1 it is ice. Cells that are ice or touch ice keep their water and gain γ each step. Water elsewhere diffuses, and what reaches the crystal freezes onto it.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small("On the bigger fields the top speeds are out of reach: each frame does at most about a million cell updates.");
        ui.small("Drag on the field to freeze extra cells.");
        ui.separator();

        ui.label(RichText::new("Crystals from Reiter's paper").strong());
        ui.horizontal_wrapped(|ui| {
            for preset in PRESETS {
                let chosen = self.params == preset.params;
                if ui
                    .selectable_label(chosen, preset.name)
                    .on_hover_text(preset.note)
                    .clicked()
                {
                    self.params = preset.params;
                    self.restart();
                }
            }
        });
        if let Some(preset) = PRESETS.iter().find(|p| p.params == self.params) {
            ui.small(preset.note);
        }
        ui.separator();

        ui.label("Background vapour β");
        ui.add(egui::Slider::new(&mut self.params.beta, 0.0..=0.95));
        if self.params.beta != self.crystal.beta() {
            ui.small("The background changes at the next Restart.");
        }
        ui.label("Water added to the crystal each step γ");
        ui.add(
            egui::Slider::new(&mut self.params.gamma, 0.0..=1.0)
                .logarithmic(true)
                .smallest_positive(1e-5)
                .custom_formatter(|v, _| plain(v)),
        );
        ui.label("Diffusion α (1 is Reiter's standard)");
        ui.add(egui::Slider::new(&mut self.params.alpha, 0.0..=2.7).max_decimals(3));
        self.crystal.params.alpha = self.params.alpha;
        self.crystal.params.gamma = self.params.gamma;
        ui.small("γ and α take effect at once, so a crystal can change form as it grows.");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("Room to grow, radius:");
            for &radius in RADII {
                if ui
                    .selectable_label(self.radius == radius, radius.to_string())
                    .clicked()
                    && self.radius != radius
                {
                    self.radius = radius;
                    self.restart();
                }
            }
        });
        ui.separator();

        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            ui.label("step");
            ui.label(self.crystal.generation.to_string());
            ui.end_row();
            ui.label("ice cells");
            ui.label(self.crystal.ice_cells().to_string());
            ui.end_row();
            ui.label("reach");
            ui.label(format!(
                "{:.0} of {}",
                self.crystal.reach(),
                self.crystal.radius()
            ));
            ui.end_row();
        });
        if self.crystal.finished() {
            ui.label(
                "Fully grown: the ice has reached the edge of its field. Restart to grow another.",
            );
        }
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.restart();
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => {
                let cap = generations_per_frame(self.crystal.active_cells(), 100);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            if !self.crystal.step() {
                self.playback.running = false;
                break;
            }
            self.dirty = true;
        }
    }
}

/// A cell's colour from its water `w`, given the most water in any cell. Vapour shades from
/// navy (none) towards pale blue as it nears freezing; ice is cream-white where it has just
/// frozen, turning icy blue where it has gathered the most water.
fn colour(w: f64, most: f64) -> Color32 {
    if w < 1.0 {
        mix(DRY, VAPOUR, w.max(0.0))
    } else {
        let t = if most > 1.0 {
            (w - 1.0) / (most - 1.0)
        } else {
            0.0
        };
        mix(NEW_ICE, OLD_ICE, t.clamp(0.0, 1.0).sqrt())
    }
}

fn mix(a: Color32, b: Color32, t: f64) -> Color32 {
    let m = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// A number in plain decimals with no trailing zeros (0.0001, not 1e-4 or 0.000100), so
/// the presets' small gammas can be told apart.
fn plain(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let s = format!("{v:.5}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "0" {
        format!("{v:.1e}")
    } else {
        s.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The gamma slider must tell the paper's values apart: 0.0001, 0.001, 0.01, 0.05 and 1.
    #[test]
    fn small_gammas_read_plainly() {
        let shown = [0.0, 0.0001, 0.001, 0.01, 0.05, 1.0, 0.00015].map(plain);
        assert_eq!(
            shown,
            ["0", "0.0001", "0.001", "0.01", "0.05", "1", "0.00015"]
        );
    }

    /// Vapour and ice must never share a colour, or the crystal's edge would vanish: the
    /// wettest vapour is well darker than the oldest ice.
    #[test]
    fn ice_stands_out_from_vapour() {
        let lum = |c: Color32| u32::from(c.r()) + u32::from(c.g()) + u32::from(c.b());
        let wettest = colour(0.9999, 5.0);
        for w in [1.0, 2.0, 5.0] {
            assert!(lum(colour(w, 5.0)) > lum(wettest) + 100, "water {w}");
        }
        assert_eq!(colour(0.0, 5.0), DRY);
    }
}
