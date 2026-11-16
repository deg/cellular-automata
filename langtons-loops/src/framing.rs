//! Which part of the world to show: the colony, with room to grow.
//!
//! A loop is 15 cells across, a few pixels on a big world. So the view starts close in and
//! steps back as the colony grows, in jumps rather than every generation, so the picture
//! doesn't keep shifting under the pointer.

use crate::sim::Bounds;

/// A rectangle of the world, `w` x `h` cells from `(x, y)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Frame {
    pub fn whole(world: (usize, usize)) -> Self {
        Self {
            x: 0,
            y: 0,
            w: world.0,
            h: world.1,
        }
    }

    /// The world cell shown at cell `(x, y)` of the frame.
    pub fn to_world(self, (x, y): (usize, usize)) -> (usize, usize) {
        (self.x + x, self.y + y)
    }

    fn contains(&self, b: &Bounds) -> bool {
        b.x0 >= self.x && b.y0 >= self.y && b.x1 <= self.x + self.w && b.y1 <= self.y + self.h
    }
}

/// Free cells kept around the colony.
const MARGIN: usize = 4;

/// How much bigger than the colony a new frame is, so it lasts a while.
const GROWTH: f64 = 1.5;

/// The frame to show next: `current`, if the colony (`bounds`, plus a margin) still fits in
/// it; otherwise a bigger one around the colony, at least `min_width` cells wide, in the
/// world's proportions, and inside the world.
pub fn follow(
    current: Option<Frame>,
    bounds: Option<Bounds>,
    world: (usize, usize),
    min_width: usize,
) -> Frame {
    let Some(b) = bounds else {
        return current.unwrap_or(Frame::whole(world));
    };
    let wanted = Bounds {
        x0: b.x0.saturating_sub(MARGIN),
        y0: b.y0.saturating_sub(MARGIN),
        x1: (b.x1 + MARGIN).min(world.0),
        y1: (b.y1 + MARGIN).min(world.1),
    };
    if let Some(f) = current
        && f.contains(&wanted)
    {
        return f;
    }
    let aspect = world.0 as f64 / world.1 as f64;
    let w = (wanted.width() as f64 * GROWTH)
        .max(wanted.height() as f64 * GROWTH * aspect)
        .max(min_width as f64);
    let w = (w.ceil() as usize).min(world.0);
    let h = ((w as f64 / aspect).round() as usize).clamp(1, world.1);
    // Centre on the colony, then slide back inside the world.
    let cx = (wanted.x0 + wanted.x1) / 2;
    let cy = (wanted.y0 + wanted.y1) / 2;
    let x = cx.saturating_sub(w / 2).min(world.0 - w);
    let y = cy.saturating_sub(h / 2).min(world.1 - h);
    Frame { x, y, w, h }
}

/// The frame for the next picture when following the colony. While a paint stroke is held
/// it stays put: a frame that zoomed out under a moving pointer would change which cell the
/// pointer is over, and a straight drag would paint a crooked line. It catches up once the
/// stroke ends.
pub fn next(
    current: Frame,
    bounds: Option<Bounds>,
    world: (usize, usize),
    min_width: usize,
    painting: bool,
) -> Frame {
    if painting {
        current
    } else {
        follow(Some(current), bounds, world, min_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(x0: usize, y0: usize, x1: usize, y1: usize) -> Option<Bounds> {
        Some(Bounds { x0, y0, x1, y1 })
    }

    /// A single loop in the middle of a big world is shown close up, centred, in the world's
    /// proportions.
    #[test]
    fn starts_close_in_and_centred() {
        let f = follow(None, bounds(295, 255, 310, 265), (600, 520), 60);
        assert_eq!(f.w, 60);
        assert_eq!(f.h, 52);
        assert!(f.x <= 291 && f.x + f.w >= 314, "{f:?}");
        assert!(f.y <= 251 && f.y + f.h >= 269, "{f:?}");
    }

    /// The frame stays put while the colony fits in it with its margin, and grows (never
    /// shrinks) once it doesn't, still covering the whole colony.
    #[test]
    fn grows_in_jumps_only_when_needed() {
        let world = (600, 520);
        let f = follow(None, bounds(295, 255, 310, 265), world, 60);
        assert_eq!(follow(Some(f), bounds(294, 254, 312, 268), world, 60), f);
        let b = bounds(280, 250, 340, 270);
        let g = follow(Some(f), b, world, 60);
        assert!(g.w > f.w && g.h > f.h);
        assert!(g.contains(&b.unwrap()));
    }

    /// Near an edge, or bigger than the world, the frame is clamped inside it.
    #[test]
    fn stays_inside_the_world() {
        let world = (300, 260);
        let f = follow(None, bounds(0, 0, 20, 20), world, 60);
        assert_eq!((f.x, f.y), (0, 0));
        let f = follow(None, bounds(0, 0, 300, 260), world, 60);
        assert_eq!(f, Frame::whole(world));
        let f = follow(None, bounds(280, 240, 300, 260), world, 60);
        assert_eq!((f.x + f.w, f.y + f.h), (300, 260));
    }

    /// Painting into the margin while a stroke is held must not move the frame, or the
    /// pointer would land on other cells mid-stroke; once released, the frame grows to take
    /// in what was painted. A straight drag across the frame then maps to a straight row of
    /// world cells, however far it reaches.
    #[test]
    fn the_frame_holds_still_while_painting() {
        let world = (600, 520);
        let f = follow(None, bounds(295, 255, 310, 265), world, 60);
        // A stroke along the frame's top row, from edge to edge, painted into the margin.
        let row: Vec<_> = (0..f.w).map(|x| f.to_world((x, 0))).collect();
        assert!(row.iter().all(|&(_, y)| y == f.y));
        assert!(row.windows(2).all(|p| p[1].0 == p[0].0 + 1));
        let painted = bounds(f.x, f.y, f.x + f.w, 265);
        assert_eq!(next(f, painted, world, 60, true), f);
        let after = next(f, painted, world, 60, false);
        assert!(after.w > f.w && after.contains(&painted.unwrap()));
    }
}
