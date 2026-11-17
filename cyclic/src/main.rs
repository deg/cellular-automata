//! Cyclic cellular automaton demo: random colours organise themselves into spiral waves.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, GridView, Pace, PaintStroke, Playback, Rng, eframe, generations_per_frame,
    grouped,
};
use sim::{Cyclic, MAX_COLOURS, MAX_RANGE, Neighbourhood, PRESETS};

fn main() {
    ca_core::run("Cyclic cellular automaton", CyclicApp::new);
}

/// The project's palette, in the order the colour wheel runs through it.
const WHEEL: [[u8; 3]; 4] = [
    [0x3a, 0x6e, 0xa5], // blue
    [0x2a, 0x9d, 0x8f], // teal
    [0xe9, 0xc4, 0x6a], // gold
    [0xe7, 0x6f, 0x51], // coral
];

const SIZES: &[(usize, usize)] = &[(160, 100), (320, 200), (640, 400)];

/// The world size to use for a preset needing at least `min`: the current one if it is big
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

/// `n` colours evenly spaced along a closed loop through the palette, so colour k + 1 is
/// always a similar small step on from colour k, and the last colour leads back to the
/// first. Spaced by distance along the loop, not by keyframe: blue to teal is a much
/// shorter step than the others, and equal shares would bunch colours there.
fn palette(n: u8) -> Vec<Color32> {
    let n = usize::from(n.max(1));
    let point = |i: usize| WHEEL[i % WHEEL.len()].map(f32::from);
    let lengths: Vec<f32> = (0..WHEEL.len())
        .map(|i| {
            let (a, b) = (point(i), point(i + 1));
            (0..3).map(|j| (a[j] - b[j]).powi(2)).sum::<f32>().sqrt()
        })
        .collect();
    let total: f32 = lengths.iter().sum();
    (0..n)
        .map(|k| {
            let mut along = k as f32 * total / n as f32;
            let mut i = 0;
            while i + 1 < lengths.len() && along > lengths[i] {
                along -= lengths[i];
                i += 1;
            }
            let f = along / lengths[i];
            let (a, b) = (point(i), point(i + 1));
            let mix = |j: usize| (a[j] * (1.0 - f) + b[j] * f).round() as u8;
            Color32::from_rgb(mix(0), mix(1), mix(2))
        })
        .collect()
}

struct CyclicApp {
    ca: Cyclic,
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
    /// The cell under the pointer last frame, for the readout.
    hovered: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for CyclicApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            let colours = &self.colours;
            self.view.set_image(ui.ctx(), self.ca.grid(), |c| {
                colours[usize::from(c) % colours.len()]
            });
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
                let solid = ui.input(|i| i.modifiers.shift);
                for (x, y) in cells {
                    if solid {
                        self.ca.paint_solid(x, y, self.brush, 0);
                    } else {
                        self.ca.paint_noise(x, y, self.brush, &mut self.rng);
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

impl CyclicApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let rule = PRESETS[0].rule;
        let mut app = Self {
            ca: Cyclic::new(size.0, size.1, rule, true),
            view: GridView::new("cyclic"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(30, 1_000, "generations / second"),
            pace: Pace::default(),
            colours: palette(rule.colours),
            seed: 1,
            size,
            brush: 6,
            rng: Rng::new(0),
            stroke: PaintStroke::default(),
            hovered: None,
            dirty: true,
        };
        app.restart();
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Cyclic cellular automaton");
        ui.label("Each cell has a colour on a wheel. A cell moves on to the next colour when enough cells near it already have that colour; the last colour leads back to the first.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ca_core::speed_cap_note(ui, "world");
        ui.small("Restart starts again from fresh random colours.");
        ui.separator();

        ui.label(RichText::new("Presets").strong());
        ui.horizontal_wrapped(|ui| {
            for p in PRESETS {
                if ui
                    .selectable_label(self.ca.rule == p.rule, p.name)
                    .clicked()
                {
                    self.ca.rule = p.rule;
                    self.size = size_for(self.size, p.min_size);
                    self.restart();
                }
            }
        });
        if let Some(p) = PRESETS.iter().find(|p| p.rule == self.ca.rule) {
            ui.small(format!("{} {}", p.note, p.credit));
        }
        ui.separator();

        self.rule_controls(ui);
        ui.separator();

        ui.label(RichText::new("Draw").strong());
        ui.small(
            "Drag on the grid to paint random colours; shift-drag paints a patch of one colour.",
        );
        ui.add(egui::Slider::new(&mut self.brush, 1..=30).text("brush radius"));
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("Random seed:");
            if ui
                .add(egui::DragValue::new(&mut self.seed).range(0..=1_000_000_000))
                .changed()
            {
                self.restart();
            }
        });
        ui.checkbox(&mut self.ca.wrap, "Wrap around the edges")
            .on_hover_text("On: the world is a torus, and a wave leaving one side returns on the other. Off: nothing lies past the edge, so no cell there counts.");
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
        let changed = self.ca.changed();
        let grid = self.ca.grid();
        ca_core::readout(ui, |r| {
            r.row("generation", grouped(self.ca.generation));
            r.row(
                "changed last generation",
                format!(
                    "{} ({:.1}%)",
                    grouped(changed as u64),
                    100.0 * changed as f64 / cells as f64
                ),
            );
            r.pointer(
                self.hovered
                    .map(|(x, y)| format!("({x}, {y}): colour {}", grid.get(x, y))),
            );
        });
        if self.ca.generation > 0 && changed == 0 {
            ui.small("Nothing changed in the last generation: the world has fixated, and stays as it is unless you paint on it.");
        } else if changed == cells {
            ui.small("Every cell changed: the whole world is cycling through the colours, one step each generation.");
        }
    }

    /// Range, threshold, colours and neighbourhood, by hand. Changing the number of colours
    /// starts again (old colours may not exist any more); the rest carry on from here.
    fn rule_controls(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Rule").strong());
        let mut rule = self.ca.rule;
        ui.add(egui::Slider::new(&mut rule.range, 1..=MAX_RANGE).text("range"));
        ui.horizontal_wrapped(|ui| {
            ui.label("Neighbourhood:");
            ui.radio_value(&mut rule.neighbourhood, Neighbourhood::Moore, "box")
                .on_hover_text(
                    "Moore: every cell within range across, down or diagonally, a square.",
                );
            ui.radio_value(
                &mut rule.neighbourhood,
                Neighbourhood::VonNeumann,
                "diamond",
            )
            .on_hover_text(
                "Von Neumann: every cell within range counting steps north, south, east and west.",
            );
        });
        let most = rule.neighbours();
        rule.threshold = rule.threshold.min(most);
        ui.label(format!("Threshold: next-colour cells needed, of {most}"));
        ui.add(egui::Slider::new(&mut rule.threshold, 1..=most).logarithmic(most > 40));
        ui.add(egui::Slider::new(&mut rule.colours, 2..=MAX_COLOURS).text("colours"));
        ui.small(format!("In MCell's notation: {}", rule.notation()));

        if rule != self.ca.rule {
            let recolour = rule.colours != self.ca.rule.colours;
            self.ca.rule = rule;
            if recolour {
                self.restart();
            }
        }
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
                let work = self.size.0 * self.size.1 * self.ca.work_per_cell();
                let cap = generations_per_frame(work, 50);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            self.ca.step();
        }
        self.dirty |= steps > 0;
    }

    /// A fresh world of random colours from the current seed, rule, edges and size.
    fn restart(&mut self) {
        // The pointer's cell may not exist in the restarted world (it can be smaller).
        self.hovered = None;
        let (rule, wrap) = (self.ca.rule, self.ca.wrap);
        self.ca = Cyclic::new(self.size.0, self.size.1, rule, wrap);
        self.ca.fill_random(&mut Rng::new(self.seed));
        self.colours = palette(rule.colours);
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A preset that needs a bigger world gets the smallest one that fits, and every
    /// preset's minimum is one of the sizes offered, so none is quietly run too small.
    #[test]
    fn presets_get_a_big_enough_world() {
        assert_eq!(size_for((160, 100), (320, 200)), (320, 200));
        assert_eq!(size_for((640, 400), (320, 200)), (640, 400));
        assert_eq!(size_for((160, 100), (0, 0)), (160, 100));
        for p in PRESETS {
            let s = size_for(SIZES[0], p.min_size);
            assert!(s.0 >= p.min_size.0 && s.1 >= p.min_size.1, "{}", p.name);
        }
    }

    /// The colours must read as a cycle: all different, starting on the palette's blue, and
    /// evenly spaced all the way round, the step from the last colour back to the first
    /// included, so the wheel has no visible seam and no near-duplicates.
    #[test]
    fn the_palette_is_an_even_closed_loop() {
        let dist = |a: Color32, b: Color32| {
            let d = |x: u8, y: u8| (f32::from(x) - f32::from(y)).powi(2);
            (d(a.r(), b.r()) + d(a.g(), b.g()) + d(a.b(), b.b())).sqrt()
        };
        for n in 2..=MAX_COLOURS {
            let p = palette(n);
            assert_eq!(p.len(), usize::from(n));
            assert_eq!(p[0], Color32::from_rgb(0x3a, 0x6e, 0xa5));
            let steps: Vec<f32> = (0..p.len())
                .map(|i| dist(p[i], p[(i + 1) % p.len()]))
                .collect();
            let longest = steps.iter().copied().fold(0.0, f32::max);
            let shortest = steps.iter().copied().fold(f32::INFINITY, f32::min);
            assert!(
                longest < 2.0 * shortest,
                "{n} colours: steps from {shortest} to {longest}"
            );
        }
    }
}
