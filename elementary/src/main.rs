//! Elementary cellular automata demo: all 256 one-dimensional rules, drawn as a
//! space-time diagram in which each new generation is a new row.

mod sim;

use ca_core::egui::{self, Color32, CornerRadius, Rect, RichText, Sense, Stroke, pos2, vec2};
use ca_core::{About, Advance, GridView, Pace, Playback, eframe};
use sim::{Elementary, Start};

fn main() {
    ca_core::run("Elementary cellular automata", ElementaryApp::new);
}

const OFF: Color32 = ca_core::NAVY;
const ON: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

/// Rules worth a look, with what to notice.
const PRESETS: &[(u8, &str)] = &[
    (
        30,
        "Chaos from a single cell; its centre column looks random.",
    ),
    (
        90,
        "Each cell is the XOR of its neighbours: the Sierpinski triangle.",
    ),
    (
        110,
        "Can compute anything, as Matthew Cook proved in 2004. Try a random start.",
    ),
    (184, "Traffic: each 1 is a car that moves right into a gap."),
    (
        150,
        "XOR of all three cells: another nested, fractal pattern.",
    ),
];

/// Row widths offered; the diagram keeps three rows for every four cells of width.
const WIDTHS: &[usize] = &[201, 401, 801];

struct ElementaryApp {
    automaton: Elementary,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    width: usize,
    random_start: bool,
    seed: u64,
    /// The cell under the pointer, for the readout.
    hovered: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for ElementaryApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            self.view
                .set_image(ui.ctx(), self.automaton.history(), |on| {
                    if on { ON } else { OFF }
                });
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            // The readout was drawn before the picture this frame: draw again, or the
            // pointer row lags a cell behind while paused.
            if shown.hovered != self.hovered {
                self.hovered = shown.hovered;
                ui.ctx().request_repaint();
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl ElementaryApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let width = WIDTHS[1];
        let mut playback = Playback::new(60, 2_000, "generations / second");
        playback.running = true; // a 1D rule is best seen growing
        Self {
            automaton: Elementary::new(width, rows_for(width), 30, true, Start::SingleCell),
            view: GridView::new("elementary"),
            about: About::new(include_str!("../README.md")),
            playback,
            pace: Pace::default(),
            width,
            random_start: false,
            seed: 1,
            hovered: None,
            dirty: true,
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Elementary cellular automata");
        ui.label("One row of cells, each 0 or 1. Each generation, every cell looks at itself and its two neighbours and follows the rule's table below. Each new generation is drawn as the next row down.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small("Restart begins again from the start chosen below.");
        ui.separator();

        ui.label(RichText::new("Rule").strong());
        ui.horizontal_wrapped(|ui| {
            ui.label("Rule number");
            ui.add(egui::DragValue::new(&mut self.automaton.rule).range(0..=255));
        });
        rule_table(ui, &mut self.automaton.rule);
        ui.small("Click an output square to flip it. The outputs, read as binary from left to right, are the rule number.");
        ui.horizontal_wrapped(|ui| {
            for &(rule, _) in PRESETS {
                let chosen = self.automaton.rule == rule;
                if ui.selectable_label(chosen, rule.to_string()).clicked() {
                    self.automaton.rule = rule;
                    self.restart();
                }
            }
        });
        ca_core::preset_note(
            ui,
            PRESETS
                .iter()
                .find(|(r, _)| *r == self.automaton.rule)
                .map(|(_, note)| *note),
            PRESETS.iter().map(|(_, note)| *note),
            ca_core::OWN_SETTINGS,
        );
        ui.small("A preset restarts; changing the rule by hand carries on from the current row.");
        ui.separator();

        ui.label(RichText::new("Start").strong());
        ui.horizontal_wrapped(|ui| {
            if ui.radio(!self.random_start, "single cell").clicked() {
                self.random_start = false;
                self.restart();
            }
            if ui.radio(self.random_start, "random row").clicked() {
                self.random_start = true;
                self.seed += 1;
                self.restart();
            }
        });
        ui.checkbox(&mut self.automaton.wrap, "Wrap the row's ends together")
            .on_hover_text("On: the row is a ring. Off: beyond each end is always 0.");
        ui.horizontal_wrapped(|ui| {
            ui.label("Width:");
            for &w in WIDTHS {
                if ui
                    .selectable_label(self.width == w, w.to_string())
                    .clicked()
                    && self.width != w
                {
                    self.width = w;
                    self.restart();
                }
            }
        });
        if ui.button("Fill the picture now").clicked() {
            for _ in 0..rows_for(self.width) {
                self.automaton.step();
            }
            self.dirty = true;
        }
        ui.separator();

        let a = &self.automaton;
        let on = a.row().iter().filter(|&&b| b).count();
        let under = (self.hovered).map(|(x, y)| {
            let history = a.history();
            match a.generation_of_row(y) {
                Some(g) if x < history.width() => {
                    let state = if history.get(x, y) { "on" } else { "off" };
                    // (cell, generation), as the 2D demos write (x, y).
                    format!("({x}, {g}): {state}")
                }
                // Only while the diagram is still filling, when row y will hold generation y.
                _ => format!("({x}, {y}): not reached yet"),
            }
        });
        ca_core::readout(ui, |r| {
            r.row("generation", ca_core::grouped(a.generation));
            r.row("cells on", ca_core::grouped(on as u64));
            r.pointer(under);
        });
        ui.small("Under Rule 184 on a ring, the count of cells on never changes: cars are neither made nor destroyed.");
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.seed += 1;
            self.restart();
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => self.pace.steps(per_second as f64, dt, 200),
        };
        for _ in 0..steps {
            self.automaton.step();
        }
        self.dirty |= steps > 0;
    }

    /// Begin again with the current rule, start, edge and width.
    fn restart(&mut self) {
        // The pointer's cell may not exist in the restarted world (it can be smaller).
        self.hovered = None;
        let start = if self.random_start {
            Start::Random(self.seed)
        } else {
            Start::SingleCell
        };
        let (rule, wrap) = (self.automaton.rule, self.automaton.wrap);
        self.automaton = Elementary::new(self.width, rows_for(self.width), rule, wrap, start);
        self.dirty = true;
    }
}

fn rows_for(width: usize) -> usize {
    width * 3 / 4
}

/// Draw the rule as its table: each of the 8 three-cell patterns, from 111 to 000, with the
/// cell it produces underneath. Clicking an output flips that bit of the rule.
fn rule_table(ui: &mut egui::Ui, rule: &mut u8) {
    let cell = 9.0;
    let gap = 5.0;
    let group = 3.0 * cell + gap;
    let (rect, _) = ui.allocate_exact_size(vec2(8.0 * group, 2.0 * cell + 4.0), Sense::hover());
    let painter = ui.painter_at(rect);
    for (slot, pattern) in (0..8u8).rev().enumerate() {
        let left = rect.left() + slot as f32 * group;
        let square = |x: f32, y: f32| Rect::from_min_size(pos2(x, y), vec2(cell, cell));
        for bit in 0..3 {
            let on = pattern >> (2 - bit) & 1 == 1;
            let r = square(left + bit as f32 * cell, rect.top());
            painter.rect_filled(r, CornerRadius::ZERO, if on { ON } else { OFF });
            painter.rect_stroke(
                r,
                CornerRadius::ZERO,
                Stroke::new(1.0, Color32::GRAY),
                egui::StrokeKind::Inside,
            );
        }
        let out = square(left + cell, rect.top() + cell + 4.0);
        let on = *rule >> pattern & 1 == 1;
        painter.rect_filled(out, CornerRadius::ZERO, if on { ON } else { OFF });
        painter.rect_stroke(
            out,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::WHITE),
            egui::StrokeKind::Inside,
        );
        let response = ui.interact(
            out,
            ui.make_persistent_id(("rule-bit", pattern)),
            Sense::click(),
        );
        if response
            .on_hover_text(format!("pattern {:03b}", pattern))
            .clicked()
        {
            *rule ^= 1 << pattern;
        }
    }
}
