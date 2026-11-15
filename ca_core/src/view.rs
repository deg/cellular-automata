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
    layout: Layout,
    /// The grid's size in cells, as of the last `set_image`.
    grid_size: (usize, usize),
}

/// How a grid's cells are laid out in the texture, and so on screen.
///
/// Both directions of the mapping (cell to texels for drawing and markers, texel to cell for
/// the pointer) live here as plain functions, so they can be tested against each other.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Layout {
    /// One square texel per cell.
    #[default]
    Square,
    /// Hexagonal cells in axial coordinates: `x` runs along a row, and `y` down and to the
    /// right, so each cell's six neighbours are at a fixed set of offsets. Drawn as bricks:
    /// each cell is two texels wide, each row starts one texel (half a cell) right of the row
    /// above, and texels are drawn taller than wide so rows sit at true hex spacing.
    Hex,
}

impl Layout {
    /// Texture size, `[width, height]` in texels, for a `w` x `h` grid.
    pub fn texture_size(self, w: usize, h: usize) -> [usize; 2] {
        match self {
            Self::Square => [w, h],
            // Row y starts at texel y, and its last cell ends at 2(w-1) + y + 1.
            Self::Hex => [2 * w + h.saturating_sub(1), h],
        }
    }

    /// Texels per cell, across.
    pub fn cell_width(self) -> usize {
        match self {
            Self::Square => 1,
            Self::Hex => 2,
        }
    }

    /// A texel's height on screen relative to its width. Hex rows are sqrt(3)/2 of a cell
    /// width apart, and a cell is 2 texels wide, so a row is sqrt(3) texel widths tall.
    pub fn texel_aspect(self) -> f32 {
        match self {
            Self::Square => 1.0,
            Self::Hex => 3f32.sqrt(),
        }
    }

    /// The texel at the left end of cell `(x, y)` (it spans [`Layout::cell_width`] texels).
    pub fn cell_texel(self, x: usize, y: usize) -> (usize, usize) {
        match self {
            Self::Square => (x, y),
            Self::Hex => (2 * x + y, y),
        }
    }

    /// The picture of `grid` in this layout, one colour per cell from `color`. Texels no cell
    /// covers stay transparent, so the panel shows through.
    pub fn image<T: Copy>(self, grid: &Grid<T>, color: impl Fn(T) -> Color32) -> ColorImage {
        let (w, h) = (grid.width(), grid.height());
        if self == Self::Square {
            let pixels = grid.cells().iter().map(|&c| color(c)).collect();
            return ColorImage::new([w, h], pixels);
        }
        let size = self.texture_size(w, h);
        let mut pixels = vec![Color32::TRANSPARENT; size[0] * size[1]];
        for y in 0..h {
            for x in 0..w {
                let (tx, ty) = self.cell_texel(x, y);
                let c = color(grid.get(x, y));
                for dx in 0..self.cell_width() {
                    pixels[ty * size[0] + tx + dx] = c;
                }
            }
        }
        ColorImage::new(size, pixels)
    }

    /// The cell drawn at texel `(tx, ty)` of a `w` x `h` grid, or `None` for a texel no cell
    /// covers (the empty corners of the hex layout).
    pub fn cell_at_texel(self, tx: usize, ty: usize, w: usize, h: usize) -> Option<(usize, usize)> {
        if ty >= h {
            return None;
        }
        let x = match self {
            Self::Square => tx,
            Self::Hex => tx.checked_sub(ty)? / 2,
        };
        (x < w).then_some((x, ty))
    }
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
    /// Where the grid was drawn, and a texel's width and height on screen, for drawing over it.
    image_rect: Rect,
    texel: egui::Vec2,
    layout: Layout,
}

impl GridResponse {
    /// The screen rectangle of cell `(x, y)`, for drawing markers over the grid.
    pub fn cell_rect(&self, x: usize, y: usize) -> Rect {
        let (tx, ty) = self.layout.cell_texel(x, y);
        let min = self.image_rect.min + vec2(tx as f32 * self.texel.x, ty as f32 * self.texel.y);
        let size = vec2(self.layout.cell_width() as f32 * self.texel.x, self.texel.y);
        Rect::from_min_size(min, size)
    }

    /// The width of one cell on screen, in points.
    pub fn cell_size(&self) -> f32 {
        self.layout.cell_width() as f32 * self.texel.x
    }
}

impl GridView {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            texture: None,
            layout: Layout::Square,
            grid_size: (0, 0),
        }
    }

    /// Choose how cells are laid out; takes effect at the next [`GridView::set_image`].
    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    pub fn set_image<T: Copy>(
        &mut self,
        ctx: &egui::Context,
        grid: &Grid<T>,
        color: impl Fn(T) -> Color32,
    ) {
        self.grid_size = (grid.width(), grid.height());
        let image = self.layout.image(grid, color);
        match &mut self.texture {
            Some(texture) => texture.set(image, TextureOptions::NEAREST),
            None => {
                self.texture = Some(ctx.load_texture(&self.name, image, TextureOptions::NEAREST))
            }
        }
    }

    /// Fill the remaining space, drawing the grid centred, in its layout's proportions.
    pub fn show(&self, ui: &mut egui::Ui) -> GridResponse {
        let (area, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let Some(texture) = &self.texture else {
            return GridResponse {
                response,
                hovered: None,
                pressed: None,
                clicked: None,
                image_rect: Rect::NOTHING,
                texel: egui::Vec2::ZERO,
                layout: self.layout,
            };
        };

        let [w, h] = texture.size().map(|s| s as f32);
        let aspect = self.layout.texel_aspect();
        let mut scale = (area.width() / w).min(area.height() / (h * aspect));
        if scale >= 1.0 && self.layout == Layout::Square {
            scale = scale.floor(); // whole pixels per cell keep a square grid crisp
        }
        let texel = vec2(scale, scale * aspect);
        let image_rect = Rect::from_center_size(area.center(), vec2(w * texel.x, h * texel.y));
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        ui.painter()
            .image(texture.id(), image_rect, uv, Color32::WHITE);

        let (gw, gh) = self.grid_size;
        let layout = self.layout;
        let cell_at = |pos: egui::Pos2| {
            let rel = pos - image_rect.min;
            let (tx, ty) = (rel.x / texel.x, rel.y / texel.y);
            if tx < 0.0 || ty < 0.0 || tx >= w || ty >= h {
                return None;
            }
            layout.cell_at_texel(tx as usize, ty as usize, gw, gh)
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
            image_rect,
            texel,
            layout,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drawing, markers and the pointer must agree: every texel of every cell maps back to
    /// that cell, in both layouts, and every texel of the texture is either one cell's or
    /// (hex only) covered by none. A slip here puts clicks and drip rings on the wrong cell,
    /// which nobody can spot by eye on a big grid.
    #[test]
    fn cells_and_texels_round_trip() {
        let (w, h) = (7, 5);
        for layout in [Layout::Square, Layout::Hex] {
            let [tw, th] = layout.texture_size(w, h);
            let mut owner = vec![None; tw * th];
            for y in 0..h {
                for x in 0..w {
                    let (tx, ty) = layout.cell_texel(x, y);
                    for dx in 0..layout.cell_width() {
                        assert!(
                            tx + dx < tw && ty < th,
                            "{layout:?} cell ({x}, {y}) off the texture"
                        );
                        assert!(
                            owner[ty * tw + tx + dx].is_none(),
                            "{layout:?} overlap at ({x}, {y})"
                        );
                        owner[ty * tw + tx + dx] = Some((x, y));
                    }
                }
            }
            for ty in 0..th {
                for tx in 0..tw {
                    assert_eq!(
                        layout.cell_at_texel(tx, ty, w, h),
                        owner[ty * tw + tx],
                        "{layout:?} texel ({tx}, {ty})"
                    );
                }
            }
            assert_eq!(layout.cell_at_texel(tw, 0, w, h), None);
            assert_eq!(layout.cell_at_texel(0, th, w, h), None);
        }
    }

    /// In the hex layout each row starts half a cell (one texel) right of the row above, so
    /// a cell's six axial neighbours touch it: the two beside it, and two in each of the rows
    /// above and below, overlapping it by one texel each.
    #[test]
    fn hex_neighbours_touch() {
        let l = Layout::Hex;
        let (cx, cy) = l.cell_texel(3, 2);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, -1), (-1, 1)] {
            let (nx, ny) = l.cell_texel((3 + dx) as usize, (2 + dy) as usize);
            let overlap = (cx.max(nx)..(cx + 2).min(nx + 2)).len();
            match dy {
                0 => assert_eq!(nx.abs_diff(cx), 2, "side neighbour ({dx}, {dy})"),
                _ => assert_eq!((ny.abs_diff(cy), overlap), (1, 1), "neighbour ({dx}, {dy})"),
            }
        }
    }
}
