//! Brian's Brain demo: neuron-like cells that fire, rest and recover.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{About, Advance, GridView, Pace, Playback, Rng, eframe, line};
use sim::{Brain, Cell};

fn main() {
    ca_core::run("Brian's Brain", BrainApp::new);
}

const READY: Color32 = Color32::from_rgb(0x14, 0x18, 0x30);
const FIRING: Color32 = Color32::from_rgb(0xf4, 0xf1, 0xde);
const REFRACTORY: Color32 = Color32::from_rgb(0x3a, 0x6e, 0xa5);

const SIZES: &[(usize, usize)] = &[(160, 100), (320, 200), (640, 400)];

struct BrainApp {
    brain: Brain,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    rng: Rng,
    size: (usize, usize),
    density: f64,
    /// Fraction of the world's width and height the random seed fills, centred.
    seed_extent: f64,
    last_paint: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for BrainApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("controls")
            .resizable(false)
            .default_size(300.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
            });
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            self.view
                .set_image(ui.ctx(), self.brain.grid(), |c| match c {
                    Cell::Ready => READY,
                    Cell::Firing => FIRING,
                    Cell::Refractory => REFRACTORY,
                });
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            match shown.pressed {
                Some(cell) => {
                    let state = if ui.input(|i| i.modifiers.shift) {
                        Cell::Ready
                    } else {
                        Cell::Firing
                    };
                    for (x, y) in line(self.last_paint.unwrap_or(cell), cell) {
                        self.brain.set(x, y, state);
                    }
                    self.last_paint = Some(cell);
                    self.dirty = true;
                }
                None => self.last_paint = None,
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl BrainApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let mut app = Self {
            brain: Brain::new(size.0, size.1, true),
            view: GridView::new("brain"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(20, 1_000, "generations / second"),
            pace: Pace::default(),
            rng: Rng::new(1987),
            size,
            density: 0.3,
            seed_extent: 0.25,
            last_paint: None,
            dirty: true,
        };
        app.random_seed();
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Brian's Brain");
        ui.label("Each cell is ready, firing or recovering. A ready cell fires when exactly 2 of its 8 neighbours are firing; a firing cell then recovers for one generation; then it is ready again.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small(
            "Restart clears the world. Drag on the grid to draw firing cells; shift-drag to erase.",
        );
        ui.separator();

        ui.label(RichText::new("Start").strong());
        if ui.button("Random seed in the middle").clicked() {
            self.brain = Brain::new(self.size.0, self.size.1, self.brain.wrap);
            self.random_seed();
        }
        ui.add(egui::Slider::new(&mut self.seed_extent, 0.05..=1.0).text("seed size"));
        ui.add(egui::Slider::new(&mut self.density, 0.05..=0.6).text("density"));
        if ui
            .button("Add four gliders")
            .on_hover_text(
                "A firing pair with a recovering pair behind it moves one cell per generation.",
            )
            .clicked()
        {
            self.add_gliders();
        }
        ui.separator();

        ui.checkbox(&mut self.brain.wrap, "Wrap around the edges");
        ui.horizontal(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{} x {}", size.0, size.1);
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.brain = Brain::new(size.0, size.1, self.brain.wrap);
                    self.random_seed();
                }
            }
        });
        ui.separator();

        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            ui.label("generation");
            ui.label(self.brain.generation.to_string());
            ui.end_row();
            ui.colored_label(FIRING, "firing");
            ui.label(self.brain.count(Cell::Firing).to_string());
            ui.end_row();
            ui.colored_label(REFRACTORY, "recovering");
            ui.label(self.brain.count(Cell::Refractory).to_string());
            ui.end_row();
        });
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.brain = Brain::new(self.size.0, self.size.1, self.brain.wrap);
            self.dirty = true;
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => self.pace.steps(per_second as f64, dt, 50),
        };
        for _ in 0..steps {
            self.brain.step();
        }
        self.dirty |= steps > 0;
    }

    /// Fill a centred patch with random firing cells: a small seed shows the patterns
    /// spreading outward, which a full-screen fill hides.
    fn random_seed(&mut self) {
        let (w, h) = self.size;
        let (pw, ph) = (
            (w as f64 * self.seed_extent) as usize,
            (h as f64 * self.seed_extent) as usize,
        );
        let (x0, y0) = ((w - pw) / 2, (h - ph) / 2);
        for y in y0..y0 + ph {
            for x in x0..x0 + pw {
                if self.rng.chance(self.density) {
                    self.brain.set(x, y, Cell::Firing);
                }
            }
        }
        self.dirty = true;
    }

    /// Four gliders around the centre, heading up, down, left and right.
    fn add_gliders(&mut self) {
        let (cx, cy) = (self.size.0 / 2, self.size.1 / 2);
        let d = 20;
        // (firing cells, recovering cells) for each glider; recovering cells sit behind.
        let gliders = [
            (
                [(cx, cy - d), (cx + 1, cy - d)],
                [(cx, cy - d + 1), (cx + 1, cy - d + 1)],
            ),
            (
                [(cx, cy + d), (cx + 1, cy + d)],
                [(cx, cy + d - 1), (cx + 1, cy + d - 1)],
            ),
            (
                [(cx - d, cy), (cx - d, cy + 1)],
                [(cx - d + 1, cy), (cx - d + 1, cy + 1)],
            ),
            (
                [(cx + d, cy), (cx + d, cy + 1)],
                [(cx + d - 1, cy), (cx + d - 1, cy + 1)],
            ),
        ];
        for (firing, recovering) in gliders {
            for (x, y) in firing {
                self.brain.set(x, y, Cell::Firing);
            }
            for (x, y) in recovering {
                self.brain.set(x, y, Cell::Refractory);
            }
        }
        self.dirty = true;
    }
}
