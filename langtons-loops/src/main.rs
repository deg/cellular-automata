//! Langton's loops demo: a loop of signals builds a copy of itself, and the copies a colony.

mod framing;
mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, Grid, GridView, Pace, PaintStroke, Playback, eframe, generations_per_frame,
};
use framing::Frame;
use sim::{LANGTON_LOOP, Loops, Rule, STATES, pattern};

fn main() {
    ca_core::run("Langton's loops", LoopsApp::new);
}

/// One colour per state. The background and the 0s between signals are the same navy, which
/// is what makes each "7 0" or "4 0" pair read as one signal. The sheath and the core, the
/// loop's body, are muted; the two genome signals are the brightest; the messengers that
/// manage the copying, and the marker, get colours of their own.
const COLOURS: [Color32; STATES] = [
    Color32::from_rgb(0x14, 0x18, 0x30), // 0 empty: navy
    Color32::from_rgb(0x2a, 0x9d, 0x8f), // 1 core: teal
    Color32::from_rgb(0x4a, 0x4f, 0x80), // 2 sheath: muted indigo
    Color32::from_rgb(0x5a, 0xb4, 0xff), // 3 marker: sky blue
    Color32::from_rgb(0xe7, 0x6f, 0x51), // 4 turn signal: coral
    Color32::from_rgb(0xd1, 0x6b, 0xa5), // 5 pull the arm back: pink
    Color32::from_rgb(0xf4, 0xf1, 0xde), // 6 new arm: spark white
    Color32::from_rgb(0xe9, 0xc4, 0x6a), // 7 extend signal: gold
];

const NAMES: [&str; STATES] = [
    "empty",
    "core",
    "sheath",
    "marker",
    "turn left",
    "pull back",
    "new arm",
    "extend",
];

const SIZES: &[(usize, usize)] = &[(300, 260), (600, 520), (1200, 1040)];

/// The narrowest view when following the colony: about four loops across.
const MIN_VIEW: usize = 60;

struct LoopsApp {
    loops: Loops,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    size: (usize, usize),
    /// Show only the colony and its surroundings, zooming out as it grows.
    follow: bool,
    /// The part of the world shown.
    frame: Frame,
    brush: u8,
    stroke: PaintStroke,
    hovered: Option<(usize, usize)>,
    /// Whether a paint stroke is held: the frame holds still meanwhile.
    painting: bool,
    dirty: bool,
}

impl eframe::App for LoopsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            if self.follow {
                self.frame = framing::next(
                    self.frame,
                    self.loops.bounds(),
                    self.size,
                    MIN_VIEW,
                    self.painting,
                );
            }
            let shown = self.shown();
            self.view
                .set_image(ui.ctx(), &shown, |s| COLOURS[s as usize]);
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            let frame = self.frame;
            self.hovered = shown.hovered.map(|c| frame.to_world(c));
            // Once the stroke ends, let the frame catch up with what was painted.
            self.dirty |= self.painting && shown.pressed.is_none();
            self.painting = shown.pressed.is_some();
            // In world cells, so the stroke stays joined up even if the frame does move.
            let cells = self.stroke.cells(shown.pressed.map(|c| frame.to_world(c)));
            if !cells.is_empty() {
                let state = if ui.input(|i| i.modifiers.shift) {
                    0
                } else {
                    self.brush
                };
                for (x, y) in cells {
                    self.loops.set(x, y, state);
                }
                self.dirty = true;
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl LoopsApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let mut app = Self {
            loops: Loops::new(size.0, size.1, Rule::langton()),
            view: GridView::new("loops"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(30, 2_000, "generations / second"),
            pace: Pace::default(),
            size,
            follow: true,
            frame: Frame::whole(size),
            brush: 7,
            stroke: PaintStroke::default(),
            hovered: None,
            painting: false,
            dirty: true,
        };
        app.restart();
        app.playback.running = true; // the loop is best seen copying itself
        app
    }

    /// A new world holding one loop in the middle.
    fn restart(&mut self) {
        let (w, h) = self.size;
        self.loops = Loops::new(w, h, Rule::langton());
        let start = pattern(&LANGTON_LOOP);
        let (pw, ph) = (start[0].len(), start.len());
        self.loops.place((w - pw) / 2, (h - ph) / 2, &start);
        self.reframe();
    }

    /// Choose the view from scratch: close in on the colony, or the whole world.
    fn reframe(&mut self) {
        self.frame = if self.follow {
            framing::follow(None, self.loops.bounds(), self.size, MIN_VIEW)
        } else {
            Frame::whole(self.size)
        };
        self.dirty = true;
    }

    /// The cells of the world in view.
    fn shown(&self) -> Grid<u8> {
        let f = self.frame;
        let grid = self.loops.grid();
        let mut shown = Grid::new(f.w, f.h, 0);
        for y in 0..f.h {
            let row = &grid.cells()[grid.index_of(f.x, f.y + y)..][..f.w];
            let start = shown.index_of(0, y);
            shown.cells_mut()[start..start + f.w].copy_from_slice(row);
        }
        shown
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Langton's loops");
        ui.label("Each cell is in one of 8 states and sees only its four nearest neighbours. Christopher Langton chose 219 rules so that a loop carrying a train of signals builds a copy of itself, which builds another, and so on.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small(
            "The first copy is complete at generation 151. Restart brings back the single loop.",
        );
        ui.separator();

        if ui
            .checkbox(&mut self.follow, "Follow the colony")
            .on_hover_text("Show just the colony, zooming out as it grows. Off: the whole world.")
            .changed()
        {
            self.reframe();
        }
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
        ui.small("The colony grows until it reaches the edge of the world. Beyond the edge every cell is empty.");
        ui.separator();

        ui.label(RichText::new("Draw").strong());
        // Two brushes a row, each a colour swatch and its name.
        egui::Grid::new("brushes").num_columns(2).show(ui, |ui| {
            for (state, name) in NAMES.iter().enumerate() {
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, COLOURS[state]);
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(1.0, Color32::GRAY),
                        egui::StrokeKind::Inside,
                    );
                    ui.selectable_value(&mut self.brush, state as u8, format!("{state} {name}"));
                });
                if state % 2 == 1 {
                    ui.end_row();
                }
            }
        });
        ui.small("Drag on the grid to paint the chosen state; shift-drag erases. A loop with a broken sheath or genome goes wrong in its own way.");
        ui.separator();

        egui::Grid::new("stats").num_columns(2).show(ui, |ui| {
            ui.label("generation");
            ui.label(self.loops.generation.to_string());
            ui.end_row();
            ui.label("colony");
            ui.label(match self.loops.bounds() {
                Some(b) => format!("{} x {} cells", b.width(), b.height()),
                None => "empty".to_string(),
            });
            ui.end_row();
            ui.label("under the pointer");
            ui.label(match self.hovered {
                Some((x, y)) => {
                    let s = self.loops.grid().get(x, y);
                    format!("{s} {}", NAMES[s as usize])
                }
                None => "-".to_string(),
            });
            ui.end_row();
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
                let cap = generations_per_frame(self.loops.work(), 500);
                self.pace.steps(per_second as f64, dt, cap)
            }
        };
        for _ in 0..steps {
            self.loops.step();
        }
        self.dirty |= steps > 0;
    }
}
