//! Where drips land: the cells that receive grains on every drip.

use ca_core::Layout;

/// Drip points, as `(dx, dy)` offsets from the pile's centre.
///
/// Offsets, not grid coordinates, because the grid grows evenly on every side: the centre
/// stays put, so an offset keeps naming the same spot after growth and across a Restart.
pub struct DripPoints {
    points: Vec<(isize, isize)>,
}

impl DripPoints {
    /// Just the centre, as in the classic sandpile.
    pub fn centre() -> Self {
        Self {
            points: vec![(0, 0)],
        }
    }

    pub fn points(&self) -> &[(isize, isize)] {
        &self.points
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn reset_to_centre(&mut self) {
        *self = Self::centre();
    }

    /// Move every point to the cell at the same place on screen in another layout.
    ///
    /// The same offset means different places in the two: square `(dx, dy)` is `dx` cells
    /// across and `dy` down, but hex axial `(dq, dr)` is `dq + dr/2` across and
    /// `dr * sqrt(3)/2` down (in cell widths). Without this, switching layout would make
    /// every point off the centre row jump.
    pub fn relayout(&mut self, from: Layout, to: Layout) {
        if from == to {
            return;
        }
        for p in &mut self.points {
            *p = match to {
                Layout::Hex => square_to_hex(*p),
                Layout::Square => hex_to_square(*p),
            };
        }
    }

    /// Remove the point nearest `at`, if one lies within `tolerance` cells of it; otherwise
    /// add a point at `at`. Returns true if it added one.
    ///
    /// The tolerance matters because on a big grid a cell is smaller than a pixel, and nobody
    /// can click the exact cell they marked earlier.
    pub fn toggle(&mut self, at: (isize, isize), tolerance: isize) -> bool {
        let distance = |p: &(isize, isize)| (p.0 - at.0).abs().max((p.1 - at.1).abs());
        let nearest = self
            .points
            .iter()
            .enumerate()
            .min_by_key(|(_, p)| distance(p))
            .filter(|(_, p)| distance(p) <= tolerance)
            .map(|(i, _)| i);
        match nearest {
            Some(i) => {
                self.points.remove(i);
                false
            }
            None => {
                self.points.push(at);
                true
            }
        }
    }
}

/// The hex cell (axial offset) nearest the square cell `(dx, dy)` on screen.
fn square_to_hex((dx, dy): (isize, isize)) -> (isize, isize) {
    let dr = (dy as f64 * 2.0 / 3f64.sqrt()).round();
    let dq = (dx as f64 - dr / 2.0).round();
    (dq as isize, dr as isize)
}

/// The square cell nearest the hex cell (axial offset) `(dq, dr)` on screen.
fn hex_to_square((dq, dr): (isize, isize)) -> (isize, isize) {
    let dx = (dq as f64 + dr as f64 / 2.0).round();
    let dy = (dr as f64 * 3f64.sqrt() / 2.0).round();
    (dx as isize, dy as isize)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shift-clicking empty space must add a point there, and shift-clicking it again must
    /// remove it: that pair is the whole user interface for drip points.
    #[test]
    fn toggle_adds_then_removes() {
        let mut d = DripPoints::centre();
        assert!(d.toggle((20, -5), 0));
        assert_eq!(d.points(), &[(0, 0), (20, -5)]);
        assert!(!d.toggle((20, -5), 0));
        assert_eq!(d.points(), &[(0, 0)]);
    }

    /// A click near a point, within the tolerance, removes that point rather than adding a
    /// second one beside it; and of two points in range, the nearer one goes.
    #[test]
    fn toggle_removes_the_nearest_point_in_range() {
        let mut d = DripPoints::centre();
        d.toggle((10, 0), 0);
        d.toggle((14, 0), 0);
        assert!(!d.toggle((13, 1), 3));
        assert_eq!(d.points(), &[(0, 0), (10, 0)]);
    }

    /// A click just outside the tolerance is a new point, not a removal.
    #[test]
    fn toggle_outside_tolerance_adds() {
        let mut d = DripPoints::centre();
        assert!(d.toggle((4, 0), 3));
        assert_eq!(d.len(), 2);
    }

    /// Removing the centre is allowed, leaving no drip points at all; reset restores it.
    #[test]
    fn all_points_can_go_and_reset_brings_back_the_centre() {
        let mut d = DripPoints::centre();
        d.toggle((0, 0), 0);
        assert_eq!(d.len(), 0);
        d.reset_to_centre();
        assert_eq!(d.points(), &[(0, 0)]);
    }

    /// Switching layout must leave drip points where they were on screen, so going to hex
    /// and back returns each point to within a cell of where it started (rounding to the
    /// nearest cell each way can't do better), and the centre stays exactly the centre.
    #[test]
    fn relayout_keeps_points_in_place() {
        let start = [(0, 0), (20, 20), (-13, 7), (5, -30), (40, 0)];
        let mut d = DripPoints {
            points: start.to_vec(),
        };
        d.relayout(Layout::Square, Layout::Hex);
        assert_eq!(d.points()[0], (0, 0));
        assert_eq!(
            d.points()[4],
            (40, 0),
            "points on the centre row don't move"
        );
        d.relayout(Layout::Hex, Layout::Square);
        for (&(x0, y0), &(x1, y1)) in start.iter().zip(d.points()) {
            assert!(
                (x0 - x1).abs() <= 1 && (y0 - y1).abs() <= 1,
                "({x0}, {y0}) came back as ({x1}, {y1})"
            );
        }
    }

    /// The conversion must follow the screen geometry, not just copy the numbers: a square
    /// point 26 down is 30 hex rows down (rows are sqrt(3)/2 of a cell apart), and the axial
    /// q shrinks by half the rows to stay in the same column. (An even row, because odd hex
    /// rows sit half a cell off the square columns, where "the same column" is a tie.)
    #[test]
    fn square_to_hex_follows_the_screen() {
        assert_eq!(square_to_hex((0, 26)), (-15, 30));
        assert_eq!(hex_to_square((-15, 30)), (0, 26));
    }
}
