//! Drawing a [`Picture`] on screen, at any position and scale of the plane.
//!
//! Not `ca_core::GridView`: that draws whole cells, letterboxed to the grid's shape and
//! rounded to whole pixels, and keeps its screen mapping to itself. Zooming about the
//! pointer needs an exact mapping both ways between the screen and the plane, and the
//! picture of the old view has to be drawn moved and scaled under the new one while that
//! is computed.

use crate::colour::Colours;
use crate::sim::{Picture, Viewport};
use ca_core::egui::{self, Color32, ColorImage, Painter, Rect, TextureHandle, TextureOptions};

const OPTIONS: TextureOptions = TextureOptions::LINEAR;

pub struct PlaneView {
    name: &'static str,
    /// The latest picture, and the view it shows.
    texture: Option<(TextureHandle, Viewport)>,
    /// An earlier, finished picture, drawn underneath where the latest is still blank: after
    /// a zoom, the old view, scaled, fills in until the new one is computed.
    backdrop: Option<(TextureHandle, Viewport)>,
}

impl PlaneView {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            texture: None,
            backdrop: None,
        }
    }

    /// Upload the picture as it stands.
    pub fn set_picture(&mut self, ctx: &egui::Context, picture: &Picture, colours: &Colours) {
        let image = image(picture, colours);
        let view = *picture.view();
        match &mut self.texture {
            Some((texture, shown)) => {
                texture.set(image, OPTIONS);
                *shown = view;
            }
            None => self.texture = Some((ctx.load_texture(self.name, image, OPTIONS), view)),
        }
    }

    /// Keep the latest picture as the backdrop for the next one.
    pub fn keep_as_backdrop(&mut self) {
        if let Some(latest) = self.texture.take() {
            self.backdrop = Some(latest);
        }
    }

    pub fn has_backdrop(&self) -> bool {
        self.backdrop.is_some()
    }

    pub fn drop_backdrop(&mut self) {
        self.backdrop = None;
    }

    /// Draw into `rect`, which shows `current` at `scale` pixels per point.
    pub fn paint(&self, painter: &Painter, rect: Rect, scale: f32, current: &Viewport) {
        let painter = painter.with_clip_rect(rect);
        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        if current.is_fisheye() {
            // A fisheye bends straight lines, so the corners can't place the picture; it is
            // always a picture of the current view (every move recomputes it), so it goes
            // where the view is.
            if let Some((texture, view)) = &self.texture {
                let size = egui::vec2(view.width as f32, view.height as f32) / scale;
                let shown = Rect::from_min_size(rect.min, size);
                painter.image(texture.id(), shown, uv, Color32::WHITE);
            }
            return;
        }
        for (texture, view) in self.backdrop.iter().chain(self.texture.iter()) {
            let corner = |x: f64, y: f64| {
                let [px, py] = current.pixel_of(view.at(x, y));
                rect.min + egui::vec2(px as f32, py as f32) / scale
            };
            let shown = Rect::from_min_max(
                corner(0.0, 0.0),
                corner(view.width as f64, view.height as f64),
            );
            painter.image(texture.id(), shown, uv, Color32::WHITE);
        }
    }
}

/// The picture's colours. A pixel not computed yet borrows the colour of the coarse pixel
/// above and left of it, which is computed first, so a new view appears at once in blocks
/// and then sharpens; if that isn't done either, it is transparent and the backdrop shows.
fn image(picture: &Picture, colours: &Colours) -> ColorImage {
    let view = picture.view();
    let (w, h) = (view.width, view.height);
    let mut pixels = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let colour = colours
                .of(picture.pixel(x, y))
                .or_else(|| colours.of(picture.pixel(x & !1, y & !1)))
                .or_else(|| colours.of(picture.pixel(x & !3, y & !3)))
                .unwrap_or(Color32::TRANSPARENT);
            pixels.push(colour);
        }
    }
    ColorImage::new([w, h], pixels)
}
