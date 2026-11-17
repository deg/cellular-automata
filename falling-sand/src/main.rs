//! Falling-sand demo: paint sand, water, walls, plants and fire, and watch them fall, flow,
//! grow and burn.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, Grid, GridView, NAVY, Pace, PaintStroke, Playback, eframe,
    generations_per_frame, grouped, mix, readout, speed_cap_note,
};
use sim::{Material, OPENINGS, World};

fn main() {
    ca_core::run("Falling sand", SandApp::new);
}

const GOLD: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);
const BLUE: Color32 = Color32::from_rgb(0x3a, 0x6e, 0xa5);
const TEAL: Color32 = Color32::from_rgb(0x2a, 0x9d, 0x8f);
const CORAL: Color32 = Color32::from_rgb(0xe7, 0x6f, 0x51);
const SPARK: Color32 = Color32::from_rgb(0xf4, 0xf1, 0xde);

const SIZES: &[(usize, usize)] = &[(160, 100), (320, 200), (640, 400)];

/// The colour of every possible cell byte: each material in its palette colour, grains a
/// little darker or lighter by their shade (so moving sand can be seen moving), and flames
/// from gold when new to coral as they burn down.
fn palette() -> [Color32; 256] {
    let mut colours = [NAVY; 256];
    for (c, colour) in colours.iter_mut().enumerate() {
        let c = c as u8;
        let shade = f32::from(sim::extra(c)) / 15.0;
        *colour = match sim::material(c) {
            Material::Empty => NAVY,
            Material::Wall => mix(NAVY, SPARK, 0.42),
            Material::Sand => mix(GOLD, NAVY, 0.22 * shade),
            Material::Water => mix(BLUE, NAVY, 0.12 * shade),
            Material::Plant => mix(TEAL, NAVY, 0.25 * shade),
            Material::Fire => mix(CORAL, GOLD, shade),
        };
    }
    colours
}

struct SandApp {
    world: World,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    size: (usize, usize),
    /// Which of `OPENINGS` Restart returns to.
    opening: usize,
    /// The seed for the next world; Restart moves it on, so grains get new shades.
    seed: u64,
    brush: Material,
    radius: usize,
    stroke: PaintStroke,
    hovered: Option<(usize, usize)>,
    colours: [Color32; 256],
    /// The cells' bytes, copied for drawing.
    picture: Grid<u8>,
    counts: [usize; 6],
    dirty: bool,
}

impl eframe::App for SandApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            self.picture.cells_mut().copy_from_slice(self.world.cells());
            let colours = &self.colours;
            self.view
                .set_image(ui.ctx(), &self.picture, |c| colours[usize::from(c)]);
            self.counts = self.world.counts();
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            if shown.hovered != self.hovered {
                // Redraw the "under the pointer" row now, not at the next event.
                self.hovered = shown.hovered;
                ui.ctx().request_repaint();
            }
            let cells = self.stroke.cells(shown.pressed);
            if !cells.is_empty() {
                let paint = if ui.input(|i| i.modifiers.shift) {
                    Material::Empty
                } else {
                    self.brush
                };
                for (x, y) in cells {
                    self.world.paint(x, y, self.radius, paint);
                }
                self.dirty = true;
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl SandApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let mut app = Self {
            world: World::new(size.0, size.1, 1),
            view: GridView::new("falling-sand"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(120, 1_000, "steps / second"),
            pace: Pace::default(),
            size,
            opening: 0,
            seed: 1,
            brush: Material::Sand,
            radius: 3,
            stroke: PaintStroke::default(),
            hovered: None,
            colours: palette(),
            picture: Grid::new(size.0, size.1, 0),
            counts: [0; 6],
            dirty: true,
        };
        app.restart();
        // The hourglass running out is the opening worth seeing.
        app.playback.running = true;
        app
    }

    /// The current opening again, at the current size, with new shades for its grains.
    fn restart(&mut self) {
        self.seed += 1;
        self.world = World::new(self.size.0, self.size.1, self.seed);
        (OPENINGS[self.opening].build)(&mut self.world);
        self.picture = Grid::new(self.size.0, self.size.1, 0);
        self.dirty = true;
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Falling sand");
        ui.label("Each step, every grain of sand falls or slides down a slope, water falls and spreads to find its level, plants grow into water, and fire rises and burns plants.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        speed_cap_note(ui, "world");
        if ui.button("Clear").clicked() {
            self.world.clear();
            self.dirty = true;
        }
        ui.small("Restart goes back to the opening; Clear empties the world.");
        ui.separator();

        ui.label(RichText::new("Opening").strong());
        ui.horizontal_wrapped(|ui| {
            for (k, opening) in OPENINGS.iter().enumerate() {
                if ui
                    .selectable_label(self.opening == k, opening.name)
                    .clicked()
                {
                    self.opening = k;
                    self.restart();
                }
            }
        });
        ui.small(OPENINGS[self.opening].note);
        ui.separator();

        ui.label(RichText::new("Draw").strong());
        ui.horizontal_wrapped(|ui| {
            for m in Material::PAINTS {
                ui.selectable_value(&mut self.brush, m, m.name());
            }
        });
        ui.add(egui::Slider::new(&mut self.radius, 0..=12).text("brush radius"));
        ui.small("Drag on the grid to paint; shift-drag to erase. Paint fills only empty cells, but fire also lights plants.");
        ui.separator();

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
        ui.small("The edges are walls.");
        ui.separator();

        let n = |m: Material| grouped(self.counts[m as usize] as u64);
        readout(ui, |r| {
            r.row("step", grouped(self.world.step));
            r.row("grains of sand", n(Material::Sand));
            r.row("water cells", n(Material::Water));
            r.row("plant cells", n(Material::Plant));
            r.row("flames", n(Material::Fire));
            r.pointer(
                self.hovered
                    .map(|(x, y)| format!("({x}, {y}): {}", self.world.get(x, y).name())),
            );
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
                // A step costs 1 to 3.5 ns a cell natively on the openings at 640 x 400
                // (the pressure rule is up to half of it), and 11 ns with half the world in
                // motion, against the 11 ns `generations_per_frame` budgets for: count half
                // the cells, leaving room for the slower browser build.
                let cap = generations_per_frame(self.size.0 * self.size.1 / 2, 50);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            self.world.step();
        }
        self.dirty |= steps > 0;
    }
}
