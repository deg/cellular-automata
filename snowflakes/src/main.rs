//! Snowflakes demo: Clifford Reiter's local model of snow crystal growth on a hex grid.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, GridView, Layout, NAVY, Pace, Playback, eframe, generations_per_frame, grouped,
};
use sim::{Crystal, PRESETS, Params};

fn main() {
    ca_core::run("Snowflakes", SnowflakeApp::new);
}

/// Vapour just short of freezing; water in between shades from `NAVY` (none) towards it.
const VAPOUR: Color32 = Color32::from_rgb(0x6c, 0x8c, 0xc4);
/// Ice just frozen, at the crystal's growing edge.
const NEW_ICE: Color32 = Color32::from_rgb(0xf4, 0xf1, 0xde);
/// The ice holding the most water, at the oldest parts of the crystal.
const OLD_ICE: Color32 = Color32::from_rgb(0x8f, 0xc1, 0xe3);

/// Room to grow: the radius of the circle held at the background, in hex rows as Reiter
/// measured it (see `sim`). His auxiliary page has tables of growth at these three radii.
const RADII: &[usize] = &[100, 200, 400];

struct SnowflakeApp {
    crystal: Crystal,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    /// The parameters for the next Restart; alpha and gamma also apply at once.
    params: Params,
    radius: usize,
    stroke: HexStroke,
    /// The cell under the pointer, as of the last frame, shown in the controls.
    pointer: Option<(usize, usize)>,
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
                // The project's demos show the cell under the pointer as the last row of
                // the readout, not as a tooltip on the grid.
                let pointer = shown.hovered.filter(|&(x, y)| self.crystal.is_inside(x, y));
                if pointer != self.pointer {
                    self.pointer = pointer;
                    ui.ctx().request_repaint(); // the controls were drawn before this
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
        // About 540 steps a second fit the default field at 60 frames a second.
        let mut playback = Playback::new(500, 5_000, "steps / second");
        // A crystal is best seen growing, from the first step.
        playback.running = true;
        Self {
            crystal: Crystal::new(radius, params),
            view,
            about: About::new(include_str!("../README.md")),
            playback,
            pace: Pace::default(),
            params,
            radius,
            stroke: HexStroke::default(),
            pointer: None,
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
        ca_core::speed_cap_note(ui, "field");
        ui.small("Restart grows a new crystal from a single seed, with the β below.");
        ui.small("Drag on the field to freeze extra cells.");
        ui.separator();

        ui.label(RichText::new("Crystals from Reiter's paper").strong());
        ui.horizontal_wrapped(|ui| {
            for preset in PRESETS {
                let chosen = self.params == preset.params;
                if ui.selectable_label(chosen, preset.name).clicked() {
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

        let crystal = &self.crystal;
        ca_core::readout(ui, |r| {
            r.row("step", grouped(crystal.generation));
            r.row("ice cells", grouped(crystal.ice_cells() as u64));
            r.row(
                "reach",
                format!("{:.0} of {}", crystal.reach(), crystal.radius()),
            );
            r.pointer(self.pointer.map(|(x, y)| {
                let w = crystal.water().get(x, y);
                let state = if w >= 1.0 { "ice" } else { "vapour" };
                format!("water {w:.4} ({state})")
            }));
        });
        if self.crystal.finished() {
            ui.small(
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
        mix(NAVY, VAPOUR, w.max(0.0))
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

/// Turns a held pointer into the cells to freeze each frame, like `ca_core::PaintStroke`
/// but drawing each frame's segment as a line of hex cells (see [`hex_line`]): a square-grid
/// line in axial coordinates steps diagonally between cells that don't touch, leaving gaps.
#[derive(Default)]
struct HexStroke {
    last: Option<(usize, usize)>,
}

impl HexStroke {
    fn cells(&mut self, pressed: Option<(usize, usize)>) -> Vec<(usize, usize)> {
        let Some(cell) = pressed else {
            self.last = None;
            return Vec::new();
        };
        let cells = hex_line(self.last.unwrap_or(cell), cell);
        self.last = Some(cell);
        cells
    }
}

/// The cells of a straight line between two hex cells in axial coordinates, each a
/// neighbour of the one before. The method of Red Blob Games' "Hexagonal Grids"
/// (<https://www.redblobgames.com/grids/hexagons/#line-drawing>): sample the segment at
/// N + 1 evenly spaced points, N the hex distance, and round each to the nearest hex in cube
/// coordinates (q, r, s = -q - r). The start is nudged by a tiny fixed amount, as that page
/// suggests, so that points exactly between two hexes always round the same way.
fn hex_line(a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
    let (aq, ar) = (a.0 as f64 + 1e-6, a.1 as f64 + 2e-6);
    let (dq, dr) = (b.0 as f64 - a.0 as f64, b.1 as f64 - a.1 as f64);
    let n = hex_distance(a, b);
    (0..=n)
        .map(|i| {
            let t = if n == 0 { 0.0 } else { i as f64 / n as f64 };
            let (q, r) = cube_round(aq + dq * t, ar + dr * t);
            (q as usize, r as usize)
        })
        .collect()
}

/// The hex distance between two cells in axial coordinates.
fn hex_distance(a: (usize, usize), b: (usize, usize)) -> usize {
    let dq = a.0 as isize - b.0 as isize;
    let dr = a.1 as isize - b.1 as isize;
    (dq.abs() + dr.abs() + (dq + dr).abs()) as usize / 2
}

/// The hex containing fractional axial point (q, r): round all three cube coordinates, then
/// recompute the one that moved most from the other two, so they still sum to 0.
fn cube_round(q: f64, r: f64) -> (i64, i64) {
    let s = -q - r;
    let (mut rq, mut rr, rs) = (q.round(), r.round(), s.round());
    let (eq, er, es) = ((rq - q).abs(), (rr - r).abs(), (rs - s).abs());
    if eq > er && eq > es {
        rq = -rr - rs;
    } else if er > es {
        rr = -rq - rs;
    }
    (rq as i64, rr as i64)
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

    /// A dragged stroke must be solid: from any cell to any other, every cell of the line
    /// touches the one before, it starts and ends where asked, and it takes the fewest cells
    /// possible (hex distance + 1). Checked from a centre cell to every cell within 12, which
    /// covers all directions, including the six where a square-grid line left gaps.
    #[test]
    fn hex_lines_are_gap_free() {
        let a = (20, 20);
        for y in 8..=32 {
            for x in 8..=32 {
                let b = (x, y);
                let line = hex_line(a, b);
                assert_eq!(
                    (line[0], *line.last().unwrap()),
                    (a, b),
                    "ends of {a:?} to {b:?}"
                );
                assert_eq!(
                    line.len(),
                    hex_distance(a, b) + 1,
                    "length of {a:?} to {b:?}"
                );
                for pair in line.windows(2) {
                    assert_eq!(
                        hex_distance(pair[0], pair[1]),
                        1,
                        "{a:?} to {b:?}: {pair:?}"
                    );
                }
            }
        }
    }

    /// A stroke continues from where the last frame's segment ended, and a release ends it.
    #[test]
    fn a_stroke_joins_frames_and_ends_on_release() {
        let mut s = HexStroke::default();
        assert_eq!(s.cells(Some((5, 5))), vec![(5, 5)]);
        assert_eq!(s.cells(Some((7, 5))), vec![(5, 5), (6, 5), (7, 5)]);
        assert!(s.cells(None).is_empty());
        assert_eq!(s.cells(Some((1, 1))), vec![(1, 1)]);
    }

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
        assert_eq!(colour(0.0, 5.0), NAVY);
    }
}
