//! Double-double numbers: a value held as the unevaluated sum of two f64s, `hi + lo`, with
//! `|lo|` at most half a unit in the last place of `hi`. That gives about 106 bits, some 32
//! decimal digits, against f64's 53 bits and 16 digits: enough to place a view zoomed in
//! about 10^28 times, where f64 runs out near 10^12.
//!
//! Only what deep zooms need: adding, subtracting, multiplying, and printing. The building
//! blocks are the classic error-free transformations (Knuth's two-sum, Dekker's two-product
//! by splitting), which use only plain f64 arithmetic: `f64::mul_add` would be shorter, but
//! WebAssembly has no fused multiply-add, and there it is a slow library call.

use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Dd {
    pub hi: f64,
    pub lo: f64,
}

impl Dd {
    pub const ZERO: Dd = Dd { hi: 0.0, lo: 0.0 };

    pub fn new(x: f64) -> Self {
        Self { hi: x, lo: 0.0 }
    }

    /// The nearest f64.
    pub fn to_f64(self) -> f64 {
        self.hi + self.lo
    }

    /// `self * k` for an f64 `k`.
    pub fn scale(self, k: f64) -> Self {
        let (p, e) = two_prod(self.hi, k);
        quick_two_sum(p, e + self.lo * k)
    }

    /// The value to `digits` decimal places, rounded, as "-1.2345". For the pointer readout,
    /// which needs more digits than an f64 has once a view is zoomed in far.
    pub fn to_decimal(self, digits: usize) -> String {
        let negative = self.hi < 0.0 || (self.hi == 0.0 && self.lo < 0.0);
        let mut x = if negative { -self } else { self };
        x = x + Dd::new(0.5 * 10f64.powi(-(digits as i32)));
        let whole = floor(x);
        let mut frac = x - whole;
        let mut out = String::new();
        if negative {
            out.push('-');
        }
        out.push_str(&format!("{}", whole.to_f64() as u64));
        if digits > 0 {
            out.push('.');
            for _ in 0..digits {
                frac = frac.scale(10.0);
                let d = floor(frac);
                frac = frac - d;
                out.push(char::from(b'0' + (d.to_f64() as u8).min(9)));
            }
        }
        out
    }
}

/// `a / b`, for the tests' Newton iterations: two rounds of long division.
#[cfg(test)]
pub fn div(a: Dd, b: Dd) -> Dd {
    let q1 = a.hi / b.hi;
    let r = a - b.scale(q1);
    let q2 = r.hi / b.hi;
    let r = r - b.scale(q2);
    let q3 = r.hi / b.hi;
    quick_two_sum(q1, q2) + Dd::new(q3)
}

/// The largest whole number not above `x`.
fn floor(x: Dd) -> Dd {
    let hi = x.hi.floor();
    if hi == x.hi {
        // `hi` is whole already: the fraction, if any, is in `lo`.
        quick_two_sum(hi, x.lo.floor())
    } else {
        Dd::new(hi)
    }
}

impl From<f64> for Dd {
    fn from(x: f64) -> Self {
        Self::new(x)
    }
}

impl Neg for Dd {
    type Output = Dd;
    fn neg(self) -> Dd {
        Dd {
            hi: -self.hi,
            lo: -self.lo,
        }
    }
}

impl Add for Dd {
    type Output = Dd;
    fn add(self, o: Dd) -> Dd {
        let (s, e) = two_sum(self.hi, o.hi);
        let (t, f) = two_sum(self.lo, o.lo);
        let (s, e) = quick_two_sum(s, e + t).parts();
        quick_two_sum(s, e + f)
    }
}

impl Sub for Dd {
    type Output = Dd;
    fn sub(self, o: Dd) -> Dd {
        self + -o
    }
}

impl Mul for Dd {
    type Output = Dd;
    fn mul(self, o: Dd) -> Dd {
        let (p, e) = two_prod(self.hi, o.hi);
        quick_two_sum(p, e + (self.hi * o.lo + self.lo * o.hi))
    }
}

impl Dd {
    fn parts(self) -> (f64, f64) {
        (self.hi, self.lo)
    }
}

/// `a + b` exactly, as the rounded sum and its error (Knuth).
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let v = s - a;
    (s, (a - (s - v)) + (b - v))
}

/// `a + b` exactly, given `|a| >= |b|`: cheaper than [`two_sum`].
fn quick_two_sum(a: f64, b: f64) -> Dd {
    let s = a + b;
    Dd {
        hi: s,
        lo: b - (s - a),
    }
}

/// Split `a` into two halves of 26 bits each, so their products are exact (Dekker).
fn split(a: f64) -> (f64, f64) {
    const SPLITTER: f64 = 134_217_729.0; // 2^27 + 1
    let t = SPLITTER * a;
    let hi = t - (t - a);
    (hi, a - hi)
}

/// `a * b` exactly, as the rounded product and its error (Dekker).
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let (ah, al) = split(a);
    let (bh, bl) = split(b);
    let e = ((ah * bh - p) + ah * bl + al * bh) + al * bl;
    (p, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sums and products must keep what f64 rounds away. 1e16 + 1 is not an f64 (the
    /// spacing there is 2), and (1 + 2^-30)² = 1 + 2^-29 + 2^-60 needs 61 bits.
    #[test]
    fn keeps_what_f64_rounds_away() {
        let s = Dd::new(1e16) + Dd::new(1.0);
        assert_eq!((s.hi, s.lo), (1e16, 1.0));
        assert_eq!((s - Dd::new(1e16)).to_f64(), 1.0);
        let a = Dd::new(1.0 + 2f64.powi(-30));
        let sq = a * a;
        assert_eq!(sq.hi, 1.0 + 2f64.powi(-29));
        assert_eq!(sq.lo, 2f64.powi(-60));
        let k = Dd::new(1.0 + 2f64.powi(-30)).scale(1.0 + 2f64.powi(-30));
        assert_eq!(k, sq);
    }

    /// A third, built as 1/3 in f64 plus its error, times 3 must come back to 1 within
    /// double-double's precision (about 1e-32), far inside f64's.
    #[test]
    fn a_third_times_three() {
        let third = {
            let hi = 1.0 / 3.0;
            let (p, e) = two_prod(hi, 3.0);
            Dd {
                hi,
                lo: (1.0 - p - e) / 3.0,
            }
        };
        let back = third.scale(3.0) - Dd::new(1.0);
        assert!(back.to_f64().abs() < 1e-31, "{back:?}");
    }

    /// Printing must show the digits beyond f64's 17: 0.1 as an f64 is exactly
    /// 0.1000000000000000055511151231257827..., and a double-double holds all of that.
    #[test]
    fn prints_more_digits_than_f64_has() {
        assert_eq!(Dd::new(0.1).to_decimal(20), "0.10000000000000000555");
        assert_eq!(Dd::new(-1.75).to_decimal(3), "-1.750");
        assert_eq!(Dd::new(2.0).to_decimal(0), "2");
        let tiny = Dd::new(0.5) + Dd::new(1e-25);
        assert_eq!(tiny.to_decimal(26), "0.50000000000000000000000010");
    }
}
