use crate::egui;

/// Run / pause / step / restart controls shared by every demo.
///
/// Each frame, call [`Playback::ui`] to draw the buttons, then [`Playback::take_restart`]
/// and [`Playback::steps_this_frame`] to learn what to do. What a step and the speed count
/// is up to the demo.
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
        Self { running: false, speed, max_speed, unit, step_requested: false, restart_requested: false }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let label = if self.running { "Pause" } else { "Run" };
            if ui.button(label).clicked() {
                self.running = !self.running;
            }
            if ui.add_enabled(!self.running, egui::Button::new("Step")).clicked() {
                self.step_requested = true;
            }
            if ui.button("Restart").clicked() {
                self.restart_requested = true;
            }
        });
        ui.add(
            egui::Slider::new(&mut self.speed, 1..=self.max_speed)
                .logarithmic(true)
                .text(self.unit),
        );
    }

    /// Whether Restart was clicked since the last call. Running or paused stays as it was.
    pub fn take_restart(&mut self) -> bool {
        std::mem::take(&mut self.restart_requested)
    }

    /// Whether this frame's steps came from a Step click rather than from running.
    pub fn is_single_step(&self) -> bool {
        !self.running && self.step_requested
    }

    /// Steps to take this frame: `speed` while running, 1 after a Step click, else 0.
    pub fn steps_this_frame(&mut self) -> usize {
        if self.running {
            self.speed
        } else if std::mem::take(&mut self.step_requested) {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_gives_one_step_once() {
        let mut p = Playback::new(10, 100, "steps");
        assert_eq!(p.steps_this_frame(), 0);
        p.step_requested = true;
        assert!(p.is_single_step());
        assert_eq!(p.steps_this_frame(), 1);
        assert!(!p.is_single_step());
        assert_eq!(p.steps_this_frame(), 0);
    }

    #[test]
    fn running_gives_speed_steps() {
        let mut p = Playback::new(10, 100, "steps");
        p.running = true;
        assert!(!p.is_single_step());
        assert_eq!(p.steps_this_frame(), 10);
    }

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
