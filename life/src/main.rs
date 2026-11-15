//! Conway's Game of Life demo: paint cells or place famous patterns, then run.

mod rle;
mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{About, Advance, GridView, Pace, Playback, Rng, eframe, generations_per_frame, line};
use sim::{Life, Rule};

fn main() {
    ca_core::run("Conway's Game of Life", LifeApp::new);
}

const DEAD: Color32 = Color32::from_rgb(0x14, 0x18, 0x30);
const ALIVE: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

/// Rules worth trying, with what to look for. Each is a "Life-like" B/S rule.
const RULES: &[(&str, &str, &str)] = &[
    ("Life", "B3/S23", "Conway's original."),
    (
        "HighLife",
        "B36/S23",
        "Like Life, plus a pattern that copies itself.",
    ),
    (
        "Seeds",
        "B2/S",
        "Every live cell dies at once; most patterns explode.",
    ),
    (
        "Day & Night",
        "B3678/S34678",
        "Live and dead behave alike: inverted patterns evolve the same.",
    ),
];

/// World sizes offered, in cells.
const SIZES: &[(usize, usize)] = &[(160, 100), (320, 200), (640, 400)];

struct LifeApp {
    life: Life,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    rng: Rng,
    size: (usize, usize),
    /// The rule as typed, which may not parse; the world keeps its last good rule meanwhile.
    rule_text: String,
    rule_error: Option<String>,
    pattern: usize,
    pasted: String,
    paste_error: Option<String>,
    density: f64,
    /// The last cell painted in the current stroke, so a fast drag paints a solid line.
    last_paint: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for LifeApp {
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
            self.view.set_image(
                ui.ctx(),
                self.life.grid(),
                |alive| if alive { ALIVE } else { DEAD },
            );
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            match shown.pressed {
                Some(cell) => {
                    let alive = !ui.input(|i| i.modifiers.shift);
                    for (x, y) in line(self.last_paint.unwrap_or(cell), cell) {
                        self.life.set(x, y, alive);
                    }
                    self.last_paint = Some(cell);
                    self.dirty = true;
                }
                None => self.last_paint = None,
            }
            if let Some((x, y)) = shown.hovered {
                shown.response.on_hover_text(format!("({x}, {y})"));
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl LifeApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let mut app = Self {
            life: Life::new(size.0, size.1, Rule::life(), true),
            view: GridView::new("life"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(15, 1_000, "generations / second"),
            pace: Pace::default(),
            rng: Rng::new(1970),
            size,
            rule_text: Rule::life().to_string(),
            rule_error: None,
            pattern: 2, // the glider gun: something happens as soon as you press Run
            pasted: String::new(),
            paste_error: None,
            density: 0.3,
            last_paint: None,
            dirty: true,
        };
        app.place(PatternSource::BuiltIn);
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Conway's Game of Life");
        ui.label("Each generation, a dead cell with 3 live neighbours is born; a live cell with 2 or 3 survives; every other cell dies or stays dead.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small(
            "Restart clears the world. Drag on the grid to draw live cells; shift-drag to erase.",
        );
        ui.separator();

        ui.label(RichText::new("Patterns").strong());
        let current = &rle::PATTERNS[self.pattern];
        egui::ComboBox::from_id_salt("pattern")
            .selected_text(current.name)
            .show_ui(ui, |ui| {
                for (i, p) in rle::PATTERNS.iter().enumerate() {
                    ui.selectable_value(&mut self.pattern, i, p.name);
                }
            });
        ui.small(rle::PATTERNS[self.pattern].note);
        ui.horizontal(|ui| {
            if ui.button("Place at centre").clicked() {
                self.place(PatternSource::BuiltIn);
            }
            if ui.button("Random fill").clicked() {
                self.random_fill();
            }
            ui.add(egui::Slider::new(&mut self.density, 0.05..=0.8).text("density"));
        });
        ui.collapsing("Paste an RLE pattern", |ui| {
            ui.small("From LifeWiki or Golly, e.g. bob$2bo$3o! for a glider.");
            ui.add(
                egui::TextEdit::multiline(&mut self.pasted)
                    .desired_rows(3)
                    .code_editor(),
            );
            if ui.button("Place pasted pattern").clicked() {
                self.place(PatternSource::Pasted);
            }
            if let Some(err) = &self.paste_error {
                ui.colored_label(Color32::YELLOW, err);
            }
        });
        ui.separator();

        ui.label(RichText::new("Rule").strong());
        ui.horizontal_wrapped(|ui| {
            for &(name, rule, note) in RULES {
                if ui.button(name).on_hover_text(note).clicked() {
                    self.rule_text = rule.to_string();
                    self.apply_rule();
                }
            }
        });
        ui.horizontal(|ui| {
            let edit = ui.add(egui::TextEdit::singleline(&mut self.rule_text).desired_width(120.0));
            if edit.changed() {
                self.apply_rule();
            }
            ui.label(format!("now {}", self.life.rule));
        });
        if let Some(err) = &self.rule_error {
            ui.colored_label(Color32::YELLOW, err);
        }
        ui.small("B lists the neighbour counts that bring a dead cell to life, S those that keep a live one alive.");
        ui.separator();

        ui.checkbox(&mut self.life.wrap, "Wrap around the edges")
            .on_hover_text("On: the world is a torus, and a glider leaving one side returns on the other. Off: everything past the edge is dead.");
        ui.horizontal(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{} x {}", size.0, size.1);
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.life = Life::new(size.0, size.1, self.life.rule, self.life.wrap);
                    self.dirty = true;
                }
            }
        });
        ui.separator();

        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            ui.label("generation");
            ui.label(self.life.generation.to_string());
            ui.end_row();
            ui.label("population");
            ui.label(self.life.population().to_string());
            ui.end_row();
        });
    }

    /// Apply this frame's Restart, Step or Run.
    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.life.clear();
            self.dirty = true;
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            // Cap the work per frame so a big world at top speed slows down, not freezes.
            Advance::Run(per_second) => {
                let cap = generations_per_frame(self.size.0 * self.size.1, 50);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            self.life.step();
        }
        self.dirty |= steps > 0;
    }

    fn place(&mut self, source: PatternSource) {
        let text = match source {
            PatternSource::BuiltIn => rle::PATTERNS[self.pattern].rle,
            PatternSource::Pasted => self.pasted.as_str(),
        };
        match rle::parse(text) {
            Ok(shape) => {
                self.life.place_centred(&shape);
                self.paste_error = None;
                self.dirty = true;
            }
            Err(e) => self.paste_error = Some(e),
        }
    }

    fn random_fill(&mut self) {
        let (w, h) = self.size;
        for y in 0..h {
            for x in 0..w {
                let alive = self.rng.chance(self.density);
                self.life.set(x, y, alive);
            }
        }
        self.life.generation = 0;
        self.dirty = true;
    }

    /// Use the typed rule if it parses; otherwise keep the old one and show why.
    fn apply_rule(&mut self) {
        match Rule::parse(&self.rule_text) {
            Ok(rule) => {
                self.life.rule = rule;
                self.rule_error = None;
            }
            Err(e) => self.rule_error = Some(e),
        }
    }
}

enum PatternSource {
    BuiltIn,
    Pasted,
}
