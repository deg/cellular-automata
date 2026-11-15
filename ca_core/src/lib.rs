//! Shared pieces for the cellular-automata demos.
//!
//! - [`Grid`]: a rectangular array of cells.
//! - [`GridView`]: draws a grid as one texture and reports which cell the pointer is on.
//! - [`Playback`]: run / pause / single-step controls.
//! - [`run`]: the entry point, which opens a native window or starts in the browser.
//!
//! `eframe` and `egui` are re-exported so every demo uses the same version.

mod app;
mod grid;
mod playback;
mod view;

pub use app::run;
pub use eframe;
pub use eframe::egui;
pub use grid::Grid;
pub use playback::Playback;
pub use view::{GridResponse, GridView};
