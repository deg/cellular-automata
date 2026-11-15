use crate::Grid;
use crate::egui::{
    self, Color32, ColorImage, Rect, Sense, TextureHandle, TextureOptions, pos2, vec2,
};

/// Draws a [`Grid`] as a single texture, scaled to fit the space available.
///
/// Upload a new picture with [`GridView::set_image`] only when the grid has changed;
/// [`GridView::show`] is cheap and can run every frame.
pub struct GridView {
    name: String,
    texture: Option<TextureHandle>,
}

pub struct GridResponse {
    pub response: egui::Response,
    /// The cell under the pointer, if any.
    pub hovered: Option<(usize, usize)>,
    /// The cell under the pointer on every frame the button is held down, for painting
    /// by dragging. A single click usually spans several frames, so it reports several times.
    pub pressed: Option<(usize, usize)>,
    /// The cell clicked this frame, reported once per click (on release, and not after a
    /// drag). Use this, not `pressed`, when a click means "do one thing here".
    pub clicked: Option<(usize, usize)>,
}

impl GridView {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            texture: None,
        }
    }

    pub fn set_image<T: Copy>(
        &mut self,
        ctx: &egui::Context,
        grid: &Grid<T>,
        color: impl Fn(T) -> Color32,
    ) {
        let pixels = grid.cells().iter().map(|&c| color(c)).collect();
        let image = ColorImage::new([grid.width(), grid.height()], pixels);
        match &mut self.texture {
            Some(texture) => texture.set(image, TextureOptions::NEAREST),
            None => {
                self.texture = Some(ctx.load_texture(&self.name, image, TextureOptions::NEAREST))
            }
        }
    }

    /// Fill the remaining space, drawing the grid centred with square cells.
    pub fn show(&self, ui: &mut egui::Ui) -> GridResponse {
        let (area, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let Some(texture) = &self.texture else {
            return GridResponse {
                response,
                hovered: None,
                pressed: None,
                clicked: None,
            };
        };

        let [w, h] = texture.size().map(|s| s as f32);
        let mut scale = (area.width() / w).min(area.height() / h);
        if scale >= 1.0 {
            scale = scale.floor(); // whole pixels per cell keep the grid crisp
        }
        let image_rect = Rect::from_center_size(area.center(), vec2(w * scale, h * scale));
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        ui.painter()
            .image(texture.id(), image_rect, uv, Color32::WHITE);

        let cell_at = |pos: egui::Pos2| {
            let rel = (pos - image_rect.min) / scale;
            (rel.x >= 0.0 && rel.y >= 0.0 && rel.x < w && rel.y < h)
                .then_some((rel.x as usize, rel.y as usize))
        };
        let hovered = response.hover_pos().and_then(cell_at);
        let pressed = if response.is_pointer_button_down_on() {
            response.interact_pointer_pos().and_then(cell_at)
        } else {
            None
        };
        // Touch input has no hover position, so fall back to it only after the press position.
        let clicked = if response.clicked() {
            response
                .interact_pointer_pos()
                .or(response.hover_pos())
                .and_then(cell_at)
        } else {
            None
        };
        GridResponse {
            response,
            hovered,
            pressed,
            clicked,
        }
    }
}
