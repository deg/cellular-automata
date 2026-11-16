//! Pieces every demo's panel shares, so their wording and look can't drift apart: the
//! background colour, the note about the speed cap, and the readout grid at the bottom.

use crate::egui::{self, Color32};

/// The background of every demo: an empty cell, the space around a picture, the index
/// page's cards. The rest of the palette is in each demo and in `site/style.css`.
pub const NAVY: Color32 = Color32::from_rgb(0x14, 0x18, 0x30);

/// The note under the speed slider in a demo whose work per frame is capped (see
/// [`crate::generations_per_frame`]). `world` is what the demo calls its world: "world",
/// "field", "lattice".
pub fn speed_cap_note(ui: &mut egui::Ui, world: &str) {
    ui.small(format!(
        "On a big {world} the top speeds are out of reach: each frame does at most about a \
         million cell updates, so the {world} slows down instead of freezing."
    ));
}

/// A whole number with commas between groups of three digits, as the READMEs write them:
/// `1234567` becomes `1,234,567`.
pub fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, d) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(d);
    }
    out
}

/// The readout at the bottom of a demo's panel: a grid of lowercase labels and their values.
///
/// ```ignore
/// ca_core::readout(ui, |r| {
///     r.row("generation", ca_core::grouped(gen));
///     r.pointer(hovered.map(|(x, y)| format!("({x}, {y}): alive")));
/// });
/// ```
pub fn readout(ui: &mut egui::Ui, add_rows: impl FnOnce(&mut Readout<'_>)) {
    egui::Grid::new("readout")
        .num_columns(2)
        .show(ui, |ui| add_rows(&mut Readout { ui }));
}

/// The rows of a [`readout`].
pub struct Readout<'a> {
    ui: &'a mut egui::Ui,
}

impl Readout<'_> {
    /// One row. Returns the label's response, for a hover text explaining it.
    pub fn row(&mut self, label: &str, value: impl Into<egui::WidgetText>) -> egui::Response {
        let response = self.ui.label(label);
        self.ui.label(value);
        self.ui.end_row();
        response
    }

    /// The row every demo with a grid shows: what is under the pointer, or "-" when the
    /// pointer is not on the grid.
    pub fn pointer(&mut self, value: Option<String>) {
        self.row(
            "under the pointer",
            value.unwrap_or_else(|| "-".to_string()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::grouped;

    #[test]
    fn groups_of_three_from_the_right() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1_000), "1,000");
        assert_eq!(grouped(383_263), "383,263");
        assert_eq!(grouped(1_234_567), "1,234,567");
        assert_eq!(grouped(u64::MAX), "18,446,744,073,709,551,615");
    }
}
