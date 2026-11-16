use crate::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

/// Windows at least this wide (in points) open the explanation by default. Narrower ones,
/// like a laptop's, start with it closed, so the grid isn't squeezed between two panels.
const WIDE_WINDOW: f32 = 1400.0;

/// A side panel showing a demo's documentation, rendered from Markdown.
///
/// Pass it the demo's `README.md` with `include_str!`, so the same text reads in the
/// repository and inside the app.
pub struct About {
    markdown: &'static str,
    cache: CommonMarkCache,
    /// Shown or hidden; decided from the window's width on the first frame, then by the
    /// reader's clicks.
    open: Option<bool>,
}

impl About {
    pub fn new(markdown: &'static str) -> Self {
        Self {
            markdown,
            cache: CommonMarkCache::default(),
            open: None,
        }
    }

    /// A button that shows or hides the panel.
    pub fn toggle_button(&mut self, ui: &mut egui::Ui) {
        let open = self.is_open(ui.ctx());
        let label = if open {
            "Hide the explanation"
        } else {
            "What is this?"
        };
        if ui.button(label).clicked() {
            self.open = Some(!open);
        }
    }

    /// Show the panel on the right of the window, if it is open. Call before the
    /// central panel, so the grid gets only the space left over.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if !self.is_open(ui.ctx()) {
            return;
        }
        egui::Panel::right("about")
            .resizable(true)
            .default_size(panel_width(ui.ctx().content_rect().width()))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    CommonMarkViewer::new().show(ui, &mut self.cache, self.markdown);
                });
            });
    }

    fn is_open(&mut self, ctx: &egui::Context) -> bool {
        *self
            .open
            .get_or_insert_with(|| opens_by_default(ctx.content_rect().width()))
    }
}

fn opens_by_default(window_width: f32) -> bool {
    window_width >= WIDE_WINDOW
}

/// About a third of the window, within reason, leaving the grid the most room.
fn panel_width(window_width: f32) -> f32 {
    (window_width * 0.32).clamp(300.0, 440.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On a laptop-sized window the explanation must start closed: with it open, the controls
    /// and the text squeezed the grid into a small box in the middle (seen on a ~1240-point
    /// window). A wide desktop window still opens it.
    #[test]
    fn opens_only_on_wide_windows() {
        assert!(!opens_by_default(1236.0));
        assert!(opens_by_default(1920.0));
    }

    /// The panel takes about a third of the window, but never so little the text is
    /// cramped, nor so much on a huge screen that it dwarfs the grid.
    #[test]
    fn panel_width_stays_in_range() {
        assert_eq!(panel_width(1000.0), 320.0);
        assert_eq!(panel_width(600.0), 300.0);
        assert_eq!(panel_width(4000.0), 440.0);
    }
}
