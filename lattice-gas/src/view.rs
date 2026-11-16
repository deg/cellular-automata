//! Drawing the lattice: nodes in offset rows, as bricks.
//!
//! `ca_core::GridView`'s hex layout is for axial coordinates, where each row starts half a
//! cell right of the row above, so a world is a rhombus. This lattice's rows alternate
//! instead (odd rows half a node right), keeping the world a rectangle. Each node is two
//! texels wide, odd rows start one texel in, and texels are drawn `sqrt 3` times taller
//! than wide, so rows sit at the true spacing of a triangular lattice.
//!
//! The square lattice is drawn the same way, without the shift, and with texels twice as
//! tall as wide, so each node is square.

use crate::sim::Kind;
use ca_core::egui::{
    self, Color32, ColorImage, Pos2, Rect, Sense, TextureHandle, TextureOptions, pos2, vec2,
};

/// Texels a row is shifted right: 1 for odd rows of the hex lattice, else 0.
fn shift(kind: Kind, y: usize) -> usize {
    usize::from(kind.shifted() && y % 2 == 1)
}

/// The texture's size in texels, `[width, height]`, for a `w` x `h` lattice.
pub fn texture_size(kind: Kind, w: usize, h: usize) -> [usize; 2] {
    [2 * w + usize::from(kind.shifted()), h]
}

/// The first of node `(x, y)`'s two texels.
pub fn node_texel(kind: Kind, x: usize, y: usize) -> (usize, usize) {
    (2 * x + shift(kind, y), y)
}

/// The node drawn at texel `(tx, ty)`, or `None` for the half-node gap at the end of each
/// hex row.
pub fn node_at_texel(
    kind: Kind,
    tx: usize,
    ty: usize,
    w: usize,
    h: usize,
) -> Option<(usize, usize)> {
    if ty >= h {
        return None;
    }
    let x = tx.checked_sub(shift(kind, ty))? / 2;
    (x < w).then_some((x, ty))
}

/// The picture of a `w` x `h` lattice, one colour per node from `color(x, y)`.
pub fn image(
    kind: Kind,
    w: usize,
    h: usize,
    color: impl Fn(usize, usize) -> Color32,
) -> ColorImage {
    let size = texture_size(kind, w, h);
    let mut pixels = vec![Color32::TRANSPARENT; size[0] * size[1]];
    for y in 0..h {
        let row = &mut pixels[y * size[0]..(y + 1) * size[0]];
        for x in 0..w {
            let c = color(x, y);
            let (tx, _) = node_texel(kind, x, y);
            row[tx] = c;
            row[tx + 1] = c;
        }
    }
    ColorImage::new(size, pixels)
}

pub struct LatticeView {
    texture: Option<TextureHandle>,
    kind: Kind,
    size: (usize, usize),
}

/// Where the lattice was drawn, and the pointer's node, from [`LatticeView::show`].
pub struct Shown {
    /// The node under the pointer on every frame the button is held, for painting.
    pub pressed: Option<(usize, usize)>,
    pub hovered: Option<(usize, usize)>,
    rect: Rect,
    /// Points per node spacing.
    scale: f32,
    kind: Kind,
}

impl Shown {
    /// The screen point of the lattice position `(px, py)`, in node spacings, y down (as
    /// `Lattice::position` gives it). A node's bricks span one node spacing to the right of
    /// its position and one row down, so its centre is half of each further on.
    pub fn to_screen(&self, px: f64, py: f64) -> Pos2 {
        let (cx, cy) = (px + 0.5, py + 0.5 * self.kind.row_height());
        self.rect.min + vec2(cx as f32 * self.scale, cy as f32 * self.scale)
    }

    /// Points per node spacing.
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Where the lattice was drawn.
    pub fn rect(&self) -> Rect {
        self.rect
    }
}

impl LatticeView {
    pub fn new() -> Self {
        Self {
            texture: None,
            kind: Kind::Hex,
            size: (0, 0),
        }
    }

    pub fn set_image(
        &mut self,
        ctx: &egui::Context,
        kind: Kind,
        (w, h): (usize, usize),
        image: ColorImage,
    ) {
        (self.kind, self.size) = (kind, (w, h));
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
        let kind = self.kind;
        let Some(texture) = &self.texture else {
            return Shown {
                pressed: None,
                hovered: None,
                rect: Rect::NOTHING,
                scale: 0.0,
                kind,
            };
        };
        // In node spacings: the texture's width in half nodes, h rows down.
        let row_height = kind.row_height() as f32;
        let lw = texture_size(kind, w, h)[0] as f32 / 2.0;
        let lh = h as f32 * row_height;
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
            let ty = (rel.y / (scale * row_height)) as usize;
            node_at_texel(kind, tx, ty, w, h)
        };
        let pressed = if response.is_pointer_button_down_on() {
            response.interact_pointer_pos().and_then(node_at)
        } else {
            None
        };
        Shown {
            pressed,
            hovered: response.hover_pos().and_then(node_at),
            rect,
            scale,
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drawing and the pointer must agree: each node's two texels map back to it, and every
    /// other texel (on the hex lattice, the half-node gap ending each row) to no node.
    #[test]
    fn nodes_and_texels_round_trip() {
        let (w, h) = (5, 4);
        for (kind, gaps) in [(Kind::Hex, h), (Kind::Square, 0)] {
            let [tw, th] = texture_size(kind, w, h);
            let mut owner = vec![None; tw * th];
            for y in 0..h {
                for x in 0..w {
                    let (tx, ty) = node_texel(kind, x, y);
                    for dx in 0..2 {
                        assert!(owner[ty * tw + tx + dx].is_none());
                        owner[ty * tw + tx + dx] = Some((x, y));
                    }
                }
            }
            for ty in 0..th {
                for tx in 0..tw {
                    let at = node_at_texel(kind, tx, ty, w, h);
                    assert_eq!(at, owner[ty * tw + tx], "{kind:?} ({tx}, {ty})");
                }
            }
            let empty = owner.iter().filter(|o| o.is_none()).count();
            assert_eq!(
                empty, gaps,
                "{kind:?}: one gap per hex row, none on the square"
            );
        }
    }
}
