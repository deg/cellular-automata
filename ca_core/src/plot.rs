//! A small chart for a demo's controls panel: points or lines on linear or logarithmic axes,
//! sized to the panel. Demos that measure something as they run (the sizes of fires, the
//! flow of traffic against its density) show it here, so their charts look alike.

use crate::egui::{self, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, pos2, vec2};
use crate::grouped;

/// How a series is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// A dot at each point.
    Dots,
    /// The points joined in order.
    Line,
}

/// One set of points in a [`Plot`].
pub struct Series<'a> {
    pub points: &'a [(f64, f64)],
    pub colour: Color32,
    pub style: Style,
}

/// A chart: its axes and series. Points with a coordinate that can't be shown (not finite,
/// or not positive on a logarithmic axis) are skipped.
pub struct Plot<'a> {
    pub x_label: &'a str,
    pub y_label: &'a str,
    pub log_x: bool,
    pub log_y: bool,
    /// Fixed ranges; `None` fits the data.
    pub x_range: Option<(f64, f64)>,
    pub y_range: Option<(f64, f64)>,
    pub series: Vec<Series<'a>>,
}

/// Draw `plot` across the panel's width, `height` points tall (labels included).
pub fn plot(ui: &mut egui::Ui, plot: &Plot<'_>, height: f32) {
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let text = ui.visuals().text_color();
    let weak = ui.visuals().weak_text_color();
    let font = FontId::proportional(10.0);

    // Room for the tick labels on the left and below, and the axis names.
    let area = Rect::from_min_max(rect.min + vec2(34.0, 4.0), rect.max - vec2(6.0, 26.0));
    let x_axis = Axis::fit(
        plot.series.iter().map(|s| s.points.iter().map(|p| p.0)),
        plot.log_x,
        plot.x_range,
    );
    let y_axis = Axis::fit(
        plot.series.iter().map(|s| s.points.iter().map(|p| p.1)),
        plot.log_y,
        plot.y_range,
    );

    painter.rect_stroke(area, 0.0, Stroke::new(1.0, weak), egui::StrokeKind::Inside);
    for v in x_axis.ticks() {
        let x = area.left() + x_axis.fraction(v) as f32 * area.width();
        painter.line_segment(
            [pos2(x, area.bottom()), pos2(x, area.bottom() + 3.0)],
            Stroke::new(1.0, weak),
        );
        painter.text(
            pos2(x, area.bottom() + 4.0),
            Align2::CENTER_TOP,
            label(v),
            font.clone(),
            weak,
        );
    }
    for v in y_axis.ticks() {
        let y = area.bottom() - y_axis.fraction(v) as f32 * area.height();
        painter.line_segment(
            [pos2(area.left() - 3.0, y), pos2(area.left(), y)],
            Stroke::new(1.0, weak),
        );
        painter.text(
            pos2(area.left() - 4.0, y),
            Align2::RIGHT_CENTER,
            label(v),
            font.clone(),
            weak,
        );
    }
    painter.text(
        pos2(area.center().x, rect.bottom()),
        Align2::CENTER_BOTTOM,
        plot.x_label,
        font.clone(),
        text,
    );
    painter.text(
        pos2(area.left() + 4.0, area.top() + 2.0),
        Align2::LEFT_TOP,
        plot.y_label,
        font,
        text,
    );

    for series in &plot.series {
        let shown: Vec<Pos2> = series
            .points
            .iter()
            .filter(|&&(x, y)| x_axis.shows(x) && y_axis.shows(y))
            .map(|&(x, y)| {
                pos2(
                    area.left() + x_axis.fraction(x) as f32 * area.width(),
                    area.bottom() - y_axis.fraction(y) as f32 * area.height(),
                )
            })
            .collect();
        match series.style {
            Style::Dots => {
                for p in shown {
                    painter.circle_filled(p, 1.5, series.colour);
                }
            }
            Style::Line => {
                painter.add(Shape::line(shown, Stroke::new(1.5, series.colour)));
            }
        }
    }
}

/// One axis: its range, and whether it is logarithmic. Positions are fractions of the
/// axis's length, 0 at the low end.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Axis {
    lo: f64,
    hi: f64,
    log: bool,
}

impl Axis {
    /// The range of `values` (or `fixed`), widened to whole decades on a log axis.
    fn fit<I, J>(values: I, log: bool, fixed: Option<(f64, f64)>) -> Self
    where
        I: Iterator<Item = J>,
        J: Iterator<Item = f64>,
    {
        let (mut lo, mut hi) = fixed.unwrap_or_else(|| {
            values
                .flatten()
                .filter(|v| v.is_finite() && (!log || *v > 0.0))
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                    (lo.min(v), hi.max(v))
                })
        });
        if !lo.is_finite() || !hi.is_finite() {
            (lo, hi) = if log { (1.0, 10.0) } else { (0.0, 1.0) };
        }
        if log {
            lo = 10f64.powf(lo.log10().floor());
            hi = 10f64.powf(hi.log10().ceil()).max(lo * 10.0);
        } else if hi <= lo {
            hi = lo + 1.0;
        }
        Self { lo, hi, log }
    }

    fn shows(&self, v: f64) -> bool {
        v.is_finite() && (!self.log || v > 0.0) && v >= self.lo && v <= self.hi
    }

    fn fraction(&self, v: f64) -> f64 {
        if self.log {
            (v.log10() - self.lo.log10()) / (self.hi.log10() - self.lo.log10())
        } else {
            (v - self.lo) / (self.hi - self.lo)
        }
    }

    /// Tick values: each decade on a log axis (thinned to at most six), or about five
    /// round steps (1, 2 or 5 times a power of ten) on a linear one.
    fn ticks(&self) -> Vec<f64> {
        if self.log {
            let (a, b) = (
                self.lo.log10().round() as i32,
                self.hi.log10().round() as i32,
            );
            let every = ((b - a) / 6 + 1).max(1);
            (a..=b)
                .step_by(every as usize)
                .map(|k| 10f64.powi(k))
                .collect()
        } else {
            let raw = (self.hi - self.lo) / 5.0;
            let power = 10f64.powf(raw.log10().floor());
            let step = [1.0, 2.0, 5.0, 10.0]
                .into_iter()
                .map(|m| m * power)
                .find(|s| *s >= raw)
                .unwrap_or(10.0 * power);
            let first = (self.lo / step).ceil() as i64;
            let last = (self.hi / step).floor() as i64;
            (first..=last).map(|i| i as f64 * step).collect()
        }
    }
}

/// A tick label: whole numbers grouped (1,000), small ones to two places, negative ones with
/// a true minus sign.
fn label(v: f64) -> String {
    let body = if v.abs() >= 1.0 && v.fract() == 0.0 {
        grouped(v.abs() as u64)
    } else if v.abs() >= 1e-3 || v == 0.0 {
        let s = format!("{:.3}", v.abs());
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        format!("{:.0e}", v.abs())
    };
    if v < 0.0 {
        format!("\u{2212}{body}")
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis(values: &[f64], log: bool) -> Axis {
        Axis::fit(std::iter::once(values.iter().copied()), log, None)
    }

    #[test]
    fn a_log_axis_spans_whole_decades_and_skips_non_positive_values() {
        let a = axis(&[0.0, -3.0, 3.0, 4_500.0], true);
        assert_eq!((a.lo, a.hi), (1.0, 10_000.0));
        assert_eq!(a.ticks(), vec![1.0, 10.0, 100.0, 1_000.0, 10_000.0]);
        assert!(!a.shows(0.0));
        assert!((a.fraction(100.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn a_linear_axis_has_round_ticks_inside_its_range() {
        let a = axis(&[0.0, 0.93], false);
        let ticks: Vec<String> = a.ticks().iter().map(|&t| label(t)).collect();
        assert_eq!(ticks, ["0", "0.2", "0.4", "0.6", "0.8"]);
        assert_eq!(a.fraction(0.0), 0.0);
        assert_eq!(a.fraction(0.93), 1.0);
    }

    #[test]
    fn empty_data_still_gives_an_axis() {
        let a = axis(&[], false);
        assert_eq!((a.lo, a.hi), (0.0, 1.0));
        let l = axis(&[], true);
        assert_eq!((l.lo, l.hi), (1.0, 10.0));
    }

    #[test]
    fn labels_read_like_the_readmes() {
        assert_eq!(label(1_000.0), "1,000");
        assert_eq!(label(0.25), "0.25");
        assert_eq!(label(0.0), "0");
        assert_eq!(label(-0.5), "\u{2212}0.5");
    }
}
