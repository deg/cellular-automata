//! Wireworld demo: draw wires, add electrons, build circuits.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{About, Advance, GridView, Pace, PaintStroke, Playback, eframe};
use sim::{Cell, PATTERNS, Wireworld};

fn main() {
    ca_core::run("Wireworld", WireworldApp::new);
}

const EMPTY: Color32 = Color32::from_rgb(0x10, 0x10, 0x14);
const WIRE: Color32 = Color32::from_rgb(0xd9, 0x8c, 0x2b);
const HEAD: Color32 = Color32::from_rgb(0x5a, 0xb4, 0xff);
const TAIL: Color32 = Color32::from_rgb(0xe0, 0x45, 0x3a);

const SIZES: &[(usize, usize)] = &[(80, 50), (120, 75), (200, 125)];

struct WireworldApp {
    world: Wireworld,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    size: (usize, usize),
    brush: Cell,
    pattern: usize,
    stroke: PaintStroke,
    dirty: bool,
}

impl eframe::App for WireworldApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            self.view.set_image(ui.ctx(), self.world.grid(), colour);
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            let cells = self.stroke.cells(shown.pressed);
            if !cells.is_empty() {
                let brush = if ui.input(|i| i.modifiers.shift) {
                    Cell::Empty
                } else {
                    self.brush
                };
                for (x, y) in cells {
                    self.world.set(x, y, brush);
                }
                self.dirty = true;
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl WireworldApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[0];
        let mut app = Self {
            world: Wireworld::new(size.0, size.1),
            view: GridView::new("wireworld"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(8, 500, "generations / second"),
            pace: Pace::default(),
            size,
            brush: Cell::Conductor,
            pattern: 2, // the OR gate shows the most at once
            stroke: PaintStroke::default(),
            dirty: true,
        };
        app.place_pattern();
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Wireworld");
        ui.label("Wires carry electrons. Each generation an electron's head moves on and leaves a tail behind; a wire cell becomes a head if 1 or 2 of its 8 neighbours are heads.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small("Restart clears the board.");
        if ui
            .button("Remove all electrons")
            .on_hover_text("Turn every head and tail back into wire, keeping the circuit.")
            .clicked()
        {
            self.world.remove_electrons();
            self.dirty = true;
        }
        ui.separator();

        ui.label(RichText::new("Draw").strong());
        ui.horizontal(|ui| {
            for (cell, name) in [
                (Cell::Conductor, "wire"),
                (Cell::Head, "head"),
                (Cell::Tail, "tail"),
                (Cell::Empty, "erase"),
            ] {
                let text = RichText::new(name).color(if cell == Cell::Empty {
                    Color32::GRAY
                } else {
                    colour(cell)
                });
                ui.selectable_value(&mut self.brush, cell, text);
            }
        });
        ui.small("Drag on the board to draw; shift-drag erases. An electron is a head with a tail just behind it: it moves away from its tail.");
        ui.separator();

        ui.label(RichText::new("Circuits").strong());
        egui::ComboBox::from_id_salt("pattern")
            .selected_text(PATTERNS[self.pattern].name)
            .show_ui(ui, |ui| {
                for (i, p) in PATTERNS.iter().enumerate() {
                    ui.selectable_value(&mut self.pattern, i, p.name);
                }
            });
        ui.small(PATTERNS[self.pattern].note);
        if ui.button("Place on a clear board").clicked() {
            self.world = Wireworld::new(self.size.0, self.size.1);
            self.place_pattern();
        }
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Board:");
            for &size in SIZES {
                let label = format!("{} x {}", size.0, size.1);
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.world = Wireworld::new(size.0, size.1);
                    self.place_pattern();
                }
            }
        });
        ui.separator();

        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            ui.label("generation");
            ui.label(self.world.generation.to_string());
            ui.end_row();
            ui.label("electrons");
            ui.label(self.world.count(Cell::Head).to_string());
            ui.end_row();
        });
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.world = Wireworld::new(self.size.0, self.size.1);
            self.dirty = true;
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => self.pace.steps(per_second as f64, dt, 50),
        };
        for _ in 0..steps {
            self.world.step();
        }
        self.dirty |= steps > 0;
    }

    /// Put the chosen circuit in the middle of the board.
    fn place_pattern(&mut self) {
        let p = &PATTERNS[self.pattern];
        let (pw, ph) = p.size();
        let x0 = self.size.0.saturating_sub(pw) / 2;
        let y0 = self.size.1.saturating_sub(ph) / 2;
        self.world.place(p, x0, y0);
        self.dirty = true;
    }
}

fn colour(cell: Cell) -> Color32 {
    match cell {
        Cell::Empty => EMPTY,
        Cell::Conductor => WIRE,
        Cell::Head => HEAD,
        Cell::Tail => TAIL,
    }
}
