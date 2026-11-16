//! Lattice gas demo: particles colliding on a hexagonal lattice, which flow like a fluid.

mod sim;
mod view;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{About, Advance, Pace, Playback, eframe, generations_per_frame};
use sim::{Inflow, Lattice, PARTICLES, ROW_HEIGHT, WALL};
use view::LatticeView;

fn main() {
    ca_core::run("Lattice gas", GasApp::new);
}

const NAVY: Color32 = Color32::from_rgb(0x14, 0x18, 0x30);
const DUSK: Color32 = Color32::from_rgb(0x1f, 0x2a, 0x55);
const BLUE: Color32 = Color32::from_rgb(0x3a, 0x6e, 0xa5);
const TEAL: Color32 = Color32::from_rgb(0x2a, 0x9d, 0x8f);
const GOLD: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);
const CORAL: Color32 = Color32::from_rgb(0xe7, 0x6f, 0x51);
const SPARK: Color32 = Color32::from_rgb(0xf4, 0xf1, 0xde);
const STONE: Color32 = Color32::from_rgb(0xb8, 0xbc, 0xc8);

/// A node's colour by its particle count, 0 to 6.
const BY_COUNT: [Color32; 7] = [NAVY, DUSK, BLUE, TEAL, GOLD, CORAL, SPARK];

/// World sizes, in nodes. Heights are even, as the wrapping lattice needs.
const SIZES: &[(usize, usize)] = &[(400, 200), (800, 400), (1200, 600)];

/// Columns refilled with fresh gas each step, at the left edge.
const INFLOW_WIDTH: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Preset {
    Plate,
    Cylinder,
    Channel,
    Pulse,
}

impl Preset {
    const ALL: [Preset; 4] = [Self::Plate, Self::Cylinder, Self::Channel, Self::Pulse];

    fn label(self) -> &'static str {
        match self {
            Self::Plate => "Flat plate in a wind",
            Self::Cylinder => "Cylinder in a wind",
            Self::Channel => "Flow down a channel",
            Self::Pulse => "A pulse of gas at rest",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::Plate => "A wall across the flow. Eddies grow behind it.",
            Self::Cylinder => "A round obstacle. Its wake is gentler than the plate's.",
            Self::Channel => {
                "Walls along the top and bottom. The gas sticks to them and the flow slows near them."
            }
            Self::Pulse => {
                "A dense disc of gas in still gas spreads as a ring: a sound wave, and a round one."
            }
        }
    }
}

struct GasApp {
    lattice: Lattice,
    view: LatticeView,
    about: About,
    playback: Playback,
    pace: Pace,
    size: (usize, usize),
    preset: Preset,
    /// Particles per direction per node, 0 to 1: the gas's density.
    density: f64,
    /// The wind's speed, in node spacings per step.
    speed: f64,
    brush: f64,
    hovered: Option<(usize, usize)>,
    seed: u64,
    dirty: bool,
}

impl eframe::App for GasApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            let l = &self.lattice;
            let (w, cells) = (l.width(), l.cells());
            let image = view::image(w, l.height(), |x, y| {
                let c = cells[y * w + x];
                if c & WALL != 0 {
                    STONE
                } else {
                    BY_COUNT[(c & PARTICLES).count_ones() as usize]
                }
            });
            self.view.set_image(ui.ctx(), l.width(), l.height(), image);
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            self.hovered = shown.hovered;
            if let Some((x, y)) = shown.pressed {
                let erase = ui.input(|i| i.modifiers.shift);
                self.paint(x, y, !erase);
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl GasApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let mut app = Self {
            lattice: Lattice::new(size.0, size.1, 1986),
            view: LatticeView::new(),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(200, 5_000, "steps / second"),
            pace: Pace::default(),
            size,
            preset: Preset::Plate,
            density: 0.2,
            speed: 0.3,
            brush: 3.0,
            hovered: None,
            seed: 1986,
            dirty: true,
        };
        app.restart();
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Lattice gas");
        ui.label("Particles move along the lines of a hexagonal lattice, at most one per direction at each node. When they meet at a node, they scatter, keeping their number and momentum. Averaged over many nodes, they flow like a fluid.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small("Restart sets up the start again. Drag on the lattice to draw walls; shift-drag to erase them.");
        ui.separator();

        ui.label(RichText::new("Start").strong());
        for preset in Preset::ALL {
            if ui
                .selectable_label(self.preset == preset, preset.label())
                .on_hover_text(preset.hint())
                .clicked()
            {
                self.preset = preset;
                self.restart();
            }
        }
        ui.small(self.preset.hint());
        ui.add(egui::Slider::new(&mut self.density, 0.05..=0.5).text("density"));
        ui.add(egui::Slider::new(&mut self.speed, 0.0..=0.4).text("wind speed"));
        ui.small("Density is particles per direction per node; speed, nodes per step. They take effect on Restart (the wind at once).");
        if let Some(inflow) = &mut self.lattice.inflow {
            inflow.speed = self.speed;
            inflow.density = self.density;
        }
        ui.horizontal_wrapped(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{} x {}", size.0, size.1);
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.restart();
                }
            }
        });
        ui.add(egui::Slider::new(&mut self.brush, 1.0..=20.0).text("brush radius"));
        ui.separator();

        let l = &self.lattice;
        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            ui.label("step");
            ui.label(l.steps.to_string());
            ui.end_row();
            let n = l.particles();
            ui.label("particles");
            ui.label(n.to_string());
            ui.end_row();
            // Momentum per particle: the gas's mean velocity, in node spacings per step.
            let (px, py) = l.momentum();
            let n = n.max(1) as f64;
            ui.label("mean velocity");
            ui.label(format!(
                "({:.3}, {:.3})",
                px as f64 / 2.0 / n,
                py as f64 * 3f64.sqrt() / 4.0 / n
            ));
            ui.end_row();
            if let Some((x, y)) = self.hovered {
                let c = l.get(x, y);
                ui.label(format!("node ({x}, {y})"));
                ui.label(if c & WALL != 0 {
                    "wall".to_string()
                } else {
                    format!("{} particles", (c & PARTICLES).count_ones())
                });
                ui.end_row();
            }
        });
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.restart();
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => {
                let cap = generations_per_frame(self.size.0 * self.size.1, 200);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            self.lattice.step();
        }
        self.dirty |= steps > 0;
    }

    /// Set the current preset up afresh.
    fn restart(&mut self) {
        let (w, h) = self.size;
        self.seed += 1;
        let mut l = Lattice::new(w, h, self.seed);
        let mid_y = h as f64 * ROW_HEIGHT / 2.0;
        let wind = Some(Inflow {
            width: INFLOW_WIDTH,
            density: self.density,
            speed: self.speed,
        });
        match self.preset {
            Preset::Plate => {
                let half = h as f64 * ROW_HEIGHT / 6.0;
                let x0 = w as f64 / 4.0;
                l.wall_where(|px, py| (px - x0).abs() <= 1.0 && (py - mid_y).abs() <= half);
                l.fill(self.density, self.speed);
                l.inflow = wind;
            }
            Preset::Cylinder => {
                let r = h as f64 * ROW_HEIGHT / 8.0;
                let x0 = w as f64 / 4.0;
                l.wall_where(|px, py| (px - x0).hypot(py - mid_y) <= r);
                l.fill(self.density, self.speed);
                l.inflow = wind;
            }
            Preset::Channel => {
                for x in 0..w {
                    l.set_wall(x, 0, true);
                    l.set_wall(x, h - 1, true);
                }
                l.fill(self.density, self.speed);
                l.inflow = wind;
            }
            Preset::Pulse => {
                l.fill(self.density, 0.0);
                let r = h as f64 * ROW_HEIGHT / 20.0;
                l.fill_disc(w as f64 / 2.0, mid_y, r, (self.density * 2.5).min(1.0));
            }
        }
        self.lattice = l;
        self.dirty = true;
    }

    /// Draw (or erase) walls in a disc around node `(x, y)`.
    fn paint(&mut self, x: usize, y: usize, wall: bool) {
        let (cx, cy) = Lattice::position(x, y);
        let r = self.brush;
        let l = &mut self.lattice;
        let rows = (r / ROW_HEIGHT).ceil() as isize + 1;
        for dy in -rows..=rows {
            let ny = y as isize + dy;
            if ny < 0 || ny >= l.height() as isize {
                continue;
            }
            for dx in -(r as isize + 1)..=(r as isize + 1) {
                let nx = x as isize + dx;
                if nx < 0 || nx >= l.width() as isize {
                    continue;
                }
                let (nx, ny) = (nx as usize, ny as usize);
                let (px, py) = Lattice::position(nx, ny);
                if (px - cx).hypot(py - cy) <= r && l.is_wall(nx, ny) != wall {
                    l.set_wall(nx, ny, wall);
                }
            }
        }
        self.dirty = true;
    }
}
