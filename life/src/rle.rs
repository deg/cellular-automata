//! Run Length Encoded (RLE) patterns: the standard text format for sharing Life patterns,
//! used by LifeWiki and the Golly simulator.
//!
//! A pattern is a header line `x = 3, y = 3` (optional here), then rows of tokens: `b` is a
//! dead cell, `o` a live one, `$` ends a row, `!` ends the pattern, and a number before a
//! token repeats it. Lines starting with `#` are comments. So a glider is `bob$2bo$3o!`.

/// A named pattern, with a one-line note shown in the picker.
pub struct Pattern {
    pub name: &'static str,
    pub note: &'static str,
    pub rle: &'static str,
}

/// Built-in patterns. Each RLE is checked by a test of its known behaviour in `sim.rs`, or
/// at least by parsing.
pub const PATTERNS: &[Pattern] = &[
    Pattern {
        name: "Glider",
        note: "The smallest spaceship: moves one cell diagonally every 4 generations.",
        rle: "bob$2bo$3o!",
    },
    Pattern {
        name: "Lightweight spaceship",
        note: "Moves sideways, two cells every 4 generations.",
        rle: "bo2bo$o4b$o3bo$4o!",
    },
    Pattern {
        name: "Gosper glider gun",
        note: "The first pattern known to grow forever (1970): fires a glider every 30 generations.",
        rle: "24bo$22bobo$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o$2o8bo3bob2o4bobo$10bo5bo7bo$11bo3bo$12b2o!",
    },
    Pattern {
        name: "Pulsar",
        note: "A large oscillator with period 3.",
        rle: "2b3o3b3o2b2$o4bobo4bo$o4bobo4bo$o4bobo4bo$2b3o3b3o2b2$2b3o3b3o2b$o4bobo4bo$o4bobo4bo$o4bobo4bo2$2b3o3b3o!",
    },
    Pattern {
        name: "R-pentomino",
        note: "Five cells that take 1103 generations to settle (on an unbounded plane).",
        rle: "b2o$2o$bo!",
    },
    Pattern {
        name: "Diehard",
        note: "Seven cells that vanish completely after exactly 130 generations.",
        rle: "6bob$2o6b$bo3b3o!",
    },
    Pattern {
        name: "Acorn",
        note: "Seven cells that take 5206 generations to settle (on an unbounded plane).",
        rle: "bo5b$3bo3b$2o2b3o!",
    },
];

/// A parsed pattern: its live cells, and the size of the box they sit in.
#[derive(Debug, PartialEq, Eq)]
pub struct Shape {
    pub cells: Vec<(usize, usize)>,
    pub size: (usize, usize),
}

/// The largest repeat count accepted: far beyond any world here (640 cells wide), but small
/// enough that a mistyped or hostile count can't make the parser allocate billions of cells,
/// or overflow a coordinate.
const MAX_COUNT: usize = 10_000;

/// Parse an RLE pattern.
pub fn parse(text: &str) -> Result<Shape, String> {
    let mut cells = Vec::new();
    let (mut x, mut y, mut width) = (0usize, 0usize, 0usize);
    let mut count = String::new();
    'lines: for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with('x') || line.is_empty() {
            continue;
        }
        for c in line.chars() {
            let n = || -> Result<usize, String> {
                if count.is_empty() {
                    return Ok(1);
                }
                match count.parse::<usize>() {
                    Ok(n) if n <= MAX_COUNT => Ok(n),
                    _ => Err(format!(
                        "repeat count {count} is too large (at most {MAX_COUNT})"
                    )),
                }
            };
            match c {
                '0'..='9' => {
                    count.push(c);
                    continue;
                }
                'b' | '.' => x += n()?,
                '$' => {
                    y += n()?;
                    x = 0;
                }
                '!' => break 'lines,
                c if c.is_whitespace() => continue,
                // `o`, and the letters multi-state rules use, all count as alive here.
                c if c.is_ascii_alphabetic() => {
                    for _ in 0..n()? {
                        cells.push((x, y));
                        x += 1;
                    }
                }
                other => return Err(format!("unexpected '{other}' in RLE")),
            }
            width = width.max(x);
            count.clear();
        }
    }
    let height = cells.iter().map(|&(_, y)| y + 1).max().unwrap_or(0);
    if cells.is_empty() {
        return Err("the pattern has no live cells".into());
    }
    Ok(Shape {
        cells,
        size: (width, height),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The glider is the example everyone checks RLE against: three rows, five cells, with a
    /// repeat count and both dead-cell and row-end tokens.
    #[test]
    fn parses_a_glider() {
        let Shape { cells, size } = parse("x = 3, y = 3, rule = B3/S23\nbob$2bo$3o!").unwrap();
        assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
        assert_eq!(size, (3, 3));
    }

    /// A count before `$` means that many row ends, i.e. blank rows: the pulsar relies on it.
    #[test]
    fn a_counted_row_end_skips_blank_rows() {
        let Shape { cells, size } = parse("o$2$o!").unwrap();
        assert_eq!(cells, vec![(0, 0), (0, 3)]);
        assert_eq!(size, (1, 4));
    }

    /// Pasted patterns carry comment lines and line breaks mid-row; both must be ignored.
    #[test]
    fn ignores_comments_and_line_breaks() {
        let Shape { cells, .. } = parse("#N Blinker\n#C a comment\nx = 3, y = 1\n2o\no!").unwrap();
        assert_eq!(cells, vec![(0, 0), (1, 0), (2, 0)]);
    }

    /// A huge repeat count must be an error, not a billion-cell allocation that hangs the app,
    /// nor a coordinate overflow (a panic in debug builds, silent garbage in release).
    #[test]
    fn rejects_huge_repeat_counts() {
        assert!(parse("1000000000o!").is_err());
        assert!(parse("18446744073709551615b2bo!").is_err());
        assert!(parse("10000o!").is_ok());
    }

    /// Garbage should be an error the UI can show, not a panic or an empty pattern.
    #[test]
    fn rejects_garbage() {
        assert!(parse("hello, world").is_err());
        assert!(parse("").is_err());
    }
}
