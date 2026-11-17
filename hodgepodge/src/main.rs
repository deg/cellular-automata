//! Hodgepodge machine demo: cells fall sick, infect their neighbours and recover, in waves
//! and spirals like those of an oscillating chemical reaction.

mod sim;

use std::collections::VecDeque;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, GridView, NAVY, Pace, PaintStroke, Playback, Plot, Rng, Series, Style, eframe,
    generations_per_frame, grouped, mix, readout, speed_cap_note,
};
use sim::{Health, Hodgepodge, MAX_N, Neighbourhood, PRESETS, Rule};

fn main() {
    ca_core::run("Hodgepodge machine", HodgepodgeApp::new);
}

const TEAL: Color32 = Color32::from_rgb(0x2a, 0x9d, 0x8f);
const GOLD: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);
const CORAL: Color32 = Color32::from_rgb(0xe7, 0x6f, 0x51);
const SPARK: Color32 = Color32::from_rgb(0xf4, 0xf1, 0xde);

const SIZES: &[(usize, usize)] = &[(160, 100), (320, 200), (640, 400)];

/// Generations the chart shows.
const HISTORY: usize = 500;

/// The colour of every state of `rule`: healthy the background, infection from teal through
/// gold to coral as it rises, ill the bright cream.
fn palette(rule: Rule) -> Vec<Color32> {
    (0..=MAX_N)
        .map(|s| match rule.health(s) {
            Health::Healthy => NAVY,
            Health::Ill => SPARK,
            Health::Infected(level) => {
                let t = f32::from(level) / f32::from(rule.n);
                if t < 0.5 {
                    mix(TEAL, GOLD, 2.0 * t)
                } else {
                    mix(GOLD, CORAL, 2.0 * t - 1.0)
                }
            }
        })
        .collect()
}

struct HodgepodgeApp {
    hp: Hodgepodge,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    colours: Vec<Color32>,
    /// The seed of the current random start; Restart moves on to the next.
    seed: u64,
    size: (usize, usize),
    brush: usize,
    rng: Rng,
    stroke: PaintStroke,
    /// (generation, share of cells infected), the last [`HISTORY`] generations.
    history: VecDeque<(f64, f64)>,
    hovered: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for HodgepodgeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            let colours = &self.colours;
            self.view
                .set_image(ui.ctx(), self.hp.grid(), |s| colours[usize::from(s)]);
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            if shown.hovered != self.hovered {
                // The panel was drawn before the grid, so redraw it with the new cell now
                // rather than on the next input event.
                self.hovered = shown.hovered;
                ui.ctx().request_repaint();
            }
            let cells = self.stroke.cells(shown.pressed);
            if !cells.is_empty() {
                let erase = ui.input(|i| i.modifiers.shift);
                for (x, y) in cells {
                    if erase {
                        self.hp.paint_healthy(x, y, self.brush);
                    } else {
                        self.hp.paint_infection(x, y, self.brush, &mut self.rng);
                    }
                }
                self.dirty = true;
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl HodgepodgeApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let rule = PRESETS[PRESETS.len() - 1].rule;
        let mut app = Self {
            hp: Hodgepodge::new(size.0, size.1, rule, true),
            view: GridView::new("hodgepodge"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(30, 1_000, "generations / second"),
            pace: Pace::default(),
            colours: palette(rule),
            seed: 1,
            size,
            brush: 4,
            rng: Rng::new(0),
            stroke: PaintStroke::default(),
            history: VecDeque::with_capacity(HISTORY + 1),
            hovered: None,
            dirty: true,
        };
        app.restart();
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hodgepodge machine");
        ui.label("Each cell is healthy, infected at some level, or ill. Infection spreads to healthy neighbours and worsens until the cell falls ill; then it recovers.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        speed_cap_note(ui, "world");
        if ui.button("Clear").clicked() {
            self.hp.clear();
            self.history.clear();
            self.dirty = true;
        }
        ui.small("Restart starts again from fresh random states; Clear makes every cell healthy.");
        ui.separator();

        ui.label(RichText::new("Presets").strong());
        ui.horizontal_wrapped(|ui| {
            for p in PRESETS {
                if ui
                    .selectable_label(self.hp.rule == p.rule, p.name)
                    .clicked()
                {
                    self.hp.rule = p.rule;
                    self.restart();
                }
            }
        });
        if let Some(p) = PRESETS.iter().find(|p| p.rule == self.hp.rule) {
            ui.small(p.note);
        }
        ui.separator();

        self.rule_controls(ui);
        ui.separator();

        self.chart(ui);
        ui.separator();

        ui.label(RichText::new("Draw").strong());
        ui.small("Drag on the grid to infect cells; shift-drag to erase.");
        ui.add(egui::Slider::new(&mut self.brush, 0..=20).text("brush radius"));
        ui.separator();

        ui.checkbox(&mut self.hp.wrap, "Wrap around the edges");
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
        ui.separator();

        let cells = self.size.0 * self.size.1;
        let rule = self.hp.rule;
        let grid = self.hp.grid();
        readout(ui, |r| {
            r.row("generation", grouped(self.hp.generation));
            r.row(
                "infected",
                format!(
                    "{} ({:.1}%)",
                    grouped(self.hp.infected() as u64),
                    100.0 * self.hp.infected() as f64 / cells as f64
                ),
            );
            r.pointer(self.hovered.map(|(x, y)| {
                let what = match rule.health(grid.get(x, y)) {
                    Health::Healthy => "healthy".to_string(),
                    Health::Infected(level) => format!("infected {level}"),
                    Health::Ill => "ill".to_string(),
                };
                format!("({x}, {y}): {what}")
            }));
        });
    }

    /// The four numbers and the neighbourhood, by hand. Changing n starts again (cells may
    /// be in states that no longer exist); the rest carry on from here.
    fn rule_controls(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Rule").strong());
        let mut rule = self.hp.rule;
        ui.add(egui::Slider::new(&mut rule.n, 2..=MAX_N).text("n, the ill state"));
        rule.g = rule.g.min(rule.n);
        ui.add(egui::Slider::new(&mut rule.g, 1..=rule.n).text("g, infection's rise"));
        ui.add(egui::Slider::new(&mut rule.k1, 1..=8).text("k1, per infected"));
        ui.add(egui::Slider::new(&mut rule.k2, 1..=8).text("k2, per ill"));
        ui.horizontal_wrapped(|ui| {
            ui.label("Neighbours:");
            ui.radio_value(&mut rule.neighbourhood, Neighbourhood::VonNeumann, "4")
                .on_hover_text("Von Neumann: the 4 cells sharing an edge.");
            ui.radio_value(&mut rule.neighbourhood, Neighbourhood::Moore, "8")
                .on_hover_text("Moore: the 8 cells sharing an edge or a corner.");
        });
        ui.small(
            "A healthy cell catches a level for every k1 infected and every k2 ill neighbours.",
        );

        if rule != self.hp.rule {
            let restart = rule.n != self.hp.rule.n;
            self.hp.rule = rule;
            self.colours = palette(rule);
            if restart {
                self.restart();
            }
            self.dirty = true;
        }
    }

    /// The share of cells infected, generation by generation, as in the graphs Gerhardt and
    /// Schuster told their four types of behaviour apart by.
    fn chart(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Infected cells").strong());
        let points = self.history.make_contiguous();
        ca_core::plot(
            ui,
            &Plot {
                x_label: "generation",
                y_label: "share infected",
                log_x: false,
                log_y: false,
                x_range: None,
                y_range: Some((0.0, 1.0)),
                series: vec![Series {
                    points,
                    colour: ca_core::CHART_MEASURED,
                    style: Style::Line,
                }],
            },
            100.0,
        );
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
                let cap = generations_per_frame(self.size.0 * self.size.1, 50);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        let cells = (self.size.0 * self.size.1) as f64;
        for _ in 0..steps {
            self.hp.step();
            self.history
                .push_back((self.hp.generation as f64, self.hp.infected() as f64 / cells));
            if self.history.len() > HISTORY {
                self.history.pop_front();
            }
        }
        self.dirty |= steps > 0;
    }

    /// A fresh world of random states from the current seed, rule, edges and size.
    fn restart(&mut self) {
        let (rule, wrap) = (self.hp.rule, self.hp.wrap);
        self.hp = Hodgepodge::new(self.size.0, self.size.1, rule, wrap);
        self.hp.fill_random(&mut Rng::new(self.seed));
        self.colours = palette(rule);
        self.history.clear();
        // The cell under the pointer belonged to the old world, which may have been bigger.
        self.hovered = None;
        self.dirty = true;
    }
}
