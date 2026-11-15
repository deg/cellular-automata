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
    /// The first texel column shown: the picture may be cropped (see [`Layout::crop`]).
    crop_left: usize,
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

    /// The texel columns worth showing: all of them for a square layout. For hex, the
    /// columns of the largest centred hexagon of cells (|dq|, |dr|, |dq + dr| at most half the
    /// side), which drops the parallelogram's empty corners, widened as needed to include
    /// every cell for which `occupied` is true, so cropping never hides a grain.
    pub fn crop<T: Copy>(
        self,
        grid: &Grid<T>,
        occupied: impl Fn(T) -> bool,
    ) -> std::ops::Range<usize> {
        let (w, h) = (grid.width(), grid.height());
        let full = 0..self.texture_size(w, h)[0];
        if self == Self::Square || w != h || w == 0 {
            return full;
        }
        // On an n x n grid with centre c = n / 2, hexagon cells have 2dq + dr from -2c to 2c,
        // so texel columns c to 5c + 1.
        let c = w / 2;
        let (mut lo, mut hi) = (c, 5 * c + 2);
        for y in 0..h {
            for x in 0..w {
                if occupied(grid.get(x, y)) {
                    let (tx, _) = self.cell_texel(x, y);
                    lo = lo.min(tx);
                    hi = hi.max(tx + self.cell_width());
                }
            }
        }
        lo.max(full.start)..hi.min(full.end)
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
    crop_left: usize,
}

impl GridResponse {
    /// The screen rectangle of cell `(x, y)`, for drawing markers over the grid.
    pub fn cell_rect(&self, x: usize, y: usize) -> Rect {
        let (tx, ty) = self.layout.cell_texel(x, y);
        let tx = tx as f32 - self.crop_left as f32; // negative if cropped off, which is fine
        let min = self.image_rect.min + vec2(tx * self.texel.x, ty as f32 * self.texel.y);
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
            crop_left: 0,
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
        self.set_image_cropped(ctx, grid, color, |_| true);
    }

    /// Like [`GridView::set_image`], but in the hex layout trims the parallelogram's empty
    /// corners down to the central hexagon, widening only as far as needed to keep every cell
    /// for which `occupied` is true in view. Has no effect on a square layout.
    pub fn set_image_cropped<T: Copy>(
        &mut self,
        ctx: &egui::Context,
        grid: &Grid<T>,
        color: impl Fn(T) -> Color32,
        occupied: impl Fn(T) -> bool,
    ) {
        self.grid_size = (grid.width(), grid.height());
        let full = self.layout.image(grid, color);
        let columns = self.layout.crop(grid, occupied);
        self.crop_left = columns.start;
        let image = if columns.len() == full.size[0] {
            full
        } else {
            let (fw, h) = (full.size[0], full.size[1]);
            let pixels = (0..h)
                .flat_map(|y| {
                    full.pixels[y * fw + columns.start..y * fw + columns.end]
                        .iter()
                        .copied()
                })
                .collect();
            ColorImage::new([columns.len(), h], pixels)
        };
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
                crop_left: 0,
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
        let (layout, crop_left) = (self.layout, self.crop_left);
        let cell_at = |pos: egui::Pos2| {
            let rel = pos - image_rect.min;
            let (tx, ty) = (rel.x / texel.x, rel.y / texel.y);
            if tx < 0.0 || ty < 0.0 || tx >= w || ty >= h {
                return None;
            }
            layout.cell_at_texel(tx as usize + crop_left, ty as usize, gw, gh)
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
            crop_left,
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

    /// Cropping must drop the hex parallelogram's empty corners (the centred hexagon of an
    /// n x n grid is 2n texels wide, not 3n - 1), must widen to keep any occupied cell in
    /// view, and must leave a square layout alone.
    #[test]
    fn crop_trims_hex_corners_but_never_an_occupied_cell() {
        let mut g = Grid::new(11, 11, false);
        assert_eq!(Layout::Hex.crop(&g, |c| c), 5..27, "2 x 11 texels, centred");
        assert_eq!(Layout::Hex.texture_size(11, 11)[0], 32);
        g.set(0, 0, true); // the top-left corner cell, drawn at texels 0 and 1
        assert_eq!(Layout::Hex.crop(&g, |c| c), 0..27);
        g.set(10, 10, true); // the bottom-right corner, texels 30 and 31
        assert_eq!(Layout::Hex.crop(&g, |c| c), 0..32);
        assert_eq!(Layout::Square.crop(&g, |c| c), 0..11);
    }

    /// Every cell of the centred hexagon must lie inside the cropped columns, or the crop
    /// would cut into a pile grown from the centre.
    #[test]
    fn crop_keeps_the_whole_central_hexagon() {
        let n = 21;
        let g = Grid::new(n, n, false);
        let cols = Layout::Hex.crop(&g, |c| c);
        let c = (n / 2) as isize;
        for y in 0..n {
            for x in 0..n {
                let (dq, dr) = (x as isize - c, y as isize - c);
                if dq.abs() <= c && dr.abs() <= c && (dq + dr).abs() <= c {
                    let (tx, _) = Layout::Hex.cell_texel(x, y);
                    assert!(
                        cols.contains(&tx) && cols.contains(&(tx + 1)),
                        "cell ({x}, {y})"
                    );
                }
            }
        }
    }
}
