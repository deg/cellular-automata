//! Conway's Game of Life demo: paint cells or place famous patterns, then run.

mod rle;
mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, GridView, NAVY, Pace, PaintStroke, Playback, Rng, eframe,
    generations_per_frame, grouped,
};
use rle::Shape;
use sim::{Life, Rule};

fn main() {
    ca_core::run("Conway's Game of Life", LifeApp::new);
}

const DEAD: Color32 = NAVY;
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

/// What Restart puts back: the last pattern placed or random fill.
enum Opening {
    Pattern(usize),
    Pasted(Shape),
    Random,
}

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
    opening: Opening,
    pasted: String,
    paste_error: Option<String>,
    density: f64,
    stroke: PaintStroke,
    /// The cell under the pointer, for the readout.
    pointer: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for LifeApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
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
            let cells = self.stroke.cells(shown.pressed);
            if !cells.is_empty() {
                let alive = !ui.input(|i| i.modifiers.shift);
                for (x, y) in cells {
                    self.life.set(x, y, alive);
                }
                self.dirty = true;
            }
            if shown.hovered != self.pointer {
                self.pointer = shown.hovered;
                ui.ctx().request_repaint(); // the controls were drawn before this
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
        let pattern = 2; // the glider gun: something happens as soon as you press Run
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
            pattern,
            opening: Opening::Pattern(pattern),
            pasted: String::new(),
            paste_error: None,
            density: 0.3,
            stroke: PaintStroke::default(),
            pointer: None,
            dirty: true,
        };
        app.restart();
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Conway's Game of Life");
        ui.label("Each generation, a dead cell with 3 live neighbours is born; a live cell with 2 or 3 survives; every other cell dies or stays dead.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ca_core::speed_cap_note(ui, "world");
        if ui.button("Clear").clicked() {
            self.life.clear();
            self.dirty = true;
        }
        ui.small("Restart puts back the last pattern placed, or a fresh random fill; Clear empties the world. Drag on the grid to draw live cells; shift-drag to erase.");
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
        ui.horizontal_wrapped(|ui| {
            if ui.button("Place at centre").clicked() {
                self.place(PatternSource::BuiltIn);
            }
            if ui.button("Random fill").clicked() {
                self.opening = Opening::Random;
                self.random_fill();
            }
        });
        // On its own line: a slider doesn't wrap like a button, so beside the buttons it ran
        // past the fixed-width panel's edge.
        ui.add(egui::Slider::new(&mut self.density, 0.05..=0.8).text("fill density"));
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
        let typed = Rule::parse(&self.rule_text).ok();
        let chosen = RULES
            .iter()
            .find(|&&(_, rule, _)| typed.is_some() && Rule::parse(rule).ok() == typed);
        ui.horizontal_wrapped(|ui| {
            for &(name, rule, _) in RULES {
                let selected = chosen.is_some_and(|&(n, _, _)| n == name);
                if ui.selectable_label(selected, name).clicked() {
                    self.rule_text = rule.to_string();
                    self.apply_rule();
                }
            }
        });
        ca_core::preset_note(
            ui,
            chosen.map(|&(_, _, note)| note),
            RULES.iter().map(|&(_, _, note)| note),
            ca_core::OWN_SETTINGS,
        );
        ui.horizontal_wrapped(|ui| {
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
        ui.horizontal_wrapped(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{} x {}", size.0, size.1);
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.pointer = None;
                    self.restart();
                }
            }
        });
        ui.separator();

        let (w, h) = self.size;
        let pointer = self.pointer.filter(|&(x, y)| x < w && y < h).map(|(x, y)| {
            let state = if self.life.grid().get(x, y) {
                "alive"
            } else {
                "dead"
            };
            format!("({x}, {y}): {state}")
        });
        ca_core::readout(ui, |r| {
            r.row("generation", grouped(self.life.generation));
            r.row("population", grouped(self.life.population() as u64));
            r.pointer(pointer);
        });
    }

    /// Apply this frame's Restart, Step or Run.
    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.restart();
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

    /// Back to the opening, keeping the rule and wrap: a new world of the current size with
    /// the last pattern placed, or a fresh random fill at the current density.
    fn restart(&mut self) {
        // The pointer's cell may not exist in the restarted world (it can be smaller).
        self.pointer = None;
        self.life = Life::new(self.size.0, self.size.1, self.life.rule, self.life.wrap);
        match &self.opening {
            Opening::Pattern(i) => {
                let shape = rle::parse(rle::PATTERNS[*i].rle).expect("built-in patterns parse");
                self.life.place_centred(&shape);
            }
            Opening::Pasted(shape) => self.life.place_centred(shape),
            Opening::Random => self.random_fill(),
        }
        self.dirty = true;
    }

    /// Place a pattern at the centre, over whatever is there, and make it the opening.
    fn place(&mut self, source: PatternSource) {
        let text = match source {
            PatternSource::BuiltIn => rle::PATTERNS[self.pattern].rle,
            PatternSource::Pasted => self.pasted.as_str(),
        };
        match rle::parse(text) {
            Ok(shape) => {
                self.life.place_centred(&shape);
                self.opening = match source {
                    PatternSource::BuiltIn => Opening::Pattern(self.pattern),
                    PatternSource::Pasted => Opening::Pasted(shape),
                };
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
