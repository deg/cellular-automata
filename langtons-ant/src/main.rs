//! Langton's ant demo: one ant, two rules, and a highway out of chaos.

mod sim;

use ca_core::egui::{self, Color32, RichText, Stroke};
use ca_core::{About, Advance, GridResponse, GridView, Pace, Playback, eframe};
use sim::{Ant, Rule, World};

fn main() {
    ca_core::run("Langton's ant", AntApp::new);
}

/// Colour 0 (the blank grid) and the colours cells cycle through after it.
const BLANK: Color32 = ca_core::NAVY;
const RAMP: [Color32; 5] = [
    Color32::from_rgb(0x2a, 0x9d, 0x8f),
    Color32::from_rgb(0xe9, 0xc4, 0x6a),
    Color32::from_rgb(0xe7, 0x6f, 0x51),
    Color32::from_rgb(0xd6, 0x4f, 0xa0),
    Color32::from_rgb(0x9b, 0x8c, 0xf0),
];
const ANT: Color32 = Color32::from_rgb(0xff, 0x30, 0x30);

/// Rules to try. Only Langton's own and the two whose behaviour sources describe carry a
/// note; the others are left for the viewer to discover.
const PRESETS: &[(&str, &str)] = &[
    (
        "RL",
        "Langton's ant: chaos, then a highway after about 10,000 steps.",
    ),
    ("RLR", "Grows chaotically."),
    ("LLRR", "Four colours. Watch its shape."),
    ("RLLR", "Four colours. Compare it with LLRR."),
    ("LLRRRLRLRLLR", "Builds a convoluted highway."),
    ("LRRRRRLLR", "Nine colours. See what it makes."),
];

const SIDES: &[usize] = &[201, 401, 801];

struct AntApp {
    world: World,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    side: usize,
    rule_text: String,
    rule_error: Option<String>,
    palette: Vec<Color32>,
    /// The cell under the pointer, for the readout.
    hovered: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for AntApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            let palette = &self.palette;
            self.view
                .set_image(ui.ctx(), self.world.grid(), |c| palette[c as usize]);
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            if let Some((x, y)) = shown.clicked {
                self.world.ants.push(Ant::new(x, y));
            }
            self.mark_ants(ui, &shown);
            // The readout was drawn before the grid this frame: draw again, or the pointer
            // row lags a cell behind while paused.
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

impl AntApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let side = SIDES[1];
        let rule = Rule::parse("RL").expect("valid rule");
        Self {
            palette: palette(rule.colours()),
            rule_text: rule.to_string(),
            world: World::new(side, rule),
            view: GridView::new("ant"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(300, 1_000_000, "steps / second"),
            pace: Pace::default(),
            side,
            rule_error: None,
            hovered: None,
            dirty: true,
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Langton's ant");
        ui.label("Each step, the ant turns (right on a blank cell, left on a coloured one), changes the cell's colour, and moves forward one cell.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        if ui.button("Jump ahead 10,000 steps").clicked() {
            for _ in 0..10_000 {
                self.world.step();
            }
            self.dirty = true;
        }
        ui.small("Restart clears the grid and leaves one ant at the centre. Click the grid to add another ant there.");
        ui.separator();

        ui.label(RichText::new("Rule").strong());
        ui.small("One letter per colour: on that colour, turn Left or Right. Cells cycle through the colours in order.");
        let current = self.world.rule().to_string();
        ui.horizontal_wrapped(|ui| {
            for &(rule, _) in PRESETS {
                if ui.selectable_label(current == rule, rule).clicked() {
                    self.rule_text = rule.to_string();
                    self.restart();
                }
            }
        });
        ca_core::preset_note(
            ui,
            PRESETS
                .iter()
                .find(|(rule, _)| current == *rule)
                .map(|(_, note)| *note),
            PRESETS.iter().map(|(_, note)| *note),
            ca_core::OWN_SETTINGS,
        );
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.rule_text).desired_width(140.0));
            if ui.button("Use").clicked() {
                self.restart();
            }
        });
        if let Some(err) = &self.rule_error {
            ui.colored_label(Color32::YELLOW, err);
        }
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("World:");
            for &side in SIDES {
                if ui
                    .selectable_label(self.side == side, format!("{side} x {side}"))
                    .clicked()
                    && self.side != side
                {
                    self.side = side;
                    self.restart();
                }
            }
        });
        ui.small("The edges wrap around: an ant walking off one side comes back on the other.");
        ui.separator();

        let grid = self.world.grid();
        let under = (self.hovered)
            .filter(|&(x, y)| x < grid.width() && y < grid.height())
            .map(|(x, y)| match grid.get(x, y) {
                0 => format!("({x}, {y}): blank"),
                colour => format!("({x}, {y}): colour {colour}"),
            });
        ca_core::readout(ui, |r| {
            r.row("rule", current);
            r.row("step", ca_core::grouped(self.world.steps));
            r.row("ants", ca_core::grouped(self.world.ants.len() as u64));
            r.pointer(under);
        });
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.restart();
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => self.pace.steps(per_second as f64, dt, 200_000),
        };
        for _ in 0..steps {
            self.world.step();
        }
        self.dirty |= steps > 0;
    }

    /// A fresh grid with one ant at the centre, under the typed rule if it parses.
    fn restart(&mut self) {
        // The pointer's cell may not exist in the restarted world (it can be smaller).
        self.hovered = None;
        let rule = match Rule::parse(&self.rule_text) {
            Ok(rule) => {
                self.rule_error = None;
                rule
            }
            Err(e) => {
                self.rule_error = Some(e);
                self.world.rule().clone()
            }
        };
        self.palette = palette(rule.colours());
        self.world = World::new(self.side, rule);
        self.dirty = true;
    }

    /// Draw each ant as a red dot, big enough to find on a large grid.
    fn mark_ants(&self, ui: &egui::Ui, shown: &GridResponse) {
        let radius = (shown.cell_size() * 0.6).max(3.0);
        for ant in &self.world.ants {
            let centre = shown.cell_rect(ant.x, ant.y).center();
            ui.painter()
                .circle(centre, radius, ANT, Stroke::new(1.0, Color32::BLACK));
        }
    }
}

/// `n` colours: the blank first, then evenly spread along the ramp.
fn palette(n: usize) -> Vec<Color32> {
    let mut colours = vec![BLANK];
    for i in 1..n {
        let t = if n == 2 {
            0.0
        } else {
            (i - 1) as f32 / (n - 2) as f32
        };
        let pos = t * (RAMP.len() - 1) as f32;
        let (lo, frac) = (pos.floor() as usize, pos.fract());
        let hi = (lo + 1).min(RAMP.len() - 1);
        colours.push(RAMP[lo].lerp_to_gamma(RAMP[hi], frac));
    }
    colours
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every rule length the parser accepts (2 to 12 colours) needs exactly one colour per
    /// cell colour, starting with the blank, or drawing indexes past the end. The ramp
    /// interpolation's index arithmetic is where that would go wrong.
    #[test]
    fn palette_has_one_colour_per_rule_colour() {
        for n in 2..=12 {
            let p = palette(n);
            assert_eq!(p.len(), n);
            assert_eq!(p[0], BLANK);
        }
        assert_eq!(
            palette(12)[11],
            RAMP[RAMP.len() - 1],
            "the last colour ends the ramp"
        );
    }
}
