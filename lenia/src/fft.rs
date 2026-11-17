//! A small fast Fourier transform for square worlds whose side is a power of two, written
//! here because the project takes no crates beyond egui.
//!
//! Lenia's potential is a convolution with a kernel some 13 to 20 cells in radius: about 530
//! weights a cell at radius 13, 1,250 at 20. By the convolution theorem it is also
//! `F⁻¹(F(K) · F(A))`, which costs a few transforms of the whole world however big the
//! kernel, and wraps round the edges by itself (Chan's programs do the same, his paper's
//! equation 18).
//!
//! **Layout.** Real and imaginary parts are separate `f32` arrays, row after row. A 2D
//! transform is a transform of every column, a transpose, and the columns again. Done a
//! column at a time, a butterfly combines two whole rows element by element: two slices of
//! equal length, which the compiler turns into SIMD (four lanes in the browser, with the
//! `simd128` feature `.cargo/config.toml` turns on). Transforming along rows directly would
//! need short strided loops instead.

use std::f64::consts::PI;

/// Precomputed tables for transforms of length `n`.
pub struct Fft {
    n: usize,
    /// `cos(2πk/n)` and `sin(2πk/n)` for `k < n/2`.
    cos: Vec<f32>,
    sin: Vec<f32>,
    /// `rev[i]` is `i` with its `log2 n` bits reversed: where the iterative transform wants
    /// row `i` to start.
    rev: Vec<usize>,
}

impl Fft {
    pub fn new(n: usize) -> Self {
        assert!(
            n.is_power_of_two() && n >= 2,
            "side {n} is not a power of two"
        );
        let bits = n.trailing_zeros();
        let angle = |k: usize| 2.0 * PI * k as f64 / n as f64;
        Self {
            n,
            cos: (0..n / 2).map(|k| angle(k).cos() as f32).collect(),
            sin: (0..n / 2).map(|k| angle(k).sin() as f32).collect(),
            rev: (0..n)
                .map(|i| i.reverse_bits() >> (usize::BITS - bits))
                .collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.n
    }

    /// Transform every column of the `n x n` array `(re, im)` in place: entry `(x, k)`
    /// becomes `Σ_y a(x, y) e^(∓2πi k y / n)`, minus in the exponent forward, plus for
    /// `inverse`. Unscaled both ways, so forward then inverse multiplies by `n`.
    pub fn columns(&self, re: &mut [f32], im: &mut [f32], inverse: bool) {
        let n = self.n;
        assert!(re.len() == n * n && im.len() == n * n);
        for (i, &j) in self.rev.iter().enumerate() {
            if i < j {
                swap_rows(re, n, i, j);
                swap_rows(im, n, i, j);
            }
        }
        let sign = if inverse { 1.0 } else { -1.0 };
        let mut half = 1;
        while half < n {
            let stride = n / (2 * half);
            for start in (0..n).step_by(2 * half) {
                for k in 0..half {
                    let (wr, wi) = (self.cos[k * stride], sign * self.sin[k * stride]);
                    let (top, bottom) = (start + k, start + k + half);
                    let (ar, br) = two_rows(re, n, top, bottom);
                    let (ai, bi) = two_rows(im, n, top, bottom);
                    butterfly(ar, ai, br, bi, wr, wi);
                }
            }
            half *= 2;
        }
    }

    /// The 2D transform of `(re, im)`, left transposed: entry `(kx, ky)` of the transform is
    /// at row `kx`, column `ky`. [`Fft::inverse_2d`] undoes it from that layout.
    pub fn forward_2d(&self, re: &mut [f32], im: &mut [f32]) {
        self.columns(re, im, false);
        transpose(re, self.n);
        transpose(im, self.n);
        self.columns(re, im, false);
    }

    /// The inverse of [`Fft::forward_2d`], scaled by `n²` (not divided by it).
    pub fn inverse_2d(&self, re: &mut [f32], im: &mut [f32]) {
        self.columns(re, im, true);
        transpose(re, self.n);
        transpose(im, self.n);
        self.columns(re, im, true);
    }
}

/// `a ± w b` into `(a, b)`, element by element, for whole rows.
fn butterfly(ar: &mut [f32], ai: &mut [f32], br: &mut [f32], bi: &mut [f32], wr: f32, wi: f32) {
    let rows = ar
        .iter_mut()
        .zip(ai.iter_mut())
        .zip(br.iter_mut().zip(bi.iter_mut()));
    for ((ar, ai), (br, bi)) in rows {
        let tr = wr * *br - wi * *bi;
        let ti = wr * *bi + wi * *br;
        *br = *ar - tr;
        *bi = *ai - ti;
        *ar += tr;
        *ai += ti;
    }
}

/// Rows `i < j` of an array `n` wide, both mutable.
fn two_rows(a: &mut [f32], n: usize, i: usize, j: usize) -> (&mut [f32], &mut [f32]) {
    let (head, tail) = a.split_at_mut(j * n);
    (&mut head[i * n..(i + 1) * n], &mut tail[..n])
}

fn swap_rows(a: &mut [f32], n: usize, i: usize, j: usize) {
    let (x, y) = two_rows(a, n, i, j);
    x.swap_with_slice(y);
}

/// Transpose an `n x n` array in place, in tiles so both sides stay in cache.
pub fn transpose(a: &mut [f32], n: usize) {
    const TILE: usize = 16;
    for by in (0..n).step_by(TILE) {
        for bx in (by..n).step_by(TILE) {
            for y in by..(by + TILE).min(n) {
                let x0 = if bx == by { y + 1 } else { bx };
                for x in x0..(bx + TILE).min(n) {
                    a.swap(y * n + x, x * n + y);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ca_core::Rng;

    fn random(n: usize, seed: u64) -> Vec<f32> {
        let mut rng = Rng::new(seed);
        (0..n * n).map(|_| rng.next_f64() as f32 - 0.5).collect()
    }

    /// The 2D transform against the definition, `Σ a(x, y) e^(−2πi (kx x + ky y) / n)`,
    /// summed directly in f64 on a small array.
    #[test]
    fn matches_the_definition() {
        let n = 8;
        let a = random(n, 1);
        let b = random(n, 2);
        let (mut re, mut im) = (a.clone(), b.clone());
        Fft::new(n).forward_2d(&mut re, &mut im);
        for kx in 0..n {
            for ky in 0..n {
                let (mut sr, mut si) = (0.0f64, 0.0f64);
                for y in 0..n {
                    for x in 0..n {
                        let t = -2.0 * PI * ((kx * x + ky * y) % n) as f64 / n as f64;
                        let (c, s) = (t.cos(), t.sin());
                        let (ar, ai) = (f64::from(a[y * n + x]), f64::from(b[y * n + x]));
                        sr += ar * c - ai * s;
                        si += ar * s + ai * c;
                    }
                }
                // Transposed layout: (kx, ky) at row kx.
                let i = kx * n + ky;
                assert!((f64::from(re[i]) - sr).abs() < 1e-4, "re ({kx}, {ky})");
                assert!((f64::from(im[i]) - si).abs() < 1e-4, "im ({kx}, {ky})");
            }
        }
    }

    /// Forward then inverse gives back `n²` times the start.
    #[test]
    fn inverse_undoes_forward() {
        let n = 64;
        let a = random(n, 3);
        let (mut re, mut im) = (a.clone(), vec![0.0; n * n]);
        let fft = Fft::new(n);
        fft.forward_2d(&mut re, &mut im);
        fft.inverse_2d(&mut re, &mut im);
        let scale = (n * n) as f32;
        for (i, &v) in a.iter().enumerate() {
            assert!((re[i] / scale - v).abs() < 1e-5);
            assert!((im[i] / scale).abs() < 1e-5);
        }
    }

    #[test]
    fn transpose_swaps_rows_and_columns() {
        let n = 40; // not a multiple of the tile
        let mut a: Vec<f32> = (0..n * n).map(|i| i as f32).collect();
        transpose(&mut a, n);
        for y in 0..n {
            for x in 0..n {
                assert_eq!(a[y * n + x], (x * n + y) as f32);
            }
        }
    }
}
