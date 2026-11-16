use crate::egui;

/// Run / pause / step / restart controls shared by every demo.
///
/// Each frame, call [`Playback::ui`] to draw the buttons, then [`Playback::take_restart`]
/// and [`Playback::advance`] to learn what to do. What a step and the speed count is up to
/// the demo.
pub struct Playback {
    pub running: bool,
    /// How much to advance per frame while running, counted in `unit`.
    pub speed: usize,
    pub max_speed: usize,
    /// What `speed` counts, shown beside its slider.
    pub unit: &'static str,
    step_requested: bool,
    restart_requested: bool,
}

impl Playback {
    pub fn new(speed: usize, max_speed: usize, unit: &'static str) -> Self {
        Self {
            running: false,
            speed,
            max_speed,
            unit,
            step_requested: false,
            restart_requested: false,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let label = if self.running { "Pause" } else { "Run" };
            if ui.button(label).clicked() {
                self.running = !self.running;
            }
            if ui
                .add_enabled(!self.running, egui::Button::new("Step"))
                .clicked()
            {
                self.step_requested = true;
            }
            if ui.button("Restart").clicked() {
                self.restart_requested = true;
            }
        });
        // The unit goes on its own line: beside the slider it made the row wider than the
        // fixed-width controls panel.
        ui.label(format!("Speed, in {}", self.unit));
        ui.add(egui::Slider::new(&mut self.speed, 1..=self.max_speed).logarithmic(true));
    }

    /// Whether Restart was clicked since the last call. Running or paused stays as it was.
    pub fn take_restart(&mut self) -> bool {
        std::mem::take(&mut self.restart_requested)
    }

    /// What to do this frame. A Step click is reported once, then forgotten.
    ///
    /// One value rather than separate queries, so a demo can't ask "was that a step?" after
    /// the step has already been consumed.
    pub fn advance(&mut self) -> Advance {
        if self.running {
            Advance::Run(self.speed)
        } else if std::mem::take(&mut self.step_requested) {
            Advance::Step
        } else {
            Advance::Idle
        }
    }
}

/// What a demo should do this frame, from [`Playback::advance`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Advance {
    /// Paused, and Step was not clicked.
    Idle,
    /// Paused, and Step was clicked: advance by one step.
    Step,
    /// Running: advance by this much, counted in the playback's `unit`.
    Run(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A Step click must advance exactly once. If it were not consumed, a single click would
    /// keep stepping every frame, making Step behave like Run.
    #[test]
    fn step_is_reported_once() {
        let mut p = Playback::new(10, 100, "steps");
        assert_eq!(p.advance(), Advance::Idle);
        p.step_requested = true;
        assert_eq!(p.advance(), Advance::Step);
        assert_eq!(p.advance(), Advance::Idle);
    }

    /// While running, a demo advances by the speed slider's value each frame; the demo's
    /// per-frame work budget comes from here.
    #[test]
    fn running_advances_by_speed() {
        let mut p = Playback::new(10, 100, "steps");
        p.running = true;
        assert_eq!(p.advance(), Advance::Run(10));
    }

    /// Restart is a one-shot request that leaves Run/Pause alone, so a demo that is running
    /// keeps running on its fresh grid (David's drip workflow), and a restart is not repeated.
    #[test]
    fn restart_is_reported_once_and_keeps_running() {
        let mut p = Playback::new(10, 100, "steps");
        p.running = true;
        p.restart_requested = true;
        assert!(p.take_restart());
        assert!(!p.take_restart());
        assert!(p.running);
    }
}
