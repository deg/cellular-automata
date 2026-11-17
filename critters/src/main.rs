//! Critters demo: block cellular automata on the Margolus neighbourhood, run forwards and
//! backwards.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, Grid, GridView, NAVY, Pace, PaintStroke, Playback, Rng, eframe,
    generations_per_frame, grouped, readout, speed_cap_note,
};
use sim::{Blocks, Fill, RULES, Rule};

fn main() {
    ca_core::run("Critters", CrittersApp::new);
}

const PARTICLE: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a); // gold

const SIZES: &[(usize, usize)] = &[(160, 100), (320, 200), (640, 400)];

/// The world size to use for a rule needing at least `min`: the current one if it is big
/// enough, otherwise the smallest offered that is.
fn size_for(current: (usize, usize), min: (usize, usize)) -> (usize, usize) {
    let fits = |s: &(usize, usize)| s.0 >= min.0 && s.1 >= min.1;
    if fits(&current) {
        current
    } else {
        *SIZES
            .iter()
            .find(|s| fits(s))
            .unwrap_or(&SIZES[SIZES.len() - 1])
    }
}

/// Which way Run and Step go.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Forwards,
    Backwards,
}

struct CrittersApp {
    world: Blocks,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    direction: Direction,
    size: (usize, usize),
    /// The seed of the current opening; Restart moves on to the next.
    seed: u64,
    /// The picture at generation 0, to count how far the world is from it.
    start: Grid<u8>,
    /// Cells unlike `start`, counted when the world last changed.
    unlike: usize,
    brush: usize,
    random_brush: bool,
    rng: Rng,
    stroke: PaintStroke,
    hovered: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for CrittersApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            self.view.set_image(ui.ctx(), &self.world.particles(), |c| {
                if c == 1 { PARTICLE } else { NAVY }
            });
            self.unlike = self.world.differences(&self.start);
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
                let fill = if ui.input(|i| i.modifiers.shift) {
                    Fill::Empty
                } else if self.random_brush {
                    Fill::Random
                } else {
                    Fill::Particles
                };
                for (x, y) in cells {
                    self.world.paint(x, y, self.brush, fill, &mut self.rng);
                }
                if self.world.generation == 0 {
                    // Drawing at the start changes the start.
                    self.start = self.world.particles();
                }
                self.dirty = true;
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl CrittersApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let world = Blocks::new(size.0, size.1, RULES[0]);
        let start = world.particles();
        let mut app = Self {
            world,
            view: GridView::new("critters"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(30, 1_000, "generations / second"),
            pace: Pace::default(),
            direction: Direction::Forwards,
            size,
            seed: 1,
            start,
            unlike: 0,
            brush: 0,
            random_brush: false,
            rng: Rng::new(0),
            stroke: PaintStroke::default(),
            hovered: None,
            dirty: true,
        };
        app.restart();
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Critters");
        ui.label("The world is cut into blocks of 2 x 2 cells, and a table of 16 entries says what each block becomes; on the next step the blocks shift by one cell. Every table here can be undone, so the world can run backwards to exactly where it started.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.horizontal_wrapped(|ui| {
            ui.label("Direction:");
            ui.selectable_value(&mut self.direction, Direction::Forwards, "forwards");
            ui.selectable_value(&mut self.direction, Direction::Backwards, "backwards");
        });
        if ui.button("Clear").clicked() {
            self.world.clear();
            self.start = self.world.particles();
            self.dirty = true;
        }
        ui.small("Backwards undoes the steps one at a time, exactly, and stops at generation 0, the start; Run again to go on into its past. Restart draws a new opening; Clear empties the world.");
        speed_cap_note(ui, "world");
        ui.separator();

        ui.label(RichText::new("Rule").strong());
        let mut chosen: Option<&'static Rule> = None;
        ui.horizontal_wrapped(|ui| {
            for rule in RULES {
                if ui
                    .selectable_label(self.world.rule == rule, rule.name)
                    .clicked()
                {
                    chosen = Some(rule);
                }
            }
        });
        ui.small(self.world.rule.note);
        if let Some(rule) = chosen {
            self.size = size_for(self.size, rule.min_size);
            self.world = Blocks::new(self.size.0, self.size.1, rule);
            self.restart();
        }
        ui.separator();

        ui.label(RichText::new("Draw").strong());
        ui.horizontal_wrapped(|ui| {
            ui.label("Brush:");
            ui.selectable_value(&mut self.random_brush, false, "particles");
            ui.selectable_value(&mut self.random_brush, true, "random");
        });
        ui.add(egui::Slider::new(&mut self.brush, 0..=10).text("brush radius"));
        ui.small("Drag on the grid to paint; shift-drag to erase. Drawing after generation 0 changes the history: run back, and the start no longer returns.");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{} x {}", size.0, size.1);
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.world = Blocks::new(size.0, size.1, self.world.rule);
                    self.restart();
                }
            }
        });
        ui.small("The edges wrap round: what leaves one side comes back on the other.");
        ui.separator();

        readout(ui, |r| {
            r.row("generation", signed_whole(self.world.generation));
            r.row("particles", grouped(self.world.particle_count() as u64));
            r.row("cells unlike the start", grouped(self.unlike as u64));
            r.pointer(self.hovered.map(|(x, y)| {
                let state = if self.world.particle(x, y) {
                    "particle"
                } else {
                    "empty"
                };
                format!("({x}, {y}): {state}")
            }));
        });
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.seed += 1;
            self.restart();
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => {
                // A block step costs about 0.6 ns a cell natively (200 steps of 640 x 400 in
                // 30 ms), against the 11 ns `generations_per_frame` budgets for: count a
                // quarter of the cells, leaving room for the slower browser build.
                let cap = generations_per_frame(self.size.0 * self.size.1 / 4, 50);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            match self.direction {
                Direction::Forwards => self.world.step(),
                Direction::Backwards => {
                    self.world.step_back();
                    if self.world.generation == 0 && self.playback.running {
                        // Back at the start: stop to show it.
                        self.playback.running = false;
                        break;
                    }
                }
            }
        }
        self.dirty |= steps > 0;
    }

    /// The rule's opening, drawn afresh from the current seed.
    fn restart(&mut self) {
        self.world.restart(&mut Rng::new(self.seed));
        self.start = self.world.particles();
        self.dirty = true;
    }
}

/// A whole number with a true minus sign and commas: generations before the start are
/// negative.
fn signed_whole(n: i64) -> String {
    let digits = grouped(n.unsigned_abs());
    if n < 0 {
        format!("\u{2212}{digits}")
    } else {
        digits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A rule that needs a bigger world gets the smallest one that fits, and every rule's
    /// minimum is one of the sizes offered, so none is quietly run too small.
    #[test]
    fn rules_get_a_big_enough_world() {
        assert_eq!(size_for((160, 100), (640, 400)), (640, 400));
        assert_eq!(size_for((320, 200), (0, 0)), (320, 200));
        for rule in RULES {
            let s = size_for(SIZES[0], rule.min_size);
            assert!(
                s.0 >= rule.min_size.0 && s.1 >= rule.min_size.1,
                "{}",
                rule.name
            );
        }
    }

    #[test]
    fn signed_whole_uses_a_minus_sign() {
        assert_eq!(signed_whole(0), "0");
        assert_eq!(signed_whole(1_234), "1,234");
        assert_eq!(signed_whole(-1_234), "\u{2212}1,234");
    }
}
