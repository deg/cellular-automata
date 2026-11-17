//! Percolation demo: cells open as p rises, clusters merge, and one comes to span the grid.

mod sim;

use ca_core::egui::{self, Color32, RichText};
use ca_core::{
    About, Advance, Grid, GridView, NAVY, Pace, Playback, Plot, Series, Style, eframe, grouped,
    mix, readout,
};
use sim::{Percolation, TICKS};

fn main() {
    ca_core::run("Percolation", PercolationApp::new);
}

const TEAL: Color32 = Color32::from_rgb(0x2a, 0x9d, 0x8f);
const BLUE: Color32 = Color32::from_rgb(0x3a, 0x6e, 0xa5);
const CORAL: Color32 = Color32::from_rgb(0xe7, 0x6f, 0x51);
const SKY: Color32 = Color32::from_rgb(0x5a, 0xb4, 0xff);
const GOLD: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);
const SPARK: Color32 = Color32::from_rgb(0xf4, 0xf1, 0xde);

/// The threshold of the infinite square lattice, p_c = 0.59274605079210(2): Jacobsen,
/// Journal of Physics A 48, 454003 (2015), arXiv:1507.03027.
const P_C: f64 = 0.592_746_050_792_10;

/// Square worlds: the chance of spanning depends on the grid's shape, and the results the
/// README quotes (Ziff's 1/2, Mertens's medians) are for squares.
const SIZES: &[usize] = &[100, 200, 400, 800];

struct PercolationApp {
    grid: Percolation,
    view: GridView,
    about: About,
    playback: Playback,
    pace: Pace,
    size: usize,
    seed: u64,
    hovered: Option<(usize, usize)>,
    /// A cell of the cluster clicked, which follows that cluster as it merges.
    selected: Option<usize>,
    /// Colours for clusters other than the largest and the selected one.
    palette: [Color32; 6],
    image: Grid<Color32>,
    dirty: bool,
}

impl eframe::App for PercolationApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ca_core::controls_panel(ui, |ui| self.controls(ui));
        self.about.show(ui);

        let dt = ui.input(|i| f64::from(i.stable_dt).min(0.1));
        self.advance(dt);
        if self.dirty {
            self.draw();
            self.view.set_image(ui.ctx(), &self.image, |c| c);
            self.dirty = false;
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let shown = self.view.show(ui);
            if shown.hovered != self.hovered {
                // Redraw the "under the pointer" row now, not at the next event.
                self.hovered = shown.hovered;
                ui.ctx().request_repaint();
            }
            if let Some((x, y)) = shown.clicked {
                self.click(x, y);
            }
        });

        if self.playback.running || self.dirty {
            ui.ctx().request_repaint();
        }
    }
}

impl PercolationApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let size = SIZES[1];
        let mut app = Self {
            grid: Percolation::new(size, size, 1),
            view: GridView::new("percolation"),
            about: About::new(include_str!("../README.md")),
            playback: Playback::new(50, 1_000, "steps / second"),
            pace: Pace::default(),
            size,
            seed: 1,
            hovered: None,
            selected: None,
            palette: [
                TEAL,
                BLUE,
                CORAL,
                SKY,
                mix(TEAL, NAVY, 0.45),
                mix(CORAL, NAVY, 0.45),
            ],
            image: Grid::new(size, size, NAVY),
            dirty: true,
        };
        app.restart();
        // The grid filling in, and its clusters merging until one spans it, is the show.
        app.playback.running = true;
        app
    }

    /// Every cell closed, p = 0, with new random numbers for the cells.
    fn restart(&mut self) {
        self.seed += 1;
        self.grid = Percolation::new(self.size, self.size, self.seed);
        self.image = Grid::new(self.size, self.size, NAVY);
        self.selected = None;
        self.dirty = true;
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Percolation");
        ui.label("Each cell is open with chance p. Open cells that share an edge form clusters, and near p = 0.593 a cluster first reaches from the top to the bottom.");
        self.about.toggle_button(ui);
        ui.separator();

        self.playback.ui(ui);
        ui.small("A step raises p by 0.001; Run raises it to 1. Each cell keeps its own random number and opens once p passes it, so the grid only fills in. Restart closes every cell and draws new numbers.");
        ui.label("The chance a cell is open, p");
        let mut p = self.grid.p();
        if ui
            .add(
                egui::Slider::new(&mut p, 0.0..=1.0)
                    .step_by(0.001)
                    .fixed_decimals(3),
            )
            .changed()
        {
            self.grid.set_p(p);
            self.dirty = true;
        }
        ui.separator();

        self.chart(ui);
        ui.separator();

        ui.label(RichText::new("Clusters").strong());
        ui.small("Gold: the largest cluster. Click an open cell to light up its cluster; click it again, or a closed cell, to put it out.");
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
        ui.small(
            "The edges don't wrap. A cluster spans when it joins the top row to the bottom row.",
        );
        ui.separator();

        let largest = self.grid.largest_size();
        let spans_at = self.grid.spans_at();
        let hovered = self
            .hovered
            .map(|(x, y)| (x, y, self.grid.cluster_size(x, y)));
        readout(ui, |r| {
            r.row("p", format!("{:.3}", self.grid.p()));
            r.row("open cells", grouped(self.grid.open_cells() as u64));
            r.row("clusters", grouped(self.grid.clusters() as u64));
            r.row("largest cluster", cells(largest));
            r.row(
                "spans",
                match spans_at {
                    Some(at) => format!("from p = {at:.4}"),
                    None => "no".to_string(),
                },
            );
            // Not "open, cluster of 640,000 cells": the row must fit the panel.
            r.pointer(hovered.map(|(x, y, size)| match size {
                Some(n) => format!("({x}, {y}): cluster of {}", grouped(n as u64)),
                None => format!("({x}, {y}): closed"),
            }));
        });
    }

    /// The largest cluster's share of the grid against p, as p rose, with the threshold.
    fn chart(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("The largest cluster").strong());
        let cells = (self.grid.width() * self.grid.height()) as f64;
        let now = (self.grid.p(), self.grid.largest_size() as f64 / cells);
        let mut line: Vec<(f64, f64)> = self
            .grid
            .samples()
            .iter()
            .enumerate()
            .map(|(k, &s)| (k as f64 / TICKS as f64, s))
            .collect();
        line.push(now);
        let threshold = [(P_C, 0.0), (P_C, 1.0)];
        ca_core::plot(
            ui,
            &Plot {
                x_label: "p",
                y_label: "share of the grid",
                log_x: false,
                log_y: false,
                x_range: Some((0.0, 1.0)),
                y_range: Some((0.0, 1.0)),
                series: vec![
                    Series {
                        points: &threshold,
                        colour: GOLD,
                        style: Style::Line,
                    },
                    Series {
                        points: &line,
                        colour: SKY,
                        style: Style::Line,
                    },
                    Series {
                        points: &[now],
                        colour: SKY,
                        style: Style::Marker,
                    },
                ],
            },
            120.0,
        );
        ui.small("Blue: the largest cluster's share of all the cells as p rose. Gold: the threshold of an endless grid, p\u{a0}=\u{a0}0.59275.");
    }

    fn advance(&mut self, dt: f64) {
        if self.playback.take_restart() {
            self.restart();
        }
        let steps = match self.playback.advance() {
            Advance::Idle => 0,
            Advance::Step => 1,
            Advance::Run(per_second) => self.pace.steps(per_second as f64, dt, TICKS),
        };
        if steps == 0 || self.grid.p() >= 1.0 {
            if self.grid.p() >= 1.0 {
                self.playback.running = false;
            }
            return;
        }
        // Whole ticks stay whole, so the readout shows 0.593, not 0.59299999.
        let mut ticks = self.grid.p() * TICKS as f64 + steps as f64;
        if (ticks - ticks.round()).abs() < 1e-6 {
            ticks = ticks.round();
        }
        self.grid.set_p(ticks / TICKS as f64);
        self.dirty = true;
    }

    fn click(&mut self, x: usize, y: usize) {
        let i = y * self.grid.width() + x;
        let clicked = self.grid.root_of(i);
        let current = self.selected.and_then(|s| self.grid.root_of(s));
        self.selected = match clicked {
            Some(root) if current != Some(root) => Some(i),
            _ => None,
        };
        self.dirty = true;
    }

    /// Closed cells navy; the clicked cluster white, the largest gold, and the rest in
    /// colours picked by their roots, so a cluster keeps its colour as smaller ones join it.
    fn draw(&mut self) {
        let largest = self.grid.largest_root();
        let selected = self.selected.and_then(|s| self.grid.root_of(s));
        if selected.is_none() {
            self.selected = None;
        }
        for i in 0..self.image.len() {
            self.image[i] = match self.grid.root_of(i) {
                None => NAVY,
                Some(r) if Some(r) == selected => SPARK,
                Some(r) if Some(r) == largest => GOLD,
                Some(r) => {
                    let hash = (r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32;
                    self.palette[(hash % self.palette.len() as u64) as usize]
                }
            };
        }
    }
}

/// "1 cell", "1,234 cells".
fn cells(n: usize) -> String {
    if n == 1 {
        "1 cell".to_string()
    } else {
        format!("{} cells", grouped(n as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::cells;

    #[test]
    fn cell_counts_read_as_words() {
        assert_eq!(cells(0), "0 cells");
        assert_eq!(cells(1), "1 cell");
        assert_eq!(cells(1_234), "1,234 cells");
    }
}
