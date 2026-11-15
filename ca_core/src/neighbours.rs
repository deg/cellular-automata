use crate::Grid;

/// For every cell, how many of its 8 neighbours (the Moore neighbourhood) satisfy `counts`.
///
/// Life, Brian's Brain and Wireworld all update each cell from a count like this, so the
/// edge handling lives here once. With `wrap`, the grid is a torus and the left edge's
/// neighbours include the right edge's cells; without it, cells beyond the edge count as
/// not satisfying `counts`. Results go into `out` (resized to fit), reused across calls to
/// avoid allocating every generation.
pub fn moore_counts<T: Copy>(
    grid: &Grid<T>,
    wrap: bool,
    counts: impl Fn(T) -> bool,
    out: &mut Vec<u8>,
) {
    let (w, h) = (grid.width(), grid.height());
    out.clear();
    out.resize(w * h, 0);
    // 1 where a cell counts, so the sums below are plain additions.
    let hits: Vec<u8> = grid.cells().iter().map(|&c| counts(c) as u8).collect();
    let around = |i: usize, n: usize| -> [Option<usize>; 3] {
        let before = if i > 0 {
            Some(i - 1)
        } else {
            wrap.then_some(n - 1)
        };
        let after = if i + 1 < n {
            Some(i + 1)
        } else {
            wrap.then_some(0)
        };
        [before, Some(i), after]
    };
    let columns: Vec<_> = (0..w).map(|x| around(x, w)).collect();
    for y in 0..h {
        let rows = around(y, h);
        for x in 0..w {
            let mut sum = 0;
            for row in rows.iter().flatten() {
                for col in columns[x].iter().flatten() {
                    sum += hits[row * w + col];
                }
            }
            // The loops included the cell itself; a cell is not its own neighbour.
            out[y * w + x] = sum - hits[y * w + x];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cell must count all 8 neighbours but never itself: a lone live cell has 0 live
    /// neighbours, and each of the 8 around it has 1.
    #[test]
    fn counts_the_eight_neighbours_not_the_cell() {
        let mut g = Grid::new(5, 5, false);
        g.set(2, 2, true);
        let mut out = Vec::new();
        moore_counts(&g, false, |c| c, &mut out);
        assert_eq!(out[2 * 5 + 2], 0);
        let ones = out.iter().filter(|&&n| n == 1).count();
        assert_eq!(ones, 8);
    }

    /// With wrap, a corner cell's neighbours include the opposite edges and corners; without
    /// it, nothing beyond the edge counts. Wrong edge handling breaks gliders crossing edges.
    #[test]
    fn corners_wrap_only_when_asked() {
        let mut g = Grid::new(4, 3, false);
        g.set(3, 2, true); // the far corner
        let mut out = Vec::new();
        moore_counts(&g, true, |c| c, &mut out);
        assert_eq!(out[0], 1, "the near corner touches the far one on a torus");
        moore_counts(&g, false, |c| c, &mut out);
        assert_eq!(out[0], 0);
    }

    /// On a grid only 1 or 2 cells wide, wrapping makes the same neighbour appear more than
    /// once. That is the honest torus count, and it must not crash or underflow.
    #[test]
    fn tiny_wrapping_grids_do_not_underflow() {
        let g = Grid::new(1, 1, true);
        let mut out = Vec::new();
        moore_counts(&g, true, |c| c, &mut out);
        assert_eq!(out, vec![8]);
    }
}
