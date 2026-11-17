//! Lenia demo: Bert Chan's continuous cellular automaton and its creatures.

mod fft;
mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, Grid, GridView, NAVY, Pace, PaintStroke, Playback, Rng, eframe,
    generations_per_frame, grouped,
};
use sim::{CREATURES, GLIDER_GUN, LIFE, Rule, World, decode};

fn main() {
    ca_core::run("Lenia", LeniaApp::new);
}

const BLUE: Color32 = Color32::from_rgb(0x3a, 0x6e, 0xa5);
const TEAL: Color32 = Color32::from_rgb(0x2a, 0x9d, 0x8f);
const GOLD: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

/// A state's colour, from 0 (navy, empty) up to 1 (gold), as in reaction-diffusion.
const RAMP: [Color32; 4] = [NAVY, BLUE, TEAL, GOLD];
/// Shades in the colour table.
const SHADES: usize = 256;

/// World sizes: powers of two, for the Fourier transform.
const SIZES: &[usize] = &[128, 256, 512];

/// A soup has one patch for this many cells: 32 on the 256 x 256 world. Chan's
/// `random_world` scatters 15 to 40 patches whatever the world's size. Tried on the 256
/// world with Orbium's rule, 24 soups each, 1,500 steps (outside this repository): at one
/// patch for 1,024 cells all 24 filled the world; at 2,048, 4 left creatures, 14 filled it
/// and 6 died out; at 4,096, 3, 1 and 20. Every density left creatures about one time in
/// five; this one shows all three endings.
const CELLS_PER_PATCH: usize = 2_048;

/// The growth centre μ's slider range, and the growth width σ's (logarithmic). They hold
/// every creature's values and Life's (μ = 0.35, σ = 0.07).
const MU_RANGE: (f32, f32) = (0.05, 0.5);
const SIGMA_RANGE: (f32, f32) = (0.002, 0.1);

/// What Restart goes back to.
#[derive(Clone, Copy, PartialEq)]
enum Opening {
    /// One of [`CREATURES`], in the middle of the world.
    Creature(usize),
    /// Random patches, with Orbium's rule.
    Soup,
    /// Gosper's glider gun, with Life's rule.
    Life,
}

impl Opening {
    const ALL: [Opening; CREATURES.len() + 2] = {
        let mut all = [Opening::Soup; CREATURES.len() + 2];
        let mut i = 0;
        while i < CREATURES.len() {
            all[i] = Opening::Creature(i);
            i += 1;
        }
        all[i + 1] = Opening::Life;
        all
    };

    fn name(self) -> &'static str {
        match self {
            Opening::Creature(i) => CREATURES[i].name,
            Opening::Soup => "Soup",
            Opening::Life => "Life",
        }
    }

    fn note(self) -> &'static str {
        match self {
            Opening::Creature(i) => CREATURES[i].note,
            Opening::Soup => {
                "Random patches, with Orbium's rule. Most soups die out or fill the world; now and then an Orbium crawls out. Restart for another."
            }
            Opening::Life => {
                "Conway's Game of Life, as Lenia: radius 2, whole steps, a staircase ring and an on-off growth. Gosper's glider gun fires gliders round the world, until they wrap round and hit it."
            }
        }
    }

    /// The rule as Chan gives it.
    fn rule(self) -> Rule {
        match self {
            Opening::Creature(i) => CREATURES[i].rule,
            Opening::Soup => CREATURES[0].rule,
            Opening::Life => LIFE,
        }
    }
}

/// How much a step costs against a cell-generation of Life, which `generations_per_frame`'s
/// budget is sized for (about 11 ns). Measured natively in a release build (`timing_steps`):
/// 7.6 ns a cell at 256 x 256, 8.8 at 512 x 512, whatever the radius. That is close
/// enough to Life's, and the browser is slower, so the budget is used as it stands.
const LIFE_CELLS_PER_CELL: usize = 1;

struct LeniaApp {
    world: World,
    opening: Opening,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    colours: Vec<Color32>,
    size: usize,
    rng: Rng,
    brush: f32,
    stroke: PaintStroke,
    hovered: Option<(usize, usize)>,
    dirty: bool,
}

impl eframe::App for LeniaApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            let image = shades(&self.world);
            let colours = &self.colours;
            self.view
                .set_image(ui.ctx(), &image, |s| colours[usize::from(s)]);
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
                let erase = ui.input(|i| i.modifiers.shift);
                let rng = &mut self.rng;
                for (x, y) in cells {
                    self.world.paint_disc(x, y, self.brush, || {
                        if erase {
                            0.0
                        } else {
                            (rng.next_f64() * 0.9) as f32
                        }
                    });
                }
                self.dirty = true;
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl LeniaApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let opening = Opening::Creature(0);
        let mut app = Self {
            world: World::new(size, opening.rule()),
            opening,
            view: GridView::new("lenia"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(30, 600, "steps / second"),
            pace: Pace::default(),
            colours: palette(),
            size,
            rng: Rng::new(2019),
            brush: 6.0,
            stroke: PaintStroke::default(),
            hovered: None,
            dirty: true,
        };
        app.restart();
        app.playback.running = true; // Orbium gliding across the world is the show
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Lenia");
        ui.label("Each cell holds a number from 0 to 1. Every step it takes a weighted average of the cells in a ring around it, and grows if that average is near a target value, shrinks if not.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ca_core::speed_cap_note(ui, "world");
        if ui.button("Clear").clicked() {
            self.world.clear();
            self.dirty = true;
        }
        ui.small("Restart puts the creature back (or draws a fresh soup), with the growth settings below; Clear empties the world, ready to draw on.");
        ui.separator();

        ui.label(RichText::new("Creatures").strong());
        ui.horizontal_wrapped(|ui| {
            for opening in Opening::ALL {
                if ui
                    .selectable_label(self.opening == opening, opening.name())
                    .clicked()
                {
                    self.opening = opening;
                    self.world.set_rule(opening.rule());
                    self.restart();
                }
            }
        });
        ui.small(self.opening.note());
        ui.separator();

        self.growth_controls(ui);
        ui.separator();

        ui.label(RichText::new("Draw").strong());
        ui.small("Drag on the world to scatter random values; shift-drag to erase.");
        ui.add(egui::Slider::new(&mut self.brush, 1.0..=20.0).text("brush radius"));
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{size} x {size}");
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.restart();
                }
            }
        });
        ui.separator();

        let w = &self.world;
        ca_core::readout(ui, |r| {
            r.row("step", grouped(w.steps));
            r.row("mass", grouped(w.mass().round() as u64))
                .on_hover_text("The sum of all the cells' values: a creature's stays nearly constant as it moves.");
            r.pointer(
                self.hovered
                    .map(|(x, y)| format!("({x}, {y}): {:.2}", w.get(x, y))),
            );
        });
    }

    /// The growth centre μ and width σ, which carry on from the current world.
    fn growth_controls(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Growth").strong());
        let mut rule = self.world.rule();
        ui.add(
            egui::Slider::new(&mut rule.mu, MU_RANGE.0..=MU_RANGE.1)
                .step_by(0.001)
                .fixed_decimals(3)
                .text("centre μ"),
        );
        ui.add(
            egui::Slider::new(&mut rule.sigma, SIGMA_RANGE.0..=SIGMA_RANGE.1)
                .logarithmic(true)
                .fixed_decimals(4)
                .text("width σ"),
        );
        let chans = self.opening.rule();
        ui.horizontal_wrapped(|ui| {
            ui.small(format!(
                "Chan's values: μ = {}, σ = {}.",
                chans.mu, chans.sigma
            ));
            let changed = rule.mu != chans.mu || rule.sigma != chans.sigma;
            if ui
                .add_enabled(changed, egui::Button::new("Put back"))
                .clicked()
            {
                (rule.mu, rule.sigma) = (chans.mu, chans.sigma);
            }
        });
        ui.small("A creature lives only in a narrow band of these: nudge one and watch it explode or evaporate.");
        if rule != self.world.rule() {
            self.world.set_rule(rule);
        }
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.restart();
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => {
                let cap = generations_per_frame(self.size * self.size * LIFE_CELLS_PER_CELL, 1_000);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            self.world.step();
        }
        self.dirty |= steps > 0;
    }

    /// The opening, at the current size and growth settings.
    fn restart(&mut self) {
        let rule = self.world.rule();
        if self.world.side() != self.size {
            self.world = World::new(self.size, rule);
        }
        match self.opening {
            Opening::Creature(i) => self.world.place(&decode(CREATURES[i].cells)),
            Opening::Soup => {
                let patches = self.size * self.size / CELLS_PER_PATCH;
                self.world.soup(patches, &mut self.rng);
            }
            Opening::Life => self.world.place(&decode(GLIDER_GUN)),
        }
        self.dirty = true;
    }
}

/// The world as shades, indices into [`palette`].
fn shades(w: &World) -> Grid<u8> {
    let n = w.side();
    let mut g = Grid::new(n, n, 0u8);
    let top = (SHADES - 1) as f32;
    for (s, &a) in g.cells_mut().iter_mut().zip(w.cells()) {
        *s = (a * top).round() as u8;
    }
    g
}

/// [`SHADES`] colours along [`RAMP`].
fn palette() -> Vec<Color32> {
    (0..SHADES)
        .map(|i| {
            let t = i as f32 / (SHADES - 1) as f32 * (RAMP.len() - 1) as f32;
            let j = (t as usize).min(RAMP.len() - 2);
            ca_core::mix(RAMP[j], RAMP[j + 1], t - j as f32)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An empty cell is navy like every demo's background; a full one is the ramp's top.
    #[test]
    fn empty_is_navy() {
        let p = palette();
        let mut w = World::new(8, LIFE);
        w.set(1, 1, 1.0);
        let s = shades(&w);
        assert_eq!(p[usize::from(s.get(0, 0))], NAVY);
        assert_eq!(p[usize::from(s.get(1, 1))], GOLD);
    }

    /// Every chip's rule fits the sliders, so choosing one doesn't get clamped.
    #[test]
    fn chans_values_fit_the_sliders() {
        for o in Opening::ALL {
            let r = o.rule();
            assert!((MU_RANGE.0..=MU_RANGE.1).contains(&r.mu), "{}", o.name());
            assert!(
                (SIGMA_RANGE.0..=SIGMA_RANGE.1).contains(&r.sigma),
                "{}",
                o.name()
            );
        }
    }

    /// Every opening fits the smallest world.
    #[test]
    fn openings_fit_the_smallest_world() {
        for o in Opening::ALL {
            let mut w = World::new(SIZES[0], o.rule());
            match o {
                Opening::Creature(i) => w.place(&decode(CREATURES[i].cells)),
                Opening::Soup => w.soup(SIZES[0] * SIZES[0] / CELLS_PER_PATCH, &mut Rng::new(1)),
                Opening::Life => w.place(&decode(GLIDER_GUN)),
            }
            assert!(w.mass() > 0.0);
        }
    }
}
