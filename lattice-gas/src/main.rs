//! Lattice gas demo: particles colliding on a hexagonal lattice, which flow like a fluid.

mod flow;
mod sim;
mod view;

use ca_core::egui::{self, Color32, RichText, Stroke, vec2};
use ca_core::{
    About, Advance, NAVY, Pace, PaintStroke, Playback, eframe, generations_per_frame, grouped,
};
use flow::{Blocks, Field};
use sim::{Inflow, Kind, Lattice, PARTICLES, SOUND_SPEED, WALL};
use view::{LatticeView, Shown};

fn main() {
    ca_core::run("Lattice gas", GasApp::new);
}

const DUSK: Color32 = Color32::from_rgb(0x1f, 0x2a, 0x55);
const BLUE: Color32 = Color32::from_rgb(0x3a, 0x6e, 0xa5);
const TEAL: Color32 = Color32::from_rgb(0x2a, 0x9d, 0x8f);
const GOLD: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);
const CORAL: Color32 = Color32::from_rgb(0xe7, 0x6f, 0x51);
const SPARK: Color32 = Color32::from_rgb(0xf4, 0xf1, 0xde);
const STONE: Color32 = Color32::from_rgb(0xb8, 0xbc, 0xc8);

/// A node's colour by its particle count, 0 to 6.
const BY_COUNT: [Color32; 7] = [NAVY, DUSK, BLUE, TEAL, GOLD, CORAL, SPARK];

/// Speed, from still to fast.
const SPEED_RAMP: [Color32; 5] = [NAVY, BLUE, TEAL, GOLD, SPARK];

/// World sizes, in nodes. Heights are even, as the wrapping lattice needs.
const SIZES: &[(usize, usize)] = &[(400, 200), (800, 400), (1200, 600)];

/// Columns refilled with fresh gas each step, at the left edge.
const INFLOW_WIDTH: usize = 4;

/// How much cheaper a node-step is here than a cell-generation of Life, which
/// `generations_per_frame`'s budget is sized for (about 11 ns). Measured natively in a
/// release build (`timing_steps`): 0.6 ns per node for the step, 0.4 for the averaging.
/// The browser is slower, so the factor is kept well under the 11 it suggests.
const CHEAPER_THAN_LIFE: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Preset {
    Plate,
    Cylinder,
    Channel,
    Pulse,
}

impl Preset {
    const ALL: [Preset; 4] = [Self::Plate, Self::Cylinder, Self::Channel, Self::Pulse];

    fn label(self) -> &'static str {
        match self {
            Self::Plate => "Plate",
            Self::Cylinder => "Cylinder",
            Self::Channel => "Channel",
            Self::Pulse => "Sound pulse",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::Plate => {
                "A flat plate across the wind. Behind it the flow turns back on itself in two eddies, which grow."
            }
            Self::Cylinder => "A cylinder in the wind. Its wake is smaller than the plate's.",
            Self::Channel => {
                "Wind down a channel with walls along the top and bottom. The gas sticks to them, and the flow slows near them."
            }
            Self::Pulse => {
                "A dense disc of gas at rest in still gas. It spreads as a ring at the speed of sound, as round as on a real gas."
            }
        }
    }

    /// The obstacle's width across the wind, in node spacings, for the Reynolds number.
    fn obstacle_width(self, kind: Kind, h: usize) -> Option<f64> {
        let side = h as f64 * kind.row_height();
        match self {
            Self::Plate => Some(side / 4.0),
            Self::Cylinder => Some(side / 4.0),
            Self::Channel | Self::Pulse => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Show {
    Particles,
    Speed,
    Vorticity,
    Density,
}

impl Show {
    const ALL: [Show; 4] = [Self::Particles, Self::Speed, Self::Vorticity, Self::Density];

    fn label(self) -> &'static str {
        match self {
            Self::Particles => "particles",
            Self::Speed => "speed",
            Self::Vorticity => "swirl",
            Self::Density => "density",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::Particles => {
                "Each node coloured by how many particles it holds, from none (dark) to all six, or on the square lattice four (light): the raw automaton."
            }
            Self::Speed => "The averaged flow's speed: dark is still, white is fast.",
            Self::Vorticity => {
                "The averaged flow's vorticity, how fast it spins: coral anticlockwise, blue clockwise."
            }
            Self::Density => "The averaged density: coral denser than the mean, blue thinner.",
        }
    }
}

struct GasApp {
    lattice: Lattice,
    kind: Kind,
    blocks: Blocks,
    field: Field,
    view: LatticeView,
    about: About,
    playback: Playback,
    pace: Pace,
    size: (usize, usize),
    preset: Preset,
    show: Show,
    arrows: bool,
    /// Width of an averaging block, in nodes.
    block: usize,
    /// The time average's window, in steps.
    window: f64,
    /// Particles per direction per node, 0 to 1: the gas's density.
    density: f64,
    /// The wind's speed, in node spacings per step.
    speed: f64,
    brush: f64,
    stroke: PaintStroke,
    hovered: Option<(usize, usize)>,
    seed: u64,
    dirty: bool,
}

impl eframe::App for GasApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            self.field.blend(&self.blocks, self.window);
            self.blocks.clear();
            let image = self.image();
            let l = &self.lattice;
            let size = (l.width(), l.height());
            self.view.set_image(ui.ctx(), l.kind(), size, image);
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            if self.arrows {
                self.draw_arrows(ui, &shown);
            }
            if shown.hovered != self.hovered {
                // The readout's pointer rows follow at once, even while paused.
                ui.ctx().request_repaint();
            }
            self.hovered = shown.hovered;
            let erase = ui.input(|i| i.modifiers.shift);
            for (x, y) in self.stroke.cells(shown.pressed) {
                self.paint(x, y, !erase);
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl GasApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let block = 16;
        let blocks = blocks(Kind::Hex, size, block);
        let mut app = Self {
            lattice: Lattice::new(Kind::Hex, size.0, size.1, 1986),
            kind: Kind::Hex,
            field: Field::new(&blocks),
            blocks,
            view: LatticeView::new(),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(500, 5_000, "steps / second"),
            pace: Pace::default(),
            size,
            preset: Preset::Plate,
            show: Show::Vorticity,
            arrows: true,
            block,
            window: 100.0,
            density: 0.2,
            speed: 0.3,
            brush: 3.0,
            stroke: PaintStroke::default(),
            hovered: None,
            seed: 1986,
            dirty: true,
        };
        app.restart();
        app.playback.running = true; // the plate's wake is best seen forming
        app
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Lattice gas");
        ui.label("Particles hop on a hexagonal lattice and scatter when they meet. Averaged over many nodes, they flow like a fluid.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ca_core::speed_cap_note(ui, "lattice");
        ui.small("Restart sets up the chosen start again, with fresh random gas.");
        ui.small("Drag on the lattice to draw walls; shift-drag to erase.");
        ui.add(egui::Slider::new(&mut self.brush, 1.0..=20.0).text("brush radius"));
        ui.separator();

        ui.label(RichText::new("Show").strong());
        ui.horizontal_wrapped(|ui| {
            for show in Show::ALL {
                if ui
                    .selectable_label(self.show == show, show.label())
                    .clicked()
                {
                    self.show = show;
                    self.dirty = true;
                }
            }
            ui.checkbox(&mut self.arrows, "Arrows");
        });
        ui.small(self.show.hint());
        egui::CollapsingHeader::new("Averaging").show(ui, |ui| {
            if ui
                .add(egui::Slider::new(&mut self.block, 6..=40).text("nodes per block"))
                .on_hover_text("The width of each block of nodes averaged over.")
                .changed()
            {
                self.blocks = blocks(self.kind, self.size, self.block);
                // Sample the gas as it is, so the flow views aren't blank until the next step.
                self.blocks.add(&self.lattice);
                self.field = Field::new(&self.blocks);
                self.dirty = true;
            }
            ui.add(
                egui::Slider::new(&mut self.window, 1.0..=1_000.0)
                    .logarithmic(true)
                    .text("steps averaged"),
            )
            .on_hover_text("About how many steps each average spans: a moving average.");
        });
        ui.separator();

        ui.label(RichText::new("Lattice").strong());
        ui.horizontal_wrapped(|ui| {
            for (kind, label) in [
                (Kind::Hex, "hexagonal (FHP)"),
                (Kind::Square, "square (HPP)"),
            ] {
                if ui.selectable_label(self.kind == kind, label).clicked() && self.kind != kind {
                    self.kind = kind;
                    self.blocks = blocks(kind, self.size, self.block);
                    self.restart();
                }
            }
        });
        ui.small(match self.kind {
            Kind::Hex => "Six directions: FHP-I, which flows like a real fluid.",
            Kind::Square => "Four directions: the earlier HPP model. Each row keeps its own momentum, so the flow never spreads from row to row.",
        });

        ui.label(RichText::new("Start").strong());
        ui.horizontal_wrapped(|ui| {
            for preset in Preset::ALL {
                if ui
                    .selectable_label(self.preset == preset, preset.label())
                    .clicked()
                {
                    self.preset = preset;
                    // A sound wave moves fast: see it as density, averaged over few steps.
                    // The flows are slow and noisy: average them for longer.
                    if preset == Preset::Pulse {
                        (self.show, self.window) = (Show::Density, 5.0);
                    } else if self.window < 20.0 || self.show == Show::Density {
                        (self.show, self.window) = (Show::Vorticity, 100.0);
                    }
                    self.restart();
                }
            }
        });
        ui.small(self.preset.hint());
        ui.add(egui::Slider::new(&mut self.density, 0.05..=0.5).text("density"))
            .on_hover_text("Particles per direction per node. Takes effect on Restart.");
        ui.add(egui::Slider::new(&mut self.speed, 0.0..=0.4).text("wind"))
            .on_hover_text("The wind's speed, in node spacings per step. Changes the wind at once; the gas already there on Restart.");
        if let Some(inflow) = &mut self.lattice.inflow {
            inflow.speed = self.speed;
            inflow.density = self.density;
        }
        ui.horizontal_wrapped(|ui| {
            ui.label("World:");
            for &size in SIZES {
                let label = format!("{} x {}", size.0, size.1);
                if ui.selectable_label(self.size == size, label).clicked() && self.size != size {
                    self.size = size;
                    self.blocks = blocks(self.kind, size, self.block);
                    self.restart();
                }
            }
        });
        ui.separator();

        self.stats(ui);
    }

    /// The readout at the bottom of the panel.
    fn stats(&self, ui: &mut egui::Ui) {
        let l = &self.lattice;
        ca_core::readout(ui, |r| {
            r.row("step", grouped(l.steps));
            r.row("particles", grouped(l.particles()));
            if let (Some(_), Kind::Hex) = (l.inflow, l.kind()) {
                r.row("Mach number", format!("{:.2}", self.speed / SOUND_SPEED))
                    .on_hover_text(
                        "The wind's speed over the speed of sound, 1/√2 node spacings per step.",
                    );
                if let Some(width) = self.preset.obstacle_width(l.kind(), l.height()) {
                    r.row(
                        "Reynolds number",
                        format!("{:.0}", sim::reynolds(self.speed, width, self.density)),
                    )
                    .on_hover_text("Speed x obstacle width x g / viscosity, with FHP's g and viscosity at this density (Frisch and colleagues, 1987). Higher means more eddies.");
                }
            }
            let flow = self.hovered.map(|(x, y)| {
                let (px, py) = l.position(x, y);
                let f = &self.field;
                let ux = f.at(px, py, |bx, by| f.get(bx, by).ux);
                let uy = f.at(px, py, |bx, by| f.get(bx, by).uy);
                format!("({ux:.2}, {uy:.2})")
            });
            r.row(
                "flow at the pointer",
                flow.unwrap_or_else(|| "-".to_string()),
            );
            r.pointer(self.hovered.map(|(x, y)| {
                let c = l.get(x, y);
                if c & WALL != 0 {
                    "wall".to_string()
                } else {
                    match (c & PARTICLES).count_ones() {
                        1 => "1 particle".to_string(),
                        n => format!("{n} particles"),
                    }
                }
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
                let nodes = self.size.0 * self.size.1;
                let cap = generations_per_frame(nodes / CHEAPER_THAN_LIFE, 200);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            self.lattice.step();
            self.blocks.add(&self.lattice);
        }
        self.dirty |= steps > 0;
    }

    /// The picture for the current view: one colour per node.
    fn image(&self) -> egui::ColorImage {
        picture(&self.lattice, &self.field, self.show, self.speed)
    }

    /// An arrow per block (or per few blocks, if blocks are small on screen), showing the
    /// averaged velocity: a wind's length is about one block.
    fn draw_arrows(&self, ui: &egui::Ui, shown: &Shown) {
        let f = &self.field;
        let b = &self.blocks;
        let block_points = b.bw as f32 * shown.scale();
        if block_points <= 0.0 {
            return;
        }
        let every = (14.0 / block_points).ceil().max(1.0) as usize;
        let reach = block_points * every as f32 / self.speed.max(0.05) as f32 * 0.8;
        let painter = ui.painter_at(shown.rect());
        let stroke = Stroke::new(1.2, Color32::from_rgba_unmultiplied(0xf4, 0xf1, 0xde, 190));
        for by in (0..f.ny).step_by(every) {
            for bx in (0..f.nx).step_by(every) {
                let a = f.get(bx, by);
                let (cx, cy) = b.centre(bx, by);
                // y up in the flow, down on screen.
                let v = vec2(a.ux as f32, -a.uy as f32) * reach;
                if v.length() < 1.5 {
                    continue;
                }
                let tip = shown.to_screen(cx, cy) + v * 0.5;
                painter.arrow(tip - v, v, stroke);
            }
        }
    }

    /// Set the current preset up afresh.
    fn restart(&mut self) {
        let (w, h) = self.size;
        self.seed += 1;
        let mut l = Lattice::new(self.kind, w, h, self.seed);
        let row_height = self.kind.row_height();
        let mid_y = h as f64 * row_height / 2.0;
        let wind = Some(Inflow {
            width: INFLOW_WIDTH,
            density: self.density,
            speed: self.speed,
        });
        let x0 = w as f64 / 4.0;
        let across = self.preset.obstacle_width(self.kind, h).unwrap_or(0.0);
        match self.preset {
            Preset::Plate => {
                l.wall_where(|px, py| (px - x0).abs() <= 1.0 && (py - mid_y).abs() <= across / 2.0);
                l.fill(self.density, self.speed);
                l.inflow = wind;
            }
            Preset::Cylinder => {
                l.wall_where(|px, py| (px - x0).hypot(py - mid_y) <= across / 2.0);
                l.fill(self.density, self.speed);
                l.inflow = wind;
            }
            Preset::Channel => {
                for x in 0..w {
                    l.set_wall(x, 0, true);
                    l.set_wall(x, h - 1, true);
                }
                l.fill(self.density, self.speed);
                l.inflow = wind;
            }
            Preset::Pulse => {
                l.fill(self.density, 0.0);
                let r = h as f64 * row_height / 12.0;
                l.fill_disc(w as f64 / 2.0, mid_y, r, (self.density * 2.5).min(1.0));
            }
        }
        self.lattice = l;
        self.blocks.clear();
        self.blocks.add(&self.lattice);
        self.field = Field::new(&self.blocks);
        self.dirty = true;
    }

    /// Draw (or erase) walls in a disc around node `(x, y)`.
    fn paint(&mut self, x: usize, y: usize, wall: bool) {
        let (cx, cy) = self.lattice.position(x, y);
        let r = self.brush;
        let l = &mut self.lattice;
        let rows = (r / l.kind().row_height()).ceil() as isize + 1;
        for dy in -rows..=rows {
            let ny = y as isize + dy;
            if ny < 0 || ny >= l.height() as isize {
                continue;
            }
            for dx in -(r as isize + 1)..=(r as isize + 1) {
                let nx = x as isize + dx;
                if nx < 0 || nx >= l.width() as isize {
                    continue;
                }
                let (nx, ny) = (nx as usize, ny as usize);
                let (px, py) = l.position(nx, ny);
                if (px - cx).hypot(py - cy) <= r && l.is_wall(nx, ny) != wall {
                    l.set_wall(nx, ny, wall);
                }
            }
        }
        self.dirty = true;
    }
}

/// The picture of `l` for the view `show`: one colour per node, from the particles or from
/// the averaged `field`. `wind` sets the colour scale for speed and swirl; density is drawn
/// relative to the field's own mean, so it stays centred whatever the density slider says.
fn picture(l: &Lattice, f: &Field, show: Show, wind: f64) -> egui::ColorImage {
    let (w, h, cells) = (l.width(), l.height(), l.cells());
    let wind = wind.max(0.05);
    let mean = f.mean_density();
    // Per block, the value to draw, interpolated between blocks for each node.
    let values: Vec<f64> = (0..f.ny)
        .flat_map(|by| (0..f.nx).map(move |bx| (bx, by)))
        .map(|(bx, by)| {
            let a = f.get(bx, by);
            match show {
                Show::Particles => 0.0,
                Show::Speed => a.ux.hypot(a.uy) / (1.5 * wind),
                // Shear across an eddy about 30 nodes wide gives about wind / 15.
                Show::Vorticity => f.vorticity(bx, by) / (wind / 15.0),
                Show::Density => (a.density / mean - 1.0) / 0.25,
            }
        })
        .collect();
    let nodes = match show {
        Show::Particles => Vec::new(),
        _ => f.resample(w, h, &values),
    };
    // Colours for values from -1 to 1 (0 to 1 for speed), in 512 steps.
    let (lo, steps) = (if show == Show::Speed { 0.0 } else { -1.0 }, 511.0);
    let span = 1.0 - lo;
    let colours: Vec<Color32> = (0..=steps as usize)
        .map(|k| {
            let v = lo + span * k as f64 / steps;
            match show {
                Show::Speed => ramp(&SPEED_RAMP, v),
                _ => diverging(v),
            }
        })
        .collect();
    view::image(l.kind(), w, h, |x, y| {
        let c = cells[y * w + x];
        if c & WALL != 0 {
            STONE
        } else if show == Show::Particles {
            BY_COUNT[(c & PARTICLES).count_ones() as usize]
        } else {
            let t = ((nodes[y * w + x] - lo) / span).clamp(0.0, 1.0);
            colours[(t * steps).round() as usize]
        }
    })
}

/// Averaging blocks `block` nodes across and about as tall (on the hex lattice, an even
/// number of rows, so each block holds as many odd rows as even).
fn blocks(kind: Kind, size: (usize, usize), block: usize) -> Blocks {
    let rows = match kind {
        Kind::Hex => ((block as f64 / kind.row_height() / 2.0).round() as usize * 2).max(2),
        Kind::Square => block,
    };
    Blocks::new(kind, size.0, size.1, block, rows)
}

/// `t` from 0 to 1 along a ramp of colours, clamped.
fn ramp(colours: &[Color32], t: f64) -> Color32 {
    let t = t.clamp(0.0, 1.0) * (colours.len() - 1) as f64;
    let i = (t as usize).min(colours.len() - 2);
    mix(colours[i], colours[i + 1], (t - i as f64) as f32)
}

/// `t` from -1 (blue) through 0 (navy) to 1 (coral), clamped.
fn diverging(t: f64) -> Color32 {
    if t < 0.0 {
        ramp(&[NAVY, BLUE, SPARK], -t)
    } else {
        ramp(&[NAVY, CORAL, SPARK], t)
    }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cost of drawing a frame's picture on the biggest world, natively
    /// (`--ignored --nocapture`).
    #[test]
    #[ignore]
    fn timing_picture() {
        let size = SIZES[2];
        let mut l = Lattice::new(Kind::Hex, size.0, size.1, 1);
        l.fill(0.2, 0.3);
        let mut b = blocks(Kind::Hex, size, 16);
        b.add(&l);
        let mut f = Field::new(&b);
        f.blend(&b, 100.0);
        for show in [Show::Particles, Show::Speed, Show::Vorticity] {
            let runs = 10;
            let t = std::time::Instant::now();
            for _ in 0..runs {
                std::hint::black_box(picture(&l, &f, show, 0.3));
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / runs as f64;
            println!("{}: {ms:.1} ms", show.label());
        }
    }
}
