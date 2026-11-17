//! Lenia, as Bert Wang-Chak Chan defined it in "Lenia: Biology of Artificial Life",
//! Complex Systems 28 (2019) 251–286 (arXiv:1812.05433v3; section numbers below are the
//! preprint's).
//!
//! Each cell holds a state `A` in `[0, 1]`. A step (§2.1.3, equations 7–9):
//!
//! ```text
//! U = K * A                      the potential: A convolved with the kernel K
//! A <- clip(A + Δt G(U), 0, 1)   G maps U to a growth in [−1, 1]; Δt = 1/T
//! ```
//!
//! The kernel (§2.1.4) is a ring, or `B` concentric rings, of radius `R` cells: the shell
//! `K_S(r) = β[⌊B r⌋] K_C(B r mod 1)` at distance `r` (in units of `R`), divided by its sum
//! so that `K * A` stays in `[0, 1]` (equation 12). The core function `K_C` and the growth
//! mapping `G` come in several shapes (equations 10 and 13), [`Core`] and [`Growth`] here.
//!
//! # Which core functions the creatures use
//!
//! The paper ran with exponential core functions (§2.2.4). The creatures here come from
//! Chan's `Python/animals.json`, whose entries all say `kn: 1, gn: 1`. In his
//! `Python/LeniaND.py`, which reads that file, those numbers index the tables
//! `kernel_core` and `growth_func` as `[kn - 1]` (lines 272–281 and 317), and entry 0 of
//! both is the *polynomial* form, although the program's labels (lines 2308–2309) call
//! setting 1 "Exponential". Run here, the two disagree in a way that settles it: with the
//! polynomial forms every creature below holds its shape for 1,000 steps; with the
//! exponential ones, Synorbium and Vagorbium evaporate within 200. So the creatures use the
//! polynomial forms, as the program that saved them computes them. The paper's figure 7
//! measurements, made with the exponential forms, are checked with those (the slow test
//! `orbium_mass_and_speed_match_figure_7`).
//!
//! # Life as a special case
//!
//! §2.1.6: the Game of Life is discrete Lenia with a "staircase" kernel core
//! `1[1/4, 3/4](r) + ½ 1[0, 1/4)(r)` and a step growth mapping, `μ = 0.35, σ = 0.07`.
//! The paper says `R = T = 1`, but with a round neighbourhood of radius 1 the diagonal
//! neighbours (√2 away) are left out. Chan's data has `R = 2` for its Life patterns
//! (`animals.json`, the "Conway's Game of Life" section), and that works: the eight
//! neighbours are at 1/2 and √2/2, weight 1, the cell itself at 0, weight ½, and distance 2
//! is outside, so `U = (neighbours + ½ A) / 8.5`, and `|U − 0.35| ≤ 0.07` is exactly B3/S23.
//! The test `life_is_lenia_with_a_staircase_kernel` checks it generation by generation.
//!
//! # Edges and the neighbourhood
//!
//! The world wraps round (a torus), as the Fourier transform makes it in Chan's programs. A
//! cell at distance exactly `R` is outside the kernel (`r < 1`, as in `LeniaND.py`'s
//! `kernel_shell`); the paper writes the ball as closed, but its core functions are 0 there
//! anyway, except the staircase's.
//!
//! # Speed
//!
//! The potential is computed with Fourier transforms ([`crate::fft`]), as in Chan's
//! programs: two transforms of the world a step, against 516 multiplications a cell
//! for a direct sum at radius 13. [`World::potential_direct`] is the direct sum,
//! kept as the reference the tests compare with. States are `f32`; Chan's GPU path uses
//! single precision too (`complex64` in `LeniaND.py`).
//!
//! # Licence of the creatures
//!
//! The creatures' patterns ([`CREATURES`]) are copied from `Python/animals.json` in
//! <https://github.com/Chakazul/Lenia> at commit adfc542939266de7f4bb7ebb552e8499701ee107,
//! which is under the MIT licence:
//!
//! > MIT License
//! >
//! > Copyright (c) 2018 Bert Chan
//! >
//! > Permission is hereby granted, free of charge, to any person obtaining a copy of this
//! > software and associated documentation files (the "Software"), to deal in the Software
//! > without restriction, including without limitation the rights to use, copy, modify,
//! > merge, publish, distribute, sublicense, and/or sell copies of the Software, and to
//! > permit persons to whom the Software is furnished to do so, subject to the following
//! > conditions:
//! >
//! > The above copyright notice and this permission notice shall be included in all copies
//! > or substantial portions of the Software.
//! >
//! > THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
//! > INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//! > PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
//! > HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF
//! > CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE
//! > OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use crate::fft::Fft;
use ca_core::Rng;

/// The kernel core `K_C(r)` for `r` in `[0, 1]` (equation 10, and 14 for Life's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Core {
    /// `exp(α − α / (4 r (1 − r)))`, `α = 4`. The paper's choice; only the test of its
    /// figure 7 uses it (see the module notes).
    #[cfg_attr(not(test), allow(dead_code))]
    Exponential,
    /// `(4 r (1 − r))^α`, `α = 4`.
    Polynomial,
    /// Life's: 1 on `[1/4, 3/4]`, ½ below 1/4.
    Staircase,
}

impl Core {
    pub fn at(self, r: f64) -> f64 {
        match self {
            Core::Exponential => {
                let q = 4.0 * r * (1.0 - r);
                if q <= 0.0 { 0.0 } else { (4.0 - 4.0 / q).exp() }
            }
            Core::Polynomial => (4.0 * r * (1.0 - r)).max(0.0).powi(4),
            Core::Staircase => {
                if (0.25..=0.75).contains(&r) {
                    1.0
                } else if (0.0..0.25).contains(&r) {
                    0.5
                } else {
                    0.0
                }
            }
        }
    }
}

/// The growth mapping `G(u; μ, σ)` (equation 13): 1 at `u = μ`, falling to −1 away from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Growth {
    /// `2 exp(−(u − μ)² / 2σ²) − 1`. As for [`Core::Exponential`].
    #[cfg_attr(not(test), allow(dead_code))]
    Exponential,
    /// `2 (1 − (u − μ)² / 9σ²)^α − 1` within `μ ± 3σ`, −1 outside; `α = 4`.
    Polynomial,
    /// 1 within `μ ± σ`, −1 outside (the paper's "rectangular").
    Step,
}

impl Growth {
    pub fn at(self, u: f32, mu: f32, sigma: f32) -> f32 {
        let d = u - mu;
        match self {
            Growth::Exponential => 2.0 * (-d * d / (2.0 * sigma * sigma)).exp() - 1.0,
            Growth::Polynomial => {
                let q = (1.0 - d * d / (9.0 * sigma * sigma)).max(0.0);
                let q2 = q * q;
                2.0 * q2 * q2 - 1.0
            }
            Growth::Step => {
                if d.abs() <= sigma {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }
}

/// Everything that decides how a world evolves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rule {
    /// The kernel's radius `R`, in cells.
    pub radius: usize,
    /// Steps per unit of time, `T`: each step adds `G / T`.
    pub time: u32,
    /// The growth centre μ and width σ.
    pub mu: f32,
    pub sigma: f32,
    /// The rings' peak heights β, innermost first.
    pub peaks: &'static [f64],
    pub core: Core,
    pub growth: Growth,
}

impl Rule {
    /// Whether `other` has the same kernel, so a world can switch to it without rebuilding.
    fn same_kernel(&self, other: &Rule) -> bool {
        self.radius == other.radius && self.peaks == other.peaks && self.core == other.core
    }

    /// The kernel shell `K_S` at distance `d` cells, before normalising: 0 from `R` out.
    pub fn shell(&self, d: f64) -> f64 {
        let r = d / self.radius as f64;
        if r >= 1.0 {
            return 0.0;
        }
        let rings = self.peaks.len();
        let br = rings as f64 * r;
        let ring = (br.floor() as usize).min(rings - 1);
        self.peaks[ring] * self.core.at(br.fract())
    }

    /// The kernel: every offset `(dx, dy)` with a nonzero weight, the weights summing to 1.
    pub fn kernel(&self) -> Vec<(isize, isize, f64)> {
        let r = self.radius as isize;
        let mut out = Vec::new();
        for dy in -r..=r {
            for dx in -r..=r {
                let w = self.shell(((dx * dx + dy * dy) as f64).sqrt());
                if w > 0.0 {
                    out.push((dx, dy, w));
                }
            }
        }
        let total: f64 = out.iter().map(|k| k.2).sum();
        for k in &mut out {
            k.2 /= total;
        }
        out
    }
}

/// A creature from Chan's catalogue: its rule and its starting pattern.
pub struct Creature {
    pub name: &'static str,
    /// What it does, for the panel.
    pub note: &'static str,
    pub rule: Rule,
    /// The pattern in Chan's run-length encoding (see [`decode`]).
    pub cells: &'static str,
}

const fn chans(radius: usize, mu: f32, sigma: f32, peaks: &'static [f64]) -> Rule {
    Rule {
        radius,
        time: 10,
        mu,
        sigma,
        peaks,
        core: Core::Polynomial,
        growth: Growth::Polynomial,
    }
}

/// The creatures offered, from `animals.json` (see the licence above). Each entry's name,
/// `R`, `T`, `b`, `m`, `s` and `cells` are copied unchanged; the codes are O2u, OG2g,
/// OV2u, O4s, S1s and 2S1v.
pub const CREATURES: [Creature; 6] = [
    Creature {
        name: "Orbium",
        note: "Orbium unicaudatus, Lenia's best-known creature: an orb with a tail that glides in a straight line.",
        rule: chans(13, 0.15, 0.015, &[1.0]),
        cells: concat!(
            "7.MD6.qL$6.pKqEqFURpApBRAqQ$5.VqTrSsBrOpXpWpTpWpUpCrQ$4.CQrQsTsWsApITNPpGqGvL$3.IpIp",
            "WrOsGsBqXpJ4.LsFrL$A.DpKpSpJpDqOqUqSqE5.ExD$qL.pBpTT2.qCrGrVrWqM5.sTpP$.pGpWpD3.qUsM",
            "tItQtJ6.tL$.uFqGH3.pXtOuR2vFsK5.sM$.tUqL4.GuNwAwVxBwNpC4.qXpA$2.uH5.vBxGyEyMyHtW4.qI",
            "pL$2.wV5.tIyG3yOxQqW2.FqHpJ$2.tUS4.rM2yOyJyOyHtVpPMpFqNV$2.HsR4.pUxAyOxLxDxEuVrMqBqG",
            "qKJ$3.sLpE3.pEuNxHwRwGvUuLsHrCqTpR$3.TrMS2.pFsLvDvPvEuPtNsGrGqIP$4.pRqRpNpFpTrNtGtVt",
            "StGsMrNqNpF$5.pMqKqLqRrIsCsLsIrTrFqJpHE$6.RpSqJqPqVqWqRqKpRXE$8.OpBpIpJpFTK!",
        ),
    },
    Creature {
        name: "Gyrorbium",
        note: "Gyrorbium gyrans, a relative of Orbium that steers as it goes, round and round.",
        rule: chans(13, 0.156, 0.0224, &[1.0]),
        cells: concat!(
            "10.EL2QLE$7.TpU2qHqCpXpUpNpFL$4.JrVtTuKuPuKtLrXqTqHqCpPpDG$3.qWtDqRpKqEsMuXvBtGrApXp",
            "UpSpIO$2.rQrN4.pAuAvRtTrIpUpIpKpFO$.pSsM6.tJwFuNsPsFrVpPpDL$.uFB6.tJ2yO2yLyOyDsKL$pD",
            "uC6.pFxW3yOwIwD2xPqH$rNtV5.EsMxCyIyOwXtJsMtJwFuX$sHuSV3.EpDvOwFxEwQsRqR2qHsFvWE$rQvJ",
            "sWpPQpKpSqCvEvBuCpD3.BpDtGrQ$pXuKvMuPtLsWsCrIuCtBrS6.qWrQ$EsKvEwXyBwLtVrVsCrDqH6.pXr",
            "G$.qHtVxJyOwQrQpNqJpPV6.qJqE$.JsUxMyOrX10.pFqRJ$2.rQxPwIpI9.pKqJT$2.qJxEuPpKB7.qCpP$",
            "2.EvOvMpPO5.TrGqH$3.sCyOqEpIOEBOqHqEsRtG$4.xMsMqJqCpXqJqRpIqOuCtBsF$5.xPrAqTqMpSE.rS",
            "sMrLqRqHV.TpS$6.vErDE2.VpPB$7.pIrNqHpKQ!",
        ),
    },
    Creature {
        name: "Vagorbium",
        note: "Vagorbium undulatus, a larger cousin (radius 20) that swerves one way, then the other: a zigzag.",
        rule: chans(20, 0.2, 0.031, &[1.0]),
        cells: concat!(
            "17.DWqD2qUqES$7.D2FDCBCHqGsTvBwQxL2xSxOwWvJsSpC$5.DJOQPNMPqWtQwHyAyHxLwAuSuGuTwBxLxX",
            "wNtAO$4.FQpB2pGpEpDqLsSvJxOyOxXvGrRR3.NrNvDxQxPuXqP$3.ESpJpSpXpWqCrMtOvRxWyOxWuOpV6.",
            "CpHtWxCxPvIrM$3.PpMqEqNqQqXsBtQvKxJ2yOvWqW8.GpXuNxFxBuWrJ$2.HpIqIrArIrPsJtLuQwGxWyOy",
            "HuHL9.XsPwExEwEtWqU$2.UqCrDrSsCsQtItXuWwIyDyOyDtK10.PsIvTwXwKuSsPqC$.GpMqUrTsJsUtItO",
            "tWuOwFyHyOyJtF10.LtIwMxAwDuUtFrHpJ$.PqErMsJsWtKtOtKtNuIwAyN2yOtVpF9.pRvKxRxCvPuFsUrO",
            "qIpCJ$.pBqRrXsStHtOtL2tEuDwE3yOvIrHpKK7.sKxWyOwUuStDrWrDqLpTpGR$DpLrCsHtAtJtMtGsXtDu",
            "HwR2yOyHwAsSqUpPO3.DqGpKvH2yOwCtMrSqQqEqApXpUpMJ$GpQrHsJsXtHtItBtAtMvByC2yOyAvHsNrFq",
            "BU3.qVrStD2yOyNvEsGqLpKpApFpQqCqApA$HpSrGsGsSsXtBsXtHuHwJ3yOxOuGrRqQpXQ2.qLqWsCwC2yO",
            "yIwTuVqXJ.JpEqAqFpI$HpPrArVsHsKsRtCtXvN4yOxCsGqJpTpOP.pRqAqPsAwW2yOyCwIvDuKqI2.PpUqD",
            "pJ$FpIqLrFrNrSsHtHuVxX4yOtGpQXSQRUpBpJpUrJxQ2yOxTwDvBuQtAW.HpMpUpC$CVpTqJqPqUrStNwA4",
            "yOuA6.BEISqExH2yOxPwDvEuUuBqW.BpCpHM$.KpApJpNpRrAtPyI3yOuHS10.WvQyOyNxNwHvLuTtQrO2.M",
            "N$.CJ2NMqAtQ3yOuVrMR11.pIsDvGvOvBuOuCtGrJ$2.A2CANtR2yOxQsQrHX10.CpPrJrWrXtE2tSsXqV$7",
            ".uIyLyOtLsKrIpJ10.LpUqWrErBrLsUtKsSqG$7.uCyAyMsUsIrMqCK9.TqAqP2qOqVrVsUsJpM$7.sNxExL",
            "sMsErTqSpHE7.KpJqEqJqGqEqNrHsBrMN$7.qBvUvJ3rVrJqKpEH5.IpApTqFqDpXpWqEqSrCqH$8.uBtJqP",
            "rBrNrQrHqMpNRKIKRpHpTqEqCpTpOpNpUqEqCR$8.rUrLOpNqHrCrMrLrAqKpWpRpSpWqGqEqCpRpLpHpG2p",
            "JS$8.pApW2.FpFqEqWrHrGrDqVqRqMqFpXpNpFXUSRI$9.P5.HpEpQ2qCpVpNpEVPJDC$19.B$17.CUS$16.",
            "OpOpSpXqD$17.GJLPS!",
        ),
    },
    Creature {
        name: "Synorbium",
        note: "Synorbium solidus: two orbs fused into one creature, the way two Orbium can fuse when they collide.",
        rule: chans(13, 0.122, 0.0106, &[1.0]),
        cells: concat!(
            "23.OTVM.qF$18.qHK.pCpDpLpIpApJpRXqN$18.qHsGpKqGqMX4.pLsO$18.OtUsLqLqHA5.RwN$19.uEvVr",
            "JRO6.LrS$17.KpHrFwGsV.pBpS6.uC$13.AIVpDpIpPpMsXuLqIqCsHqQ6.rU$12.GpBpKpAUNE2.tCuArUt",
            "OuApO5.rI$6.pV4.GpBpAE6.rWvGtUtDwBvOG4.pQpG$6.sDrJ2.IpEpA9.vRwOtWvBxVvF4.pEpR$4.AGrF",
            "uEsKWpKpH10.uJyHvEtKwSyOtK3.pFpQ$4.DqDqOuLvPpWpMI10.GyOwUsFtFxKxErPWVpTpI$3.QpIqQqAs",
            "DwDtSN12.vNyOtWsTuVwNtXqWqBqFR$2.VpKpAUWpAsEtWG12.pNxLwGuCuFuWtVrTqTpUC$.CpKN2.TEpJu",
            "CsV13.tIvVuWtXtLsPrNqLV$.OpC3.pDrCpXsPvCsL12.qLtOtTtAsErLqNpIC$VUT4.rVsHsJwEwOX11.pQ",
            "rOsCrPqVqFpGH$pLpBQ4.sGuRtNvFyOyD10.TpXqOqRqGpMTE$.uCU4.PvHwFuTwDyOvE7.BpApMpRpLpBOF",
            "$.qDpQ5.uPyKuSsPwByJrH5.RpIpANA$2.wF5.qUyOwAsGtKwMuXpQ2.LpEpAC$2.qQA5.xFxOtLtCuUvCsE",
            "qEpSpRW$3.uE5.tBxBvBtOtUtVsLrCqFV$3.MrG4.qMuRuVtStBsMrNqLpD$4.pXqKE.CpXsLtJsPrTrBqFp",
            "DB$5.pHqKpWpSqLrHrOrDqKpPTB$6.DpKpVqFqGqApMXJ$9.EHGC!",
        ),
    },
    Creature {
        name: "Scutium",
        note: "Scutium solidus, a shield-shaped glider: Lenia's version of SmoothLife's \"smooth glider\".",
        rule: chans(13, 0.29, 0.045, &[1.0]),
        cells: concat!(
            "5.pGQ$6.sUsDqRR$4.VpXrJwKvNtXrW$2.ApVrGrWsIwKyOyDwTuNO$2.qArOrIqIpPpTxH2yOyIvKqG$rVp",
            "IrNrJpH4.xP3yOvFqWA$sKvNsKqE5.pI4yOuHqPC$sNxBuMpD5.JuN3yOwXsUpN$sGxFyOT5.pCtIyF3yOuK",
            "qUH$rPwXyOxT5.qKtSxS3yOvIrQT$pQwK2yOtWXJXqHsGuWxW3yOvPsApC$.vQ3yOuVsLsGsXuMwOyK3yOvH",
            "rVpC$.rKyK3yOxEwBwCwXyE3yOxQuMrJU$.WvC5yOyM5yOwFtJqOL$2.rVwO9yOwWuMsApOC$2.pLtBwK6yO",
            "yJwQuTsQqLP$3.qHsWvEwUxTyCxRwWvRuGsNqSpA$3.HqBrUtHuGuPuLtVsXrSqIXB$4.DpDqEqW2rIqWqGp",
            "JN$6.ENTUPHA!",
        ),
    },
    Creature {
        name: "Serrated Scutium",
        note: "Scutium serratus vagus: a kernel of two rings (the inner one half height) gives it a toothed edge, and it wanders.",
        rule: chans(10, 0.27, 0.044, &[0.5, 1.0]),
        cells: concat!(
            "9.ANWSIA$9.MpRqSrAqKpJMB$6.DFOpPrItBtOtBrUqIpBH$5.pEpVqPrStKvI2wQvQuEsJqRpIL$4.PrSuK",
            "wSyI4yOxWwLuHsFqPpGF$4.qNxD9yOwWuDsDqMU$4.tM5yOyNxXxTyN2yOwVuFsFqCH$3.JwU3yOwLwDwBvK",
            "vIxJ3yOwJuBrQpG$3.uKwX2yOuDrOsBsHsIsGuFxEyKyOxQvLsXqIF$2.tDuXxIyOvW2.L.HqIsQvLxNyOyL",
            "wHtNrAR$FrLtNvOyDxS6.pMtDwHyH2yOwBtAqUU$UrJtRwFyOW6.pWtJxE2yOyLuDrFpUM$AqUtMvXrSG6.p",
            "KsTxC2yOwVrRpHMA$.qD.qBrFS6.SsLxG2yOvMqHH$3.VqRqFO5.qNwK3yOuEpQA$4.pOqKqBL3.sU5yOsPp",
            "OA$4.OpNqNqJ2pEtByOyNyM2yOvKrBpH$5.IpMqXrXxJyFxFwNwLuNsHqHTG$7.LsRvSvNvBuKI$8.sIsXtJ",
            "tDU$7.NpLqPrHqA$10.D!",
        ),
    },
];

/// The Game of Life as Lenia (§2.1.6, with `R = 2` as in Chan's data: see the module notes).
pub const LIFE: Rule = Rule {
    radius: 2,
    time: 1,
    mu: 0.35,
    sigma: 0.07,
    peaks: &[1.0],
    core: Core::Staircase,
    growth: Growth::Step,
};

/// Bill Gosper's glider gun, from the same file (code `~ggun`), in Life's 0/1 RLE.
pub const GLIDER_GUN: &str = concat!(
    "24bo$22bobo$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o$2o8bo3bob2o4bobo$10bo5bo7bo",
    "$11bo3bo$12b2o!",
);

/// A rectangle of states.
#[derive(Clone, Debug, PartialEq)]
pub struct Pattern {
    pub width: usize,
    pub height: usize,
    /// Row by row.
    pub cells: Vec<f32>,
}

/// Decode Chan's run-length encoding (`Board.rle2arr` and `ch2val` in `LeniaND.py`), in
/// two dimensions. Rows end with `$`, the pattern with `!`; a number before a symbol
/// repeats it (a number before `$` adds empty rows). A state is a symbol: `.` or `b` for
/// 0, `o` for 255 (Life's patterns), `A`..`X` for 1 to 24, and a letter `p`..`y` followed
/// by `A`..`X` for 25 to 255 (`(letter − p) × 24 + 25 + (second − A)`), all out of 255.
/// Short rows are padded with 0.
pub fn decode(rle: &str) -> Pattern {
    let mut rows: Vec<Vec<f32>> = vec![Vec::new()];
    let mut count = 0usize;
    let mut prefix: Option<char> = None;
    let take = |count: &mut usize| std::mem::take(count).max(1);
    for ch in rle.trim_end_matches('!').chars() {
        match ch {
            '0'..='9' => count = count * 10 + ch.to_digit(10).unwrap() as usize,
            'p'..='y' => prefix = Some(ch),
            '$' => {
                for _ in 0..take(&mut count) {
                    rows.push(Vec::new());
                }
            }
            _ => {
                let value = match (prefix.take(), ch) {
                    (None, '.' | 'b') => 0,
                    (None, 'o') => 255,
                    (None, c @ 'A'..='X') => c as u32 - 'A' as u32 + 1,
                    (Some(p), c @ 'A'..='X') => {
                        (p as u32 - 'p' as u32) * 24 + 25 + (c as u32 - 'A' as u32)
                    }
                    (p, c) => panic!("bad symbol {p:?}{c:?} in a pattern"),
                };
                let n = take(&mut count);
                rows.last_mut()
                    .unwrap()
                    .extend(std::iter::repeat_n(value as f32 / 255.0, n));
            }
        }
    }
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    let height = rows.len();
    let mut cells = Vec::with_capacity(width * height);
    for row in rows {
        let len = row.len();
        cells.extend(row);
        cells.extend(std::iter::repeat_n(0.0, width - len));
    }
    Pattern {
        width,
        height,
        cells,
    }
}

/// A square world of side a power of two, wrapping round its edges.
pub struct World {
    n: usize,
    cells: Vec<f32>,
    rule: Rule,
    fft: Fft,
    /// The transform of the kernel (real, since the kernel is symmetric), divided by `n²` to
    /// undo the inverse transform's scaling. In [`Fft::forward_2d`]'s transposed layout.
    spectrum: Vec<f32>,
    re: Vec<f32>,
    im: Vec<f32>,
    pub steps: u64,
}

impl World {
    pub fn new(n: usize, rule: Rule) -> Self {
        let mut w = Self {
            n,
            cells: vec![0.0; n * n],
            rule,
            fft: Fft::new(n),
            spectrum: Vec::new(),
            re: vec![0.0; n * n],
            im: vec![0.0; n * n],
            steps: 0,
        };
        w.build_spectrum();
        w
    }

    pub fn side(&self) -> usize {
        self.n
    }

    pub fn rule(&self) -> Rule {
        self.rule
    }

    /// Change the rule, carrying on from the current states.
    pub fn set_rule(&mut self, rule: Rule) {
        let rebuild = !self.rule.same_kernel(&rule);
        self.rule = rule;
        if rebuild {
            self.build_spectrum();
        }
    }

    fn build_spectrum(&mut self) {
        let n = self.n;
        assert!(2 * self.rule.radius < n, "kernel wider than the world");
        let (mut re, mut im) = (vec![0.0f32; n * n], vec![0.0f32; n * n]);
        for (dx, dy, w) in self.rule.kernel() {
            let (x, y) = (dx.rem_euclid(n as isize), dy.rem_euclid(n as isize));
            re[y as usize * n + x as usize] = w as f32;
        }
        self.fft.forward_2d(&mut re, &mut im);
        let scale = 1.0 / (n * n) as f32;
        self.spectrum = re.into_iter().map(|v| v * scale).collect();
    }

    pub fn cells(&self) -> &[f32] {
        &self.cells
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        self.cells[y * self.n + x]
    }

    pub fn set(&mut self, x: usize, y: usize, v: f32) {
        self.cells[y * self.n + x] = v;
    }

    pub fn clear(&mut self) {
        self.cells.fill(0.0);
        self.steps = 0;
    }

    /// The sum of all states: Chan's "mass", in cells.
    pub fn mass(&self) -> f64 {
        self.cells.iter().map(|&v| f64::from(v)).sum()
    }

    /// Clear the world and put `p` in the middle.
    pub fn place(&mut self, p: &Pattern) {
        self.clear();
        let (x0, y0) = ((self.n - p.width) / 2, (self.n - p.height) / 2);
        for y in 0..p.height {
            for x in 0..p.width {
                self.set(x0 + x, y0 + y, p.cells[y * p.width + x]);
            }
        }
    }

    /// Clear the world and scatter `patches` squares of random states, as Chan's
    /// `random_world` does (`LeniaND.py`): each `⌊0.9 R⌋` cells across, its states uniform
    /// in `[0, 0.9)`, at a uniformly random place (here wrapping round the edges).
    pub fn soup(&mut self, patches: usize, rng: &mut Rng) {
        self.clear();
        let n = self.n;
        let side = (self.rule.radius * 9 / 10).max(1);
        for _ in 0..patches {
            let (x0, y0) = (pick(rng, n), pick(rng, n));
            for y in 0..side {
                for x in 0..side {
                    let v = (rng.next_f64() * 0.9) as f32;
                    self.set((x0 + x) % n, (y0 + y) % n, v);
                }
            }
        }
    }

    /// Set every cell within `radius` of `(cx, cy)`, wrapping round the edges, to `value()`.
    pub fn paint_disc(
        &mut self,
        cx: usize,
        cy: usize,
        radius: f32,
        mut value: impl FnMut() -> f32,
    ) {
        let n = self.n as isize;
        let r = radius.ceil() as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                if ((dx * dx + dy * dy) as f32) <= radius * radius {
                    let x = (cx as isize + dx).rem_euclid(n) as usize;
                    let y = (cy as isize + dy).rem_euclid(n) as usize;
                    self.set(x, y, value());
                }
            }
        }
    }

    /// The potential `U = K * A`, by Fourier transform, into `self.re`.
    fn convolve(&mut self) {
        self.re.copy_from_slice(&self.cells);
        self.im.fill(0.0);
        self.fft.forward_2d(&mut self.re, &mut self.im);
        for ((re, im), &s) in self.re.iter_mut().zip(&mut self.im).zip(&self.spectrum) {
            *re *= s;
            *im *= s;
        }
        self.fft.inverse_2d(&mut self.re, &mut self.im);
    }

    /// The potential at every cell, by Fourier transform: what [`World::step`] uses.
    #[cfg(test)]
    pub fn potential(&mut self) -> Vec<f32> {
        self.convolve();
        self.re.clone()
    }

    /// The potential at every cell, summed directly over the kernel in f64: slow, the
    /// reference for the tests.
    #[cfg(test)]
    pub fn potential_direct(&self) -> Vec<f32> {
        let n = self.n as isize;
        let kernel = self.rule.kernel();
        let mut out = vec![0.0; self.cells.len()];
        for y in 0..n {
            for x in 0..n {
                let mut sum = 0.0f64;
                for &(dx, dy, w) in &kernel {
                    let i = (y + dy).rem_euclid(n) * n + (x + dx).rem_euclid(n);
                    sum += w * f64::from(self.cells[i as usize]);
                }
                out[(y * n + x) as usize] = sum as f32;
            }
        }
        out
    }

    /// One step: `A <- clip(A + G(K * A) / T, 0, 1)`.
    pub fn step(&mut self) {
        self.convolve();
        let Rule {
            mu,
            sigma,
            growth,
            time,
            ..
        } = self.rule;
        let dt = 1.0 / time as f32;
        // One loop per shape, so the shape isn't matched once a cell.
        let (cells, potential) = (&mut self.cells, &self.re);
        match growth {
            Growth::Exponential => grow(cells, potential, dt, |u| {
                Growth::Exponential.at(u, mu, sigma)
            }),
            Growth::Polynomial => grow(cells, potential, dt, |u| {
                Growth::Polynomial.at(u, mu, sigma)
            }),
            Growth::Step => grow(cells, potential, dt, |u| Growth::Step.at(u, mu, sigma)),
        }
        self.steps += 1;
    }
}

/// `A <- clip(A + Δt g(U), 0, 1)` for every cell.
fn grow(cells: &mut [f32], potential: &[f32], dt: f32, g: impl Fn(f32) -> f32) {
    for (a, &u) in cells.iter_mut().zip(potential) {
        *a = (*a + dt * g(u)).clamp(0.0, 1.0);
    }
}

fn pick(rng: &mut Rng, n: usize) -> usize {
    (rng.next_u64() % n as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use ca_core::{Grid, moore_counts};

    const ORBIUM: usize = 0;

    fn random_world(n: usize, rule: Rule, seed: u64) -> World {
        let mut w = World::new(n, rule);
        let mut rng = Rng::new(seed);
        for v in &mut w.cells {
            *v = rng.next_f64() as f32;
        }
        w
    }

    /// Equation 12 normalises the kernel to sum 1, so `K * A` stays in `[0, 1]`. The
    /// spectrum's constant term is that sum too.
    #[test]
    fn the_kernel_sums_to_1() {
        for rule in CREATURES.iter().map(|c| c.rule).chain([LIFE]) {
            let total: f64 = rule.kernel().iter().map(|k| k.2).sum();
            assert!((total - 1.0).abs() < 1e-12);
            let w = World::new(64, rule);
            assert!((w.spectrum[0] * 64.0 * 64.0 - 1.0).abs() < 1e-5);
        }
    }

    /// The paper's requirements of a core (§2.1.4): 0 at both ends, 1 in the middle, and
    /// unimodal; the staircase is Life's (equation 14).
    #[test]
    fn core_functions_have_the_papers_shape() {
        for core in [Core::Exponential, Core::Polynomial] {
            assert_eq!(core.at(0.0), 0.0);
            assert_eq!(core.at(1.0), 0.0);
            assert!((core.at(0.5) - 1.0).abs() < 1e-12);
            for i in 0..50 {
                let r = i as f64 / 100.0;
                assert!(core.at(r) < core.at(r + 0.01), "{core:?} rises to 1/2");
                assert!((core.at(r) - core.at(1.0 - r)).abs() < 1e-12, "symmetric");
            }
        }
        // (4 · 1/4 · 3/4)^4 = (3/4)^4, and e^(4 − 4/(3/4)) = e^(−4/3).
        assert!((Core::Polynomial.at(0.25) - 0.316_406_25).abs() < 1e-12);
        assert!((Core::Exponential.at(0.25) - (-4.0f64 / 3.0).exp()).abs() < 1e-12);
        let s = Core::Staircase;
        assert_eq!(
            [s.at(0.0), s.at(0.2), s.at(0.25), s.at(0.75), s.at(0.8)],
            [0.5, 0.5, 1.0, 1.0, 0.0]
        );
    }

    /// Equation 13: G(μ) = 1 for every shape; −1 far away; the polynomial exactly −1 from
    /// 3σ out and the step from just past σ; the exponential at μ ± σ is 2/√e − 1.
    #[test]
    fn growth_has_the_papers_shape() {
        let (mu, s) = (0.15f32, 0.015f32);
        for g in [Growth::Exponential, Growth::Polynomial, Growth::Step] {
            assert_eq!(g.at(mu, mu, s), 1.0);
            assert!((g.at(0.9, mu, s) + 1.0).abs() < 1e-6);
            assert!((g.at(0.0, mu, s) + 1.0).abs() < 1e-6);
        }
        let e = Growth::Exponential.at(mu + s, mu, s);
        assert!((e - (2.0 * (-0.5f32).exp() - 1.0)).abs() < 1e-5);
        assert_eq!(Growth::Polynomial.at(mu + 3.01 * s, mu, s), -1.0);
        assert_eq!(Growth::Polynomial.at(mu - 3.01 * s, mu, s), -1.0);
        // At 1.5σ: 2 (1 − 1/4)^4 − 1 = 2 · 0.31640625 − 1.
        let p = Growth::Polynomial.at(mu + 1.5 * s, mu, s);
        assert!((p - (2.0 * 0.316_406_25 - 1.0)).abs() < 1e-4);
        assert_eq!(Growth::Step.at(mu + 0.99 * s, mu, s), 1.0);
        assert_eq!(Growth::Step.at(mu + 1.01 * s, mu, s), -1.0);
    }

    /// Equation 11: the shell copies the core into `B` equal rings, ring `i` scaled by
    /// `β[i]`. With β = (1/2, 1) and radius 10 (the serrated Scutium's kernel), the inner
    /// ring peaks at 2.5 cells with height 1/2, the outer at 7.5 with height 1, and the
    /// shell is 0 at the centre, between the rings (5 cells) and from the edge (10) out.
    #[test]
    fn the_shell_copies_the_core_into_rings() {
        let rule = CREATURES[5].rule;
        assert_eq!((rule.radius, rule.peaks), (10, &[0.5, 1.0][..]));
        let at = |d: f64| rule.shell(d);
        assert!((at(2.5) - 0.5).abs() < 1e-12);
        assert!((at(7.5) - 1.0).abs() < 1e-12);
        for d in [0.0, 5.0, 10.0, 12.0] {
            assert_eq!(at(d), 0.0, "at {d}");
        }
        // Halfway up each ring's rising side: β · K_C(1/4).
        assert!((at(1.25) - 0.5 * Core::Polynomial.at(0.25)).abs() < 1e-12);
        assert!((at(6.25) - Core::Polynomial.at(0.25)).abs() < 1e-12);
    }

    /// A step worked by hand. Radius 2, polynomial core: the four orthogonal neighbours are
    /// at r = 1/2, weight 1; the diagonals at r = √2/2, weight (2√2 − 2)^4 = 0.470996;
    /// the cell itself and everything 2 or more away weigh 0. The sum is 5.883983, so a
    /// single 1 gives its orthogonal neighbours U = 0.169953 and its diagonals 0.080046.
    /// With μ = 0.16, σ = 0.01 (9σ² = 0.0009), T = 10:
    /// - orthogonal: (0.169953 − 0.16)² / 0.0009 = 0.110069, G = 2 · 0.889931^4 − 1 =
    ///   0.254436, so A = 0.0254436;
    /// - diagonal: 0.08 below μ, past 3σ, G = −1, A stays 0;
    /// - the cell itself: U = 0, G = −1, A = 1 − 0.1 = 0.9.
    #[test]
    fn a_first_step_by_hand() {
        const ONE: &[f64] = &[1.0];
        let rule = Rule {
            radius: 2,
            time: 10,
            mu: 0.16,
            sigma: 0.01,
            peaks: ONE,
            core: Core::Polynomial,
            growth: Growth::Polynomial,
        };
        let mut w = World::new(8, rule);
        w.set(3, 4, 1.0);
        w.step();
        for y in 0..8 {
            for x in 0..8 {
                let expected = match (x as i32 - 3, y as i32 - 4) {
                    (0, 0) => 0.9,
                    (0, 1 | -1) | (1 | -1, 0) => 0.025_443_6,
                    _ => 0.0,
                };
                assert!(
                    (w.get(x, y) - expected).abs() < 1e-5,
                    "({x}, {y}): {} not {expected}",
                    w.get(x, y)
                );
            }
        }
        assert_eq!(w.steps, 1);
    }

    /// The Fourier transform gives the same potential as summing over the kernel, for every
    /// creature's kernel (one, two rings; radius 10 to 20) and Life's.
    #[test]
    fn fourier_potential_matches_the_direct_sum() {
        for (i, rule) in CREATURES.iter().map(|c| c.rule).chain([LIFE]).enumerate() {
            let mut w = random_world(64, rule, i as u64);
            let direct = w.potential_direct();
            let fourier = w.potential();
            for (a, b) in direct.iter().zip(&fourier) {
                assert!((a - b).abs() < 2e-6, "rule {i}: {a} vs {b}");
            }
        }
    }

    /// §2.1.6: Life is Lenia with the staircase core and step growth. From a random start,
    /// every generation matches B3/S23 computed by counting neighbours, cell for cell.
    #[test]
    fn life_is_lenia_with_a_staircase_kernel() {
        let n = 64;
        let mut rng = Rng::new(7);
        let mut w = World::new(n, LIFE);
        let mut life = Grid::new(n, n, false);
        for y in 0..n {
            for x in 0..n {
                let alive = rng.chance(0.35);
                life.set(x, y, alive);
                w.set(x, y, if alive { 1.0 } else { 0.0 });
            }
        }
        let mut counts = Vec::new();
        for generation in 1..=100 {
            moore_counts(&life, true, |c| c, &mut counts);
            let mut next = life.clone();
            for (i, c) in next.cells_mut().iter_mut().enumerate() {
                *c = matches!((*c, counts[i]), (_, 3) | (true, 2));
            }
            life = next;
            w.step();
            for (i, &c) in life.cells().iter().enumerate() {
                assert_eq!(
                    w.cells[i],
                    if c { 1.0 } else { 0.0 },
                    "generation {generation}"
                );
            }
        }
        assert!(life.cells().iter().any(|&c| c), "the test world died out");
    }

    /// The decoder against the start of Orbium's pattern, read by hand: its first row is
    /// `7.MD6.qL$`, seven zeros, M = 13, D = 4, six zeros, qL = 24 + 25 + 11 = 60. Its
    /// size and the sum of its states (19,600 / 255) are from decoding the file with a
    /// separate Python transcription of Chan's `rle2arr`.
    #[test]
    fn decodes_chans_patterns() {
        let p = decode(CREATURES[ORBIUM].cells);
        assert_eq!((p.width, p.height), (20, 20));
        let row: Vec<u32> = p.cells[..20]
            .iter()
            .map(|v| (v * 255.0).round() as u32)
            .collect();
        let mut first = vec![0; 20];
        (first[7], first[8], first[15]) = (13, 4, 60);
        assert_eq!(row, first);
        let total: f32 = p.cells.iter().map(|v| (v * 255.0).round()).sum();
        assert_eq!(total, 19_600.0);
        let sizes: Vec<_> = CREATURES
            .iter()
            .map(|c| {
                let p = decode(c.cells);
                (p.width, p.height)
            })
            .collect();
        assert_eq!(
            sizes,
            [(20, 20), (21, 23), (39, 32), (34, 28), (18, 20), (22, 22)]
        );
        // Gosper's gun is 36 cells, 36 x 9.
        let gun = decode(GLIDER_GUN);
        assert_eq!((gun.width, gun.height), (36, 9));
        assert_eq!(gun.cells.iter().filter(|&&v| v == 1.0).count(), 36);
    }

    /// The centre of mass on a torus, as the mean angle of the states round each axis.
    fn centroid(w: &World) -> (f64, f64) {
        let n = w.side();
        let mut sums = [(0.0f64, 0.0f64); 2];
        for y in 0..n {
            for x in 0..n {
                let a = f64::from(w.get(x, y));
                for (s, c) in sums.iter_mut().zip([x, y]) {
                    let t = std::f64::consts::TAU * c as f64 / n as f64;
                    s.0 += a * t.cos();
                    s.1 += a * t.sin();
                }
            }
        }
        let at = |(c, s): (f64, f64)| {
            s.atan2(c).rem_euclid(std::f64::consts::TAU) * n as f64 / std::f64::consts::TAU
        };
        (at(sums[0]), at(sums[1]))
    }

    /// Run a creature on a `n`-cell world: its mass (in cells) at every step after a
    /// settling period, and its mean speed over them, in cells a step.
    fn run(rule: Rule, cells: &str, n: usize, settle: usize, steps: usize) -> (Vec<f64>, f64) {
        let mut w = World::new(n, rule);
        w.place(&decode(cells));
        for _ in 0..settle {
            w.step();
        }
        let start = centroid(&w);
        let mut last = start;
        let mut travelled = 0.0;
        let mut masses = Vec::new();
        for _ in 0..steps {
            w.step();
            let c = centroid(&w);
            let wrap = |d: f64| (d + n as f64 / 2.0).rem_euclid(n as f64) - n as f64 / 2.0;
            travelled += wrap(c.0 - last.0).hypot(wrap(c.1 - last.1));
            last = c;
            masses.push(w.mass());
        }
        (masses, travelled / steps as f64)
    }

    /// Chan's figure 7(a): Orbium with the exponential core functions, μ = 0.15,
    /// σ = 0.016, T = 10, measured over 300 steps at R from 9 to 55 (the caption), shows
    /// mass about 0.44 and linear speed about 0.48, both flat in R (read off the chart). In
    /// his units (§2.4.2) lengths are in kernel radii and times in units of T steps, so mass
    /// is the sum of states / R² and speed is cells a step × T / R.
    #[test]
    #[ignore]
    fn orbium_mass_and_speed_match_figure_7() {
        let orbium = &CREATURES[ORBIUM];
        let rule = Rule {
            sigma: 0.016,
            core: Core::Exponential,
            growth: Growth::Exponential,
            ..orbium.rule
        };
        let (masses, speed) = run(rule, orbium.cells, 128, 200, 300);
        let r2 = (rule.radius * rule.radius) as f64;
        let mass = masses.iter().sum::<f64>() / masses.len() as f64 / r2;
        let speed = speed * f64::from(rule.time) / rule.radius as f64;
        println!("mass {mass:.4}, speed {speed:.4}");
        assert!((mass - 0.44).abs() < 0.02, "mass {mass}");
        assert!((speed - 0.48).abs() < 0.02, "speed {speed}");
    }

    /// Every creature, with its own rule from Chan's file, neither evaporates nor explodes
    /// in 1,000 steps after settling, and each moves. The serrated Scutium pulses (its mass
    /// swings between about 97 and 160 cells), so the bound is loose; the others hold
    /// theirs within about 12%.
    #[test]
    #[ignore]
    fn every_creature_lives() {
        for c in &CREATURES {
            let (masses, speed) = run(c.rule, c.cells, 128, 200, 1_000);
            let (lo, hi) = masses
                .iter()
                .fold((f64::MAX, f64::MIN), |(lo, hi), &m| (lo.min(m), hi.max(m)));
            println!(
                "{}: mass {lo:.1} to {hi:.1}, {speed:.3} cells a step",
                c.name
            );
            assert!(lo > 0.0 && hi / lo < 2.0, "{}: mass {lo} to {hi}", c.name);
            assert!(speed > 0.05, "{} stood still", c.name);
        }
    }

    /// Orbium's mass holds to within 2% over 1,000 steps (it is about 71 cells).
    #[test]
    #[ignore]
    fn orbium_keeps_its_mass() {
        let o = &CREATURES[ORBIUM];
        let (masses, _) = run(o.rule, o.cells, 128, 200, 1_000);
        let mean = masses.iter().sum::<f64>() / masses.len() as f64;
        assert!(masses.iter().all(|m| (m / mean - 1.0).abs() < 0.02));
    }

    /// The cost of a step natively (`--ignored --nocapture`): the Fourier step at each world
    /// size, against the direct sum, for the panel's speed budget.
    #[test]
    #[ignore]
    fn timing_steps() {
        for c in [&CREATURES[ORBIUM], &CREATURES[2]] {
            for n in [128, 256, 512] {
                let mut w = World::new(n, c.rule);
                w.place(&decode(c.cells));
                let steps = 40;
                let t = std::time::Instant::now();
                for _ in 0..steps {
                    w.step();
                }
                let fourier = t.elapsed().as_secs_f64() * 1e3 / steps as f64;
                let t = std::time::Instant::now();
                std::hint::black_box(w.potential_direct());
                let direct = t.elapsed().as_secs_f64() * 1e3;
                println!(
                    "R {} at {n} x {n}: Fourier step {fourier:.2} ms, direct potential {direct:.1} ms",
                    c.rule.radius
                );
            }
        }
    }
}
