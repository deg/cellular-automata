//! Shared pieces for the cellular-automata demos.
//!
//! - [`Grid`]: a rectangular array of cells.
//! - [`GridView`]: draws a grid as one texture and reports which cell the pointer is on.
//! - [`Playback`]: run / pause / step / restart controls.
//! - [`About`]: a side panel rendering the demo's `README.md`.
//! - [`controls_panel`]: the scrolling left panel every demo puts its controls in.
//! - [`PaintStroke`]: a held pointer as solid lines of cells, for drawing on the grid.
//! - [`Pace`]: steps per second to steps per frame, for automata watched slowly.
//! - [`Rng`]: a small seeded random generator, for random fills.
//! - [`line`]: the cells on a straight line, so drag-painting leaves no gaps.
//! - [`HEX_NEIGHBOURS`]: the six neighbours of a cell in [`Layout::Hex`]'s coordinates.
//! - [`moore_counts`]: each cell's count of neighbours in some state, edges handled once.
//! - [`run`]: the entry point, which opens a native window or starts in the browser.
//!
//! `eframe` and `egui` are re-exported so every demo uses the same version.

mod about;
mod app;
mod controls;
mod grid;
mod neighbours;
mod pace;
mod playback;
mod rng;
mod view;

pub use about::About;
pub use app::run;
pub use controls::{PaintStroke, controls_panel};
pub use eframe;
pub use eframe::egui;
pub use grid::{Grid, line};
pub use neighbours::moore_counts;
pub use pace::{Pace, generations_per_frame};
pub use playback::{Advance, Playback};
pub use rng::Rng;
pub use view::{GridResponse, GridView, HEX_NEIGHBOURS, Layout};
