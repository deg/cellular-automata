//! Turning escape counts into colours.
//!
//! The count of a point outside the set runs from 1 up to the iteration limit, which climbs
//! into the thousands as you zoom in, so a gradient stretched once over that range would
//! use only a sliver of it at any one zoom. Instead the gradient repeats every `period`
//! steps.

use crate::sim::Pixel;
use ca_core::egui::Color32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palette {
    /// The site's own colours: navy, blue, teal, gold and coral.
    Site,
    Fire,
    Ice,
    Grey,
}

impl Palette {
    pub const ALL: [Palette; 4] = [Self::Site, Self::Fire, Self::Ice, Self::Grey];

    pub fn name(self) -> &'static str {
        match self {
            Self::Site => "site",
            Self::Fire => "fire",
            Self::Ice => "ice",
            Self::Grey => "grey",
        }
    }

    /// The colour of points that have not escaped.
    pub fn inside(self) -> Color32 {
        match self {
            Self::Site => Color32::from_rgb(0x14, 0x18, 0x30),
            Self::Fire | Self::Grey => Color32::BLACK,
            Self::Ice => Color32::from_rgb(0x02, 0x08, 0x16),
        }
    }

    /// The gradient's colours, in order; the last blends back into the first.
    fn stops(self) -> &'static [[u8; 3]] {
        match self {
            Self::Site => &[
                [0x1f, 0x2a, 0x55],
                [0x3a, 0x6e, 0xa5],
                [0x2a, 0x9d, 0x8f],
                [0xe9, 0xc4, 0x6a],
                [0xe7, 0x6f, 0x51],
                [0x6a, 0x2c, 0x50],
            ],
            Self::Fire => &[
                [0x18, 0x02, 0x00],
                [0x8c, 0x14, 0x00],
                [0xe6, 0x5a, 0x00],
                [0xff, 0xb8, 0x2a],
                [0xff, 0xf4, 0xc8],
                [0x70, 0x10, 0x00],
            ],
            Self::Ice => &[
                [0x06, 0x1a, 0x3a],
                [0x16, 0x56, 0x96],
                [0x46, 0xa6, 0xd8],
                [0xd8, 0xf2, 0xff],
                [0x2c, 0x6c, 0xaa],
            ],
            Self::Grey => &[[0x18, 0x18, 0x18], [0xf0, 0xf0, 0xf0]],
        }
    }
}

/// Entries in the gradient's lookup table, one cycle of it.
const TABLE: usize = 1024;

/// How pixels are coloured: a palette, how often it repeats, and whether to show the
/// escape count as flat bands.
pub struct Colours {
    palette: Palette,
    table: Vec<Color32>,
    /// Steps per repeat of the gradient.
    pub period: f32,
    /// Colour by the whole-number count, so each step is one flat band.
    pub bands: bool,
}

impl Colours {
    pub fn new(palette: Palette) -> Self {
        let mut colours = Self {
            palette,
            table: Vec::new(),
            period: 32.0,
            bands: false,
        };
        colours.set_palette(palette);
        colours
    }

    pub fn palette(&self) -> Palette {
        self.palette
    }

    pub fn set_palette(&mut self, palette: Palette) {
        self.palette = palette;
        self.table = (0..TABLE)
            .map(|i| gradient(palette.stops(), i as f32 / TABLE as f32))
            .collect();
    }

    /// The colour of a pixel that has been computed; `None` for a fresh one.
    pub fn of(&self, pixel: Pixel) -> Option<Color32> {
        Some(match pixel {
            Pixel::Fresh => return None,
            Pixel::Running(_) | Pixel::Inside => self.palette.inside(),
            Pixel::Escaped { step, smooth } => {
                let count = if self.bands { step as f32 } else { smooth };
                let t = (count / self.period).rem_euclid(1.0);
                self.table[((t * TABLE as f32) as usize).min(TABLE - 1)]
            }
        })
    }
}

/// The colour a fraction `t` (0 to 1) of the way round a cyclic gradient through `stops`.
fn gradient(stops: &[[u8; 3]], t: f32) -> Color32 {
    let n = stops.len();
    let at = t * n as f32;
    let i = (at as usize).min(n - 1);
    let f = at - i as f32;
    let (a, b) = (stops[i], stops[(i + 1) % n]);
    let mix = |k: usize| (a[k] as f32 + (b[k] as f32 - a[k] as f32) * f).round() as u8;
    Color32::from_rgb(mix(0), mix(1), mix(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The gradient passes through each stop and wraps round smoothly from the last to the
    /// first, so the repeats have no seam.
    #[test]
    fn gradient_hits_its_stops_and_wraps() {
        let stops = Palette::Site.stops();
        let n = stops.len();
        for (i, s) in stops.iter().enumerate() {
            assert_eq!(
                gradient(stops, i as f32 / n as f32),
                Color32::from_rgb(s[0], s[1], s[2])
            );
        }
        let end = gradient(stops, 0.9999);
        let start = gradient(stops, 0.0);
        for (e, s) in end.to_array().iter().zip(start.to_array()) {
            assert!(e.abs_diff(s) <= 1);
        }
    }

    /// In band mode every point that escapes at the same step gets the same colour, and the
    /// next step a different one.
    #[test]
    fn bands_follow_the_whole_count() {
        let mut c = Colours::new(Palette::Fire);
        c.bands = true;
        let at = |step, smooth| c.of(Pixel::Escaped { step, smooth });
        assert_eq!(at(7, 6.1), at(7, 6.9));
        assert_ne!(at(7, 6.5), at(8, 7.5));
        assert_eq!(c.of(Pixel::Running(5)), Some(Palette::Fire.inside()));
        assert_eq!(c.of(Pixel::Fresh), None);
    }
}
