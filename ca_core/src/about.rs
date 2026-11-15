use crate::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

/// A side panel showing a demo's documentation, rendered from Markdown.
///
/// Pass it the demo's `README.md` with `include_str!`, so the same text reads in the
/// repository and inside the app.
pub struct About {
    markdown: &'static str,
    cache: CommonMarkCache,
    pub open: bool,
}

impl About {
    pub fn new(markdown: &'static str) -> Self {
        Self { markdown, cache: CommonMarkCache::default(), open: true }
    }

    /// A button that shows or hides the panel.
    pub fn toggle_button(&mut self, ui: &mut egui::Ui) {
        let label = if self.open { "Hide the explanation" } else { "What is this?" };
        if ui.button(label).clicked() {
            self.open = !self.open;
        }
    }

    /// Show the panel on the right of the window, if it is open. Call before the
    /// central panel, so the grid gets only the space left over.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if !self.open {
            return;
        }
        egui::Panel::right("about")
            .resizable(true)
            .default_size(420.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    CommonMarkViewer::new().show(ui, &mut self.cache, self.markdown);
                });
            });
    }
}
