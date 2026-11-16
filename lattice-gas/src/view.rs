//! Drawing the lattice: nodes in offset rows, as bricks.
//!
//! `ca_core::GridView`'s hex layout is for axial coordinates, where each row starts half a
//! cell right of the row above, so a world is a rhombus. This lattice's rows alternate
//! instead (odd rows half a node right), keeping the world a rectangle. Each node is two
//! texels wide, odd rows start one texel in, and texels are drawn `sqrt 3` times taller
//! than wide, so rows sit at the true spacing of a triangular lattice.

use crate::sim::ROW_HEIGHT;
use ca_core::egui::{
    self, Color32, ColorImage, Pos2, Rect, Sense, TextureHandle, TextureOptions, pos2, vec2,
};

/// The texture's size in texels, `[width, height]`, for a `w` x `h` lattice.
pub fn texture_size(w: usize, h: usize) -> [usize; 2] {
    [2 * w + 1, h]
}

/// The first of node `(x, y)`'s two texels.
pub fn node_texel(x: usize, y: usize) -> (usize, usize) {
    (2 * x + y % 2, y)
}

/// The node drawn at texel `(tx, ty)`, or `None` for the half-node gap at the end of each
/// row.
pub fn node_at_texel(tx: usize, ty: usize, w: usize, h: usize) -> Option<(usize, usize)> {
    if ty >= h {
        return None;
    }
    let x = tx.checked_sub(ty % 2)? / 2;
    (x < w).then_some((x, ty))
}

/// The picture of a `w` x `h` lattice, one colour per node from `color(x, y)`.
pub fn image(w: usize, h: usize, color: impl Fn(usize, usize) -> Color32) -> ColorImage {
    let size = texture_size(w, h);
    let mut pixels = vec![Color32::TRANSPARENT; size[0] * size[1]];
    for y in 0..h {
        let row = &mut pixels[y * size[0]..(y + 1) * size[0]];
        for x in 0..w {
            let c = color(x, y);
            let (tx, _) = node_texel(x, y);
            row[tx] = c;
            row[tx + 1] = c;
        }
    }
    ColorImage::new(size, pixels)
}

pub struct LatticeView {
    texture: Option<TextureHandle>,
    size: (usize, usize),
}

/// Where the lattice was drawn, and the pointer's node, from [`LatticeView::show`].
pub struct Shown {
    /// The node under the pointer on every frame the button is held, for painting.
    pub pressed: Option<(usize, usize)>,
    pub hovered: Option<(usize, usize)>,
}

impl LatticeView {
    pub fn new() -> Self {
        Self {
            texture: None,
            size: (0, 0),
        }
    }

    pub fn set_image(&mut self, ctx: &egui::Context, w: usize, h: usize, image: ColorImage) {
        self.size = (w, h);
        match &mut self.texture {
            Some(t) => t.set(image, TextureOptions::NEAREST),
            None => {
                self.texture = Some(ctx.load_texture("lattice", image, TextureOptions::NEAREST))
            }
        }
    }

    /// Fill the space left, drawing the lattice centred at the largest size that fits.
    pub fn show(&self, ui: &mut egui::Ui) -> Shown {
        let (area, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let (w, h) = self.size;
        let Some(texture) = &self.texture else {
            return Shown {
                pressed: None,
                hovered: None,
            };
        };
        // In node spacings: (w + 1/2) across, h rows of ROW_HEIGHT down.
        let (lw, lh) = (w as f32 + 0.5, h as f32 * ROW_HEIGHT as f32);
        let scale = (area.width() / lw).min(area.height() / lh);
        let rect = Rect::from_center_size(area.center(), vec2(lw * scale, lh * scale));
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        ui.painter().image(texture.id(), rect, uv, Color32::WHITE);

        let node_at = |pos: Pos2| {
            let rel = pos - rect.min;
            if rel.x < 0.0 || rel.y < 0.0 || rel.x >= rect.width() || rel.y >= rect.height() {
                return None;
            }
            let tx = (rel.x / (scale * 0.5)) as usize;
            let ty = (rel.y / (scale * ROW_HEIGHT as f32)) as usize;
            node_at_texel(tx, ty, w, h)
        };
        let pressed = if response.is_pointer_button_down_on() {
            response.interact_pointer_pos().and_then(node_at)
        } else {
            None
        };
        Shown {
            pressed,
            hovered: response.hover_pos().and_then(node_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drawing and the pointer must agree: each node's two texels map back to it, and every
    /// other texel (the half-node gap ending each row) to no node.
    #[test]
    fn nodes_and_texels_round_trip() {
        let (w, h) = (5, 4);
        let [tw, th] = texture_size(w, h);
        let mut owner = vec![None; tw * th];
        for y in 0..h {
            for x in 0..w {
                let (tx, ty) = node_texel(x, y);
                for dx in 0..2 {
                    assert!(owner[ty * tw + tx + dx].is_none());
                    owner[ty * tw + tx + dx] = Some((x, y));
                }
            }
        }
        for ty in 0..th {
            for tx in 0..tw {
                assert_eq!(
                    node_at_texel(tx, ty, w, h),
                    owner[ty * tw + tx],
                    "({tx}, {ty})"
                );
            }
        }
        assert_eq!(
            owner.iter().filter(|o| o.is_none()).count(),
            h,
            "one gap per row"
        );
    }
}
