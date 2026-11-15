/// Turns a rate in steps per second into whole steps per frame.
///
/// Frames come at an uneven rate, and many automata are best watched at a few steps per
/// second, well under one per frame. Carrying the fraction from frame to frame keeps the
/// average rate right at any speed.
#[derive(Default)]
pub struct Pace {
    carry: f64,
}

impl Pace {
    /// Steps to take in a frame that lasted `dt` seconds, at `per_second` steps per second,
    /// never more than `max`. Time beyond `max` is dropped, not saved up, so a slow frame
    /// does not cause a burst of catch-up steps afterwards.
    pub fn steps(&mut self, per_second: f64, dt: f64, max: usize) -> usize {
        self.carry += per_second * dt.max(0.0);
        // The tolerance absorbs rounding: sixty frames of 1/60 s at 10 per second must
        // make 10 steps, not 9.999... floored to 9.
        let whole = (self.carry + 1e-9).floor();
        self.carry -= whole;
        (whole as usize).min(max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// At 10 steps per second and 60 frames per second, a second of frames must give exactly
    /// 10 steps: slow speeds are the reason this type exists.
    #[test]
    fn slow_rates_accumulate_across_frames() {
        let mut p = Pace::default();
        let total: usize = (0..60).map(|_| p.steps(10.0, 1.0 / 60.0, 100)).sum();
        assert_eq!(total, 10);
    }

    /// A frame that took a long time (a stall, a hidden tab) must not unleash thousands of
    /// steps at once and freeze the next frame too.
    #[test]
    fn a_long_frame_is_capped() {
        let mut p = Pace::default();
        assert_eq!(p.steps(1000.0, 5.0, 50), 50);
        assert_eq!(
            p.steps(1000.0, 0.0, 50),
            0,
            "the excess must not carry over"
        );
    }
}
