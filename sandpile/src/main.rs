//! Abelian sandpile demo: drop grains at the centre and watch them topple into a fractal.

mod drip;
mod sim;

use ca_core::egui::{self, Color32, RichText, Stroke};
use ca_core::{About, Advance, GridView, Playback, eframe};
use drip::DripPoints;
use sim::{Boundary, ORTHOGONAL, Sandpile};

fn main() {
    ca_core::run("Abelian sandpile", SandpileApp::new);
}

/// Colours for heights 0, 1, 2 and 3. A cell at or over the threshold is drawn white.
const PALETTE: [Color32; 4] = [
    Color32::from_rgb(0x14, 0x18, 0x30),
    Color32::from_rgb(0x2a, 0x9d, 0x8f),
    Color32::from_rgb(0xe9, 0xc4, 0x6a),
    Color32::from_rgb(0xe7, 0x6f, 0x51),
];
const UNSTABLE: Color32 = Color32::WHITE;
/// Rings marking the drip points: a colour the heights never use.
const DRIP_MARK: Color32 = Color32::from_rgb(0xff, 0x3f, 0xb4);

/// A small first grid: it grows as the pile does, so there is no need to guess a size.
const START_SIDE: usize = 101;

struct SandpileApp {
    pile: Sandpile,
    view: GridView,
    about: About,
    playback: Playback,
    /// The edge chosen in the panel; it applies from the next Restart.
    boundary: Boundary,
    /// Grains added to the centre by the "Drop" button.
    drop_amount: u32,
    /// While running, add `drip_amount` grains to every drip point whenever the pile is
    /// stable. A click on the grid also drops `drip_amount` grains, once.
    drip: bool,
    drip_amount: u32,
    drip_points: DripPoints,
    /// The pile changed since its picture was last uploaded.
    dirty: bool,
}

impl eframe::App for SandpileApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("controls")
            .resizable(false)
            .default_size(300.0)
            .show(ui, |ui| self.controls(ui));
        self.about.show(ui);

        self.advance();
        if self.dirty {
            let threshold = self.pile.threshold();
            self.view.set_image(ui.ctx(), self.pile.grid(), |h| {
                if h >= threshold {
                    UNSTABLE
                } else {
                    PALETTE[h as usize % PALETTE.len()]
                }
            });
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            if let Some((x, y)) = shown.clicked {
                if ui.input(|i| i.modifiers.shift) {
                    let (cx, cy) = self.pile.center();
                    let at = (x as isize - cx as isize, y as isize - cy as isize);
                    // Within about 8 pixels counts as clicking a point, however small cells are.
                    let tolerance = (8.0 / shown.cell_size()).ceil().max(1.0) as isize;
                    self.drip_points.toggle(at, tolerance);
                } else {
                    self.dirty |= self.pile.add(x, y, self.drip_amount);
                }
            }
            self.mark_drip_points(ui, &shown);
            if let Some((x, y)) = shown.hovered {
                shown
                    .response
                    .on_hover_text(format!("({x}, {y}): {}", self.pile.grid().get(x, y)));
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl SandpileApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let boundary = Boundary::Grow;
        Self {
            pile: new_pile(boundary),
            view: GridView::new("sandpile"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(2_000, 100_000, "thousand cells / frame"),
            boundary,
            drop_amount: 100_000,
            drip: true,
            drip_amount: 1,
            drip_points: DripPoints::centre(),
            dirty: true,
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Abelian sandpile");
        ui.label(format!(
            "A cell holding {} grains topples, sending one to each neighbour.",
            self.pile.threshold()
        ));
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small(
            "A step is one sweep across the pile, toppling every unstable cell it meets; \
             on a settled pile it drips first. Speed caps the cells swept per frame, \
             so a big pile still redraws smoothly.",
        );
        ui.separator();

        ui.checkbox(&mut self.drip, "Drip while running");
        ui.add(
            egui::Slider::new(&mut self.drip_amount, 1..=10_000)
                .logarithmic(true)
                .text("grains per drip"),
        );
        ui.horizontal(|ui| {
            let n = self.drip_points.len();
            ui.label(format!("{n} drip point{}", if n == 1 { "" } else { "s" }));
            if ui.button("Centre only").clicked() {
                self.drip_points.reset_to_centre();
            }
        });
        if self.drip_points.len() == 0 {
            ui.small(
                RichText::new(
                    "No drip points: Step and Drip do nothing until you shift-click the grid \
                     or press Centre only.",
                )
                .color(Color32::YELLOW),
            );
        }
        ui.small(
            "Shift-click the grid to add a drip point, or to remove one. Each drip feeds every \
             point at once, and waits until the last avalanche has stopped.",
        );
        ui.separator();

        ui.horizontal(|ui| {
            let fits = self.pile.can_add(self.drop_amount);
            let drop = ui
                .add_enabled(fits, egui::Button::new("Drop"))
                .on_disabled_hover_text("The pile is full: it holds at most 4,294,967,295 grains.");
            if drop.clicked() {
                self.drop_at_centre(self.drop_amount);
                // Drip would keep adding grains after the drop settles, so the count would
                // creep past what was dropped.
                self.drip = false;
                self.playback.running = true;
            }
            ui.add(
                egui::DragValue::new(&mut self.drop_amount)
                    .range(1..=50_000_000)
                    .speed(1_000),
            );
            ui.label("grains at the centre");
        });
        ui.small(
            "100,000 grains settle in seconds at full speed; 1,000,000 take a few minutes. \
             Click the grid to drop a drip's worth of grains there.",
        );
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Edge:");
            ui.radio_value(&mut self.boundary, Boundary::Grow, "grow the grid");
            ui.radio_value(&mut self.boundary, Boundary::Sink, "grains fall off");
        });
        if self.boundary != self.pile.boundary() {
            ui.small("Takes effect on Restart.");
        }
        ui.separator();

        let side = self.pile.grid().width();
        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            stat(ui, "grains", self.pile.grains);
            stat(ui, "lost off the edge", self.pile.lost);
            stat(ui, "topples", self.pile.topples);
            stat(ui, "sweeps", self.pile.sweeps);
            ui.label("state");
            ui.label(if self.pile.is_stable() {
                "stable"
            } else {
                "toppling"
            });
            ui.end_row();
            ui.label("grid");
            ui.label(format!("{side} x {side}"));
            ui.end_row();
        });
        if side >= sim::MAX_SIDE {
            ui.small(
                RichText::new("The grid is at its largest; grains now fall off the edge.")
                    .color(Color32::YELLOW),
            );
        }
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            for (h, &color) in PALETTE.iter().enumerate() {
                ui.label(RichText::new("■").color(color));
                ui.label(h.to_string());
            }
            ui.label(RichText::new("■").color(UNSTABLE));
            ui.label("toppling");
        });
    }

    /// Apply this frame's Restart, Step or Run to the pile.
    fn advance(&mut self) {
        if self.playback.take_restart() {
            self.pile = new_pile(self.boundary);
            self.dirty = true;
        }
        let sweeps = match self.playback.advance() {
            Advance::Idle => return,
            Advance::Step => {
                // A settled pile has nothing to topple, so a Step drips first, whether or
                // not dripping is on: that is the only way a Step can do something.
                if self.pile.is_stable() {
                    self.drip_once();
                }
                1
            }
            Advance::Run(thousand_cells) => {
                if self.drip && self.pile.is_stable() {
                    self.drip_once();
                }
                // Budget by cells swept, not by sweeps: a sweep costs the pile's whole area,
                // which grows with the pile, and a fixed sweep count would freeze big piles.
                (thousand_cells * 1_000 / self.pile.sweep_area().max(1)).max(1)
            }
        };
        // A sweep can grow the grid without toppling anything. The picture must still be
        // redrawn, or clicks would map onto the old, smaller grid and land in the wrong cell.
        let side = self.pile.grid().width();
        if self.pile.relax(sweeps) > 0 || self.pile.grid().width() != side {
            self.dirty = true;
        }
    }

    /// Add `drip_amount` grains to every drip point.
    fn drip_once(&mut self) {
        for &(dx, dy) in self.drip_points.points() {
            self.dirty |= self.pile.add_relative(dx, dy, self.drip_amount);
        }
    }

    fn drop_at_centre(&mut self, n: u32) {
        self.dirty |= self.pile.add_relative(0, 0, n);
    }

    /// Ring each drip point that lies on the grid, sized to stay visible on any zoom.
    fn mark_drip_points(&self, ui: &egui::Ui, shown: &ca_core::GridResponse) {
        let (cx, cy) = self.pile.center();
        let radius = (shown.cell_size() * 0.7).max(5.0);
        for &(dx, dy) in self.drip_points.points() {
            let Some((x, y)) = self.pile.grid().offset(cx, cy, dx, dy) else {
                continue; // off a sink grid after a Restart; its grains count as lost
            };
            let centre = shown.cell_rect(x, y).center();
            let painter = ui.painter();
            painter.circle_stroke(centre, radius + 1.5, Stroke::new(2.0, Color32::BLACK));
            painter.circle_stroke(centre, radius, Stroke::new(2.0, DRIP_MARK));
        }
    }
}

fn new_pile(boundary: Boundary) -> Sandpile {
    Sandpile::new(START_SIDE, &ORTHOGONAL, boundary)
}

fn stat(ui: &mut egui::Ui, name: &str, value: u64) {
    ui.label(name);
    ui.label(value.to_string());
    ui.end_row();
}
