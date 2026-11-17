//! Pieces every demo's panel shares, so their wording and look can't drift apart: the
//! background colour, the note about the speed cap, and the readout grid at the bottom.

use crate::egui::{self, Color32};

/// The background of every demo: an empty cell, the space around a picture, the index
/// page's cards. The rest of the palette is in each demo and in `site/style.css`.
pub const NAVY: Color32 = Color32::from_rgb(0x14, 0x18, 0x30);

/// A chart's measured points or curve (see [`crate::plot`]), the same in every demo.
pub const CHART_MEASURED: Color32 = Color32::from_rgb(0x5a, 0xb4, 0xff);
/// A chart's reference: a theory line, a fit, a published value.
pub const CHART_REFERENCE: Color32 = Color32::from_rgb(0xe9, 0xc4, 0x6a);

/// The colour `t` of the way from `a` to `b` (0 gives `a`, 1 gives `b`), each channel rounded.
/// The demos shade a value between two palette colours with it.
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// The colour scale for a continuous field (reaction-diffusion, Lenia): navy, blue, teal,
/// gold, as `n` shades from the low end to the high. A demo picks which end is "nothing
/// here" by how it maps its values onto the table.
pub fn field_palette(n: usize) -> Vec<Color32> {
    const RAMP: [Color32; 4] = [
        NAVY,
        Color32::from_rgb(0x3a, 0x6e, 0xa5),
        Color32::from_rgb(0x2a, 0x9d, 0x8f),
        Color32::from_rgb(0xe9, 0xc4, 0x6a),
    ];
    (0..n)
        .map(|i| {
            let t = i as f32 / (n.max(2) - 1) as f32 * (RAMP.len() - 1) as f32;
            let j = (t as usize).min(RAMP.len() - 2);
            mix(RAMP[j], RAMP[j + 1], t - j as f32)
        })
        .collect()
}

/// The note under the speed slider in a demo whose work per frame is capped (see
/// [`crate::generations_per_frame`]). It names no number: each demo sizes its budget by its
/// own cost per cell. `world` is what the demo calls its world: "world",
/// "field", "lattice".
pub fn speed_cap_note(ui: &mut egui::Ui, world: &str) {
    ui.small(format!(
        "On a big {world} the top speeds are out of reach: each frame does a fixed amount of \
         work, so the {world} slows down instead of freezing."
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

/// A number to two decimal places with a true minus sign (U+2212) when negative, and never
/// "-0.00": a value that rounds to zero shows as "0.00".
pub fn signed(v: f64) -> String {
    let text = format!("{:.2}", v.abs());
    if v < 0.0 && text != "0.00" {
        format!("\u{2212}{text}")
    } else {
        text
    }
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
    fn signed_uses_a_minus_sign_and_no_negative_zero() {
        assert_eq!(super::signed(0.17), "0.17");
        assert_eq!(super::signed(-0.25), "\u{2212}0.25");
        assert_eq!(super::signed(-0.001), "0.00");
        assert_eq!(super::signed(0.0), "0.00");
    }

    #[test]
    fn mix_runs_from_one_colour_to_the_other() {
        use super::{Color32, mix};
        let (a, b) = (
            Color32::from_rgb(0, 100, 200),
            Color32::from_rgb(200, 100, 0),
        );
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
        assert_eq!(mix(a, b, 0.5), Color32::from_rgb(100, 100, 100));
    }

    #[test]
    fn the_field_palette_runs_from_navy_to_gold() {
        use super::{Color32, NAVY, field_palette};
        let p = field_palette(256);
        assert_eq!(p.len(), 256);
        assert_eq!(p[0], NAVY);
        assert_eq!(p[255], Color32::from_rgb(0xe9, 0xc4, 0x6a));
    }

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
