//! Reaction-diffusion demo: the Gray–Scott model, with John Pearson's map of its patterns.

mod sim;

use ca_core::egui::{self, Align2, Color32, FontId, RichText, vec2};
use ca_core::{
    About, Advance, Grid, GridView, NAVY, Pace, PaintStroke, Playback, Plot, Rng, Series, Style,
    eframe, generations_per_frame, grouped, signed,
};
use sim::{CLASSES, Dish, FIGURE_3, Found, Rates, SEED_UV};

fn main() {
    ca_core::run("Reaction-diffusion", RdApp::new);
}

const BLUE: Color32 = Color32::from_rgb(0x3a, 0x6e, 0xa5);
const TEAL: Color32 = Color32::from_rgb(0x2a, 0x9d, 0x8f);
const GOLD: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);
const CORAL: Color32 = Color32::from_rgb(0xe7, 0x6f, 0x51);

/// U's colour, from 1 (navy: nothing has happened here) down to [`U_LOW`] (gold).
const RAMP: [Color32; 4] = [NAVY, BLUE, TEAL, GOLD];
/// About the lowest U reaches: Pearson's figures run from U = 1 down to "U ≈ .2" (p. 4).
const U_LOW: f32 = 0.2;
/// Shades in the colour table.
const SHADES: usize = 256;

/// World sizes, in cells of Pearson's mesh; his was 256 x 256.
const SIZES: &[usize] = &[128, 256, 512];

/// The (k, F) window of Pearson's figure 3, which the sliders and the map cover.
const K_RANGE: (f32, f32) = (0.03, 0.07);
const F_RANGE: (f32, f32) = (0.0, 0.08);
/// The sliders and the map move F and k in steps of this: figure 3's points all lie on it.
const RATE_STEP: f64 = 0.0005;

/// How much cheaper a cell-step is here than a cell-generation of Life, which
/// `generations_per_frame`'s budget is sized for (about 11 ns). Measured natively in a
/// release build (`timing_steps`): about 0.6 ns. The browser is slower, so the factor is
/// kept well under the 18 that suggests.
const CHEAPER_THAN_LIFE: usize = 8;

/// The map's height in the panel, labels included.
const MAP_HEIGHT: f32 = 170.0;

struct RdApp {
    dish: Dish,
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

impl eframe::App for RdApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            let image = shades(&self.dish);
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
                let uv = if ui.input(|i| i.modifiers.shift) {
                    (1.0, 0.0)
                } else {
                    SEED_UV
                };
                for (x, y) in cells {
                    self.dish.paint_disc(x, y, self.brush, uv);
                }
                self.dirty = true;
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl RdApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let mut app = Self {
            dish: Dish::new(size, size, CLASSES[4].rates),
            view: GridView::new("reaction-diffusion"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(5_000, 20_000, "steps / second"),
            pace: Pace::default(),
            colours: palette(),
            size,
            rng: Rng::new(1993),
            brush: 5.0,
            stroke: PaintStroke::default(),
            hovered: None,
            dirty: true,
        };
        app.restart();
        app.playback.running = true; // the square growing into spots is the show
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Reaction-diffusion");
        ui.label("Not a cellular automaton: two chemicals spread through a dish and react. V feeds on U to make more V; U is fed in, and V removed. By those two rates, a small square of V grows into spots, stripes, waves or chaos.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ca_core::speed_cap_note(ui, "dish");
        ui.small(
            "Restart puts Pearson's square of V back in the middle of the dish, with fresh noise.",
        );
        ui.separator();

        ui.label(RichText::new("Pearson's patterns").strong());
        ui.horizontal_wrapped(|ui| {
            for class in &CLASSES {
                if ui
                    .selectable_label(same(self.dish.rates, class.rates), class.letter)
                    .clicked()
                {
                    self.dish.rates = class.rates;
                    self.restart();
                }
            }
        });
        let rates = self.dish.rates;
        let note = match CLASSES.iter().find(|c| same(rates, c.rates)) {
            Some(class) => format!("{} {}", class.letter, class.note),
            None => "Between Pearson's points: choose a letter, or click the map.".into(),
        };
        // Two lines' room whatever the note, so the map below doesn't jump under a drag.
        let room = 2.0 * ui.text_style_height(&egui::TextStyle::Small);
        ui.allocate_ui(vec2(ui.available_width(), room), |ui| {
            ui.set_min_height(room);
            ui.small(note);
        });
        let mut f = rates.f;
        let mut k = rates.k;
        ui.add(
            egui::Slider::new(&mut f, F_RANGE.0..=F_RANGE.1)
                .step_by(RATE_STEP)
                .fixed_decimals(4)
                .text("feed F"),
        );
        ui.add(
            egui::Slider::new(&mut k, K_RANGE.0..=K_RANGE.1)
                .step_by(RATE_STEP)
                .fixed_decimals(4)
                .text("kill k"),
        );
        if let Some((mk, mf)) = map(ui, Rates { f, k }) {
            (k, f) = (mk, mf);
        }
        ui.small("Pearson's map: a letter wherever he found that pattern (large at the presets), dots where the dish went plain (coral: back to U = 1, blue: the other uniform state). Click or drag to move; the dish carries on.");
        self.dish.rates = Rates { f, k };
        ui.separator();

        ui.label(RichText::new("Draw").strong());
        ui.small("Drag on the dish to add V; shift-drag to erase.");
        ui.add(egui::Slider::new(&mut self.brush, 1.0..=20.0).text("brush radius"));
        if ui.button("Clear").clicked() {
            self.dish.clear();
            self.dirty = true;
        }
        ui.small("Clear empties the dish, ready to draw on.");
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

        let d = &self.dish;
        ca_core::readout(ui, |r| {
            r.row("step", grouped(d.steps));
            r.row(
                "change per step",
                d.change().map_or("-".to_string(), scientific),
            )
            .on_hover_text(
                "The mean change in U over the last step: near 0 once the pattern stands still.",
            );
            r.pointer(self.hovered.map(|(x, y)| {
                let (u, v) = d.get(x, y);
                format!(
                    "({x}, {y}): u {}, v {}",
                    signed(f64::from(u)),
                    signed(f64::from(v))
                )
            }));
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
                let cap = generations_per_frame(self.size * self.size / CHEAPER_THAN_LIFE, 1_000);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            self.dish.step();
        }
        self.dirty |= steps > 0;
    }

    /// Pearson's opening at the current rates and size, with fresh noise.
    fn restart(&mut self) {
        let rates = self.dish.rates;
        self.dish = Dish::new(self.size, self.size, rates);
        self.dish.pearson_opening(&mut self.rng);
        self.dirty = true;
    }
}

/// Whether two settings of the rates are the same, allowing for the sliders' rounding.
fn same(a: Rates, b: Rates) -> bool {
    (a.f - b.f).abs() < 1e-6 && (a.k - b.k).abs() < 1e-6
}

/// The dish as shades: U from 1 down to [`U_LOW`], as indices into [`palette`].
fn shades(d: &Dish) -> Grid<u8> {
    let mut g = Grid::new(d.width(), d.height(), 0u8);
    let scale = (SHADES - 1) as f32 / (1.0 - U_LOW);
    for (s, &u) in g.cells_mut().iter_mut().zip(d.u()) {
        *s = ((1.0 - u) * scale).round().clamp(0.0, (SHADES - 1) as f32) as u8;
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

/// Pearson's figure 3 as a small chart, `k` across and `F` up: a letter at each point where
/// he found a pattern (larger at the presets), his uniform outcomes as dots, and a marker at
/// the current rates. Returns new rates `(k, F)` if it was clicked or dragged on.
fn map(ui: &mut egui::Ui, now: Rates) -> Option<(f32, f32)> {
    let found = |want: Found| -> Vec<(f64, f64)> {
        FIGURE_3
            .iter()
            .filter(|p| p.2 == want)
            .map(|&(k, f, _)| (f64::from(k), f64::from(f)))
            .collect()
    };
    let (red, blue) = (found(Found::Red), found(Found::Blue));
    let here = [(f64::from(now.k), f64::from(now.f))];
    let shown = ca_core::plot(
        ui,
        &Plot {
            x_label: "kill k",
            y_label: "feed F",
            log_x: false,
            log_y: false,
            x_range: Some((f64::from(K_RANGE.0), f64::from(K_RANGE.1))),
            y_range: Some((f64::from(F_RANGE.0), f64::from(F_RANGE.1))),
            series: vec![
                Series {
                    points: &red,
                    colour: CORAL,
                    style: Style::Dots,
                },
                Series {
                    points: &blue,
                    colour: BLUE,
                    style: Style::Dots,
                },
                Series {
                    points: &here,
                    colour: GOLD,
                    style: Style::Marker,
                },
            ],
        },
        MAP_HEIGHT,
    );
    let screen = |k: f32, f: f32| shown.screen_of((f64::from(k), f64::from(f)));
    let dots: Vec<_> = red
        .iter()
        .chain(&blue)
        .map(|&p| shown.screen_of(p))
        .collect();
    let painter = ui.painter();
    // The letter on the marker, in the text's colour to stand out on its gold.
    let on_marker = ui.visuals().strong_text_color();
    for &(k, f, what) in FIGURE_3 {
        let Found::Pattern(i) = what else { continue };
        let class = &CLASSES[i];
        let preset = class.rates.k == k && class.rates.f == f;
        let mut at = screen(k, f);
        // A dot just above (ι has a B close over it): move the letter right, clear of it
        // and of the letters in the same column.
        if dots.iter().any(|d| d.distance(at) < 4.0) {
            at.x += 5.0;
        }
        painter.text(
            at,
            Align2::CENTER_CENTER,
            class.letter,
            FontId::proportional(if preset { 13.0 } else { 9.0 }),
            if preset && same(now, class.rates) {
                on_marker
            } else if preset {
                TEAL
            } else {
                TEAL.gamma_multiply(0.7)
            },
        );
    }

    let r = &shown.response;
    if !(r.clicked() || r.dragged()) {
        return None;
    }
    let pos = r.interact_pointer_pos()?;
    let value = shown.value_at(pos)?;
    Some(snap(pos, value, |p| shown.screen_of(p)))
}

/// Within this many points of one of figure 3's points, a click on the map picks that point.
const SNAP_RADIUS: f32 = 6.0;

/// The rates `(k, F)` for a click at `pos`, which is at `value` on the map: the nearest point
/// of figure 3 if one is within [`SNAP_RADIUS`] on the screen (`screen_of` maps a point there),
/// otherwise `value` rounded to the sliders' steps. A letter is too small to hit by its
/// centre, and rounding alone would land beside it.
fn snap(
    pos: egui::Pos2,
    value: (f64, f64),
    screen_of: impl Fn((f64, f64)) -> egui::Pos2,
) -> (f32, f32) {
    let nearest = FIGURE_3
        .iter()
        .map(|&(k, f, _)| (k, f, screen_of((f64::from(k), f64::from(f))).distance(pos)))
        .min_by(|a, b| a.2.total_cmp(&b.2));
    if let Some((k, f, d)) = nearest
        && d <= SNAP_RADIUS
    {
        return (k, f);
    }
    let round = |x: f64, (lo, hi): (f32, f32)| {
        ((x.clamp(f64::from(lo), f64::from(hi)) / RATE_STEP).round() * RATE_STEP) as f32
    };
    (round(value.0, K_RANGE), round(value.1, F_RANGE))
}

/// A small positive number as "3.2 × 10⁻⁶", or "0".
fn scientific(x: f64) -> String {
    if x <= 0.0 {
        return "0".to_string();
    }
    let mut exp = x.log10().floor() as i32;
    let mut mantissa = format!("{:.1}", x / 10f64.powi(exp));
    if mantissa == "10.0" {
        exp += 1;
        mantissa = "1.0".to_string();
    }
    let digits: String = exp
        .abs()
        .to_string()
        .chars()
        .map(|c| {
            "⁰¹²³⁴⁵⁶⁷⁸⁹"
                .chars()
                .nth(c.to_digit(10).unwrap() as usize)
                .unwrap()
        })
        .collect();
    let sign = if exp < 0 { "⁻" } else { "" };
    format!("{mantissa} × 10{sign}{digits}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The change per step reads as a power of ten, rounding up into the next power cleanly.
    #[test]
    fn scientific_reads_as_a_power_of_ten() {
        assert_eq!(scientific(3.24e-6), "3.2 × 10⁻⁶");
        assert_eq!(scientific(0.000_999_9), "1.0 × 10⁻³");
        assert_eq!(scientific(0.0012), "1.2 × 10⁻³");
        assert_eq!(scientific(0.0), "0");
    }

    /// U = 1, the dish untouched, is navy like every demo's background; the lowest U is
    /// the top of the ramp.
    #[test]
    fn untouched_dish_is_navy() {
        let p = palette();
        let mut d = Dish::new(4, 4, CLASSES[0].rates);
        d.set(1, 1, (U_LOW, 0.3));
        let s = shades(&d);
        assert_eq!(p[usize::from(s.get(0, 0))], NAVY);
        assert_eq!(p[usize::from(s.get(1, 1))], GOLD);
    }

    /// A click near a point of figure 3 picks it exactly: on the panel's map a letter's
    /// centre is hard to hit, and plain rounding lands a step beside it. Further away, the
    /// click rounds to the sliders' steps. The mapping here is a 100-point square map.
    #[test]
    fn a_click_near_a_letter_picks_its_point() {
        let screen_of = |(k, f): (f64, f64)| {
            egui::pos2(
                ((k - 0.03) / 0.04 * 100.0) as f32,
                (100.0 - f / 0.08 * 100.0) as f32,
            )
        };
        let value_at = |p: egui::Pos2| {
            (
                0.03 + f64::from(p.x) / 100.0 * 0.04,
                f64::from(100.0 - p.y) / 100.0 * 0.08,
            )
        };
        let lambda = CLASSES[10].rates;
        let centre = screen_of((f64::from(lambda.k), f64::from(lambda.f)));
        let near = centre + vec2(3.0, -2.0);
        assert_eq!(snap(near, value_at(near), screen_of), (lambda.k, lambda.f));
        let far = egui::pos2(10.0, 90.0);
        assert_eq!(snap(far, value_at(far), screen_of), (0.034, 0.008));
    }

    /// Every preset lies on the sliders' grid and inside the map, so choosing one
    /// highlights its chip and its letter.
    #[test]
    fn presets_sit_on_the_sliders() {
        for c in &CLASSES {
            let Rates { f, k } = c.rates;
            for (x, (lo, hi)) in [(f, F_RANGE), (k, K_RANGE)] {
                assert!((lo..=hi).contains(&x), "{}", c.letter);
                let steps = f64::from(x) / RATE_STEP;
                assert!((steps - steps.round()).abs() < 1e-3, "{}", c.letter);
            }
        }
    }

    /// The cost of a step natively (`--ignored --nocapture`), for [`CHEAPER_THAN_LIFE`].
    #[test]
    #[ignore]
    fn timing_steps() {
        for &n in SIZES {
            let mut d = Dish::new(n, n, CLASSES[4].rates);
            d.pearson_opening(&mut Rng::new(1));
            let steps = 4_000_000 / (n * n) + 100;
            let t = std::time::Instant::now();
            for _ in 0..steps {
                d.step();
            }
            let ns = t.elapsed().as_secs_f64() * 1e9 / (steps * n * n) as f64;
            let t = std::time::Instant::now();
            std::hint::black_box(shades(&d));
            let ms = t.elapsed().as_secs_f64() * 1e3;
            println!("{n} x {n}: {ns:.2} ns a cell-step; picture {ms:.2} ms");
        }
    }
}
