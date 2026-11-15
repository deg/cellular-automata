//! Where drips land: the cells that receive grains on every drip.

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
}
