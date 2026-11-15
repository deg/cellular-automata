//! Langton's ant, and its many-coloured generalisations.
//!
//! An ant stands on a grid of coloured cells, facing up, down, left or right. Each step it
//! looks at the colour under it, turns left or right as the rule says for that colour,
//! changes the cell to the next colour, and moves forward one cell. Langton's original ant
//! has two colours and the rule `RL`: on colour 0 turn right, on colour 1 turn left.

use ca_core::Grid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Turn {
    Left,
    Right,
}

/// For each colour in turn, which way the ant turns on it. The number of colours is the
/// rule's length, and cells cycle through the colours in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule(Vec<Turn>);

impl Rule {
    /// Parse a rule such as `RL` (Langton's ant) or `LLRR`.
    pub fn parse(text: &str) -> Result<Self, String> {
        let turns = text
            .trim()
            .chars()
            .map(|c| match c.to_ascii_uppercase() {
                'L' => Ok(Turn::Left),
                'R' => Ok(Turn::Right),
                other => Err(format!("'{other}' is not L or R")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !(2..=12).contains(&turns.len()) {
            return Err("a rule needs 2 to 12 letters, one per colour".into());
        }
        Ok(Self(turns))
    }

    pub fn colours(&self) -> usize {
        self.0.len()
    }
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for t in &self.0 {
            f.write_str(if *t == Turn::Left { "L" } else { "R" })?;
        }
        Ok(())
    }
}

/// Facing up, right, down, left, in clockwise order so a right turn is +1.
const HEADINGS: [(isize, isize); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ant {
    pub x: usize,
    pub y: usize,
    /// Index into [`HEADINGS`].
    heading: usize,
}

impl Ant {
    /// An ant at `(x, y)` facing up.
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y, heading: 0 }
    }
}

/// A grid whose edges wrap around (a torus), with any number of ants.
pub struct World {
    grid: Grid<u8>,
    pub ants: Vec<Ant>,
    rule: Rule,
    pub steps: u64,
}

impl World {
    /// A blank world with one ant at the centre.
    pub fn new(side: usize, rule: Rule) -> Self {
        let c = side / 2;
        Self {
            grid: Grid::new(side, side, 0),
            ants: vec![Ant::new(c, c)],
            rule,
            steps: 0,
        }
    }

    pub fn grid(&self) -> &Grid<u8> {
        &self.grid
    }

    pub fn rule(&self) -> &Rule {
        &self.rule
    }

    /// Move every ant once. Ants move in list order, so when two share a cell the second
    /// sees the colour the first left behind.
    pub fn step(&mut self) {
        let side = self.grid.width() as isize;
        let colours = self.rule.colours() as u8;
        for ant in &mut self.ants {
            let i = self.grid.index_of(ant.x, ant.y);
            let colour = self.grid[i];
            ant.heading = match self.rule.0[colour as usize] {
                Turn::Right => (ant.heading + 1) % 4,
                Turn::Left => (ant.heading + 3) % 4,
            };
            self.grid[i] = (colour + 1) % colours;
            let (dx, dy) = HEADINGS[ant.heading];
            ant.x = (ant.x as isize + dx).rem_euclid(side) as usize;
            ant.y = (ant.y as isize + dy).rem_euclid(side) as usize;
        }
        self.steps += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn langton(side: usize) -> World {
        World::new(side, Rule::parse("RL").unwrap())
    }

    /// The first moves worked by hand. On a blank (colour 0) cell, RL turns right: facing up,
    /// it turns to face right, colours the cell 1 and steps right. That repeats on fresh
    /// cells, so it walks a small clockwise square back to its start, where it now finds
    /// colour 1 and turns left. Catches a swapped L/R or a wrong heading order.
    #[test]
    fn first_steps_match_a_hand_trace() {
        let mut w = langton(11);
        let mut path = vec![];
        for _ in 0..5 {
            w.step();
            let a = w.ants[0];
            path.push((a.x as isize - 5, a.y as isize - 5));
        }
        assert_eq!(path, vec![(1, 0), (1, 1), (0, 1), (0, 0), (-1, 0)]);
    }

    /// After a chaotic start, Langton's ant builds a "highway": a 104-step cycle that moves
    /// it two cells diagonally, forever. This finds when the cycle starts, and checks it is
    /// near 10,000 steps, as the README says. It also checks the engine, since any mistake
    /// in the rules would derail such a long, sensitive run.
    #[test]
    fn langtons_ant_builds_a_highway_after_about_10000_steps() {
        let mut w = langton(400); // big enough that the ant never wraps in 12,000 steps
        let mut positions = Vec::new();
        for _ in 0..12_000 {
            positions.push((w.ants[0].x as isize, w.ants[0].y as isize));
            w.step();
        }
        let shift = |t: usize| {
            let (a, b) = (positions[t], positions[t + 104]);
            (b.0 - a.0, b.1 - a.1)
        };
        let end = positions.len() - 104;
        // The highway is underway from `onset` if every later window of 104 steps moves the
        // ant by the same diagonal (2, 2) step.
        let highway = shift(end - 1);
        assert_eq!((highway.0.abs(), highway.1.abs()), (2, 2), "{highway:?}");
        let onset = (0..end)
            .rev()
            .take_while(|&t| shift(t) == highway)
            .last()
            .unwrap();
        assert!(
            (9_000..=10_500).contains(&onset),
            "highway from step {onset}"
        );
    }

    /// Cells cycle through all the rule's colours in order, so a many-coloured rule must use
    /// every colour and nothing beyond.
    #[test]
    fn colours_cycle_within_the_rule() {
        let mut w = World::new(64, Rule::parse("LLRR").unwrap());
        for _ in 0..5_000 {
            w.step();
        }
        let max = *w.grid().cells().iter().max().unwrap();
        assert_eq!(max, 3);
    }

    /// Rules are what users type, so bad letters and bad lengths must be errors to show,
    /// and case must not matter.
    #[test]
    fn rules_parse() {
        assert_eq!(Rule::parse("rl").unwrap().to_string(), "RL");
        assert!(Rule::parse("RX").is_err());
        assert!(Rule::parse("R").is_err());
    }

    /// Two ants step in order, and the world wraps: an ant leaving one edge reappears on
    /// the opposite one.
    #[test]
    fn ants_wrap_around_the_edges() {
        let mut w = langton(5);
        w.ants = vec![Ant::new(4, 2)];
        w.step(); // blank cell: turn right (now facing right), step off the right edge
        assert_eq!((w.ants[0].x, w.ants[0].y), (0, 2));
    }
}
