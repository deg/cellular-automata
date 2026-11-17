use crate::egui;
use crate::line;

/// The controls panel's width, in points. Fixed: a non-resizable egui panel otherwise grows
/// to fit its widest row, and on a laptop the controls took a third of the window. Rows
/// that might be wider must wrap (`horizontal_wrapped`), not widen it.
const CONTROLS_WIDTH: f32 = 290.0;

/// The left-hand controls panel every demo uses: a fixed width, scrolling when its contents
/// are taller than the window (a short browser window would otherwise cut off the bottom).
/// Call it before the central panel, so the grid gets only the space left over.
pub fn controls_panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Panel::left("controls")
        .resizable(false)
        .exact_size(CONTROLS_WIDTH)
        .show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, add_contents);
        });
}

/// Turns a held pointer into the cells to paint each frame.
///
/// The pointer can cross many cells between two frames; painting only the cell under it
/// would leave a dotted trail. This remembers the last cell of the stroke and returns the
/// whole line from there, and forgets it when the button is released, so the next stroke
/// doesn't join on to the last.
#[derive(Default)]
pub struct PaintStroke {
    last: Option<(usize, usize)>,
}

impl PaintStroke {
    /// The cells to paint this frame, given [`crate::GridResponse::pressed`]. Empty when the
    /// pointer is not pressed on the grid.
    pub fn cells(&mut self, pressed: Option<(usize, usize)>) -> Vec<(usize, usize)> {
        match pressed {
            Some(cell) => {
                let cells = line(self.last.unwrap_or(cell), cell);
                self.last = Some(cell);
                cells
            }
            None => {
                self.last = None;
                Vec::new()
            }
        }
    }
}

/// What the note under a demo's presets says when the settings match none of them.
pub const OWN_SETTINGS: &str = "Your own settings: none of the presets.";

/// The note under a demo's presets: the chosen preset's (`chosen`), or `other` when the
/// settings match none of them (usually [`OWN_SETTINGS`]). It always takes the height of the
/// tallest of `notes` and `other` at the panel's width, so choosing a preset or changing a
/// setting never moves the controls below it.
pub fn preset_note<'a>(
    ui: &mut egui::Ui,
    chosen: Option<&str>,
    notes: impl IntoIterator<Item = &'a str>,
    other: &'a str,
) {
    let font = egui::TextStyle::Small.resolve(ui.style());
    let width = ui.available_width();
    let tallest = notes
        .into_iter()
        .chain([other])
        .map(|text| {
            ui.painter()
                .layout(text.to_owned(), font.clone(), egui::Color32::WHITE, width)
                .size()
                .y
        })
        .fold(0.0, f32::max);
    ui.allocate_ui(egui::vec2(width, tallest), |ui| {
        ui.set_min_height(tallest);
        ui.small(chosen.unwrap_or(other));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A press paints just the cell under it; a fast drag paints the whole line between the
    /// frames' cells, so strokes are solid.
    #[test]
    fn a_drag_paints_a_solid_line() {
        let mut s = PaintStroke::default();
        assert_eq!(s.cells(Some((2, 2))), vec![(2, 2)]);
        assert_eq!(s.cells(Some((5, 2))), vec![(2, 2), (3, 2), (4, 2), (5, 2)]);
    }

    /// Releasing ends the stroke: the next press must not draw a line back to where the last
    /// stroke ended, or every new stroke would be joined to the old one.
    #[test]
    fn a_release_ends_the_stroke() {
        let mut s = PaintStroke::default();
        s.cells(Some((0, 0)));
        assert!(s.cells(None).is_empty());
        assert_eq!(s.cells(Some((9, 9))), vec![(9, 9)]);
    }
}
