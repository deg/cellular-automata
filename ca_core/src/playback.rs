use crate::egui;

/// Run / pause / single-step controls shared by every demo.
///
/// Each frame, call [`Playback::ui`] to draw the buttons, then [`Playback::steps_this_frame`]
/// to learn how much to advance. What a step and the speed count is up to the demo.
pub struct Playback {
    pub running: bool,
    /// How much to advance per frame while running, counted in `unit`.
    pub speed: usize,
    pub max_speed: usize,
    /// What `speed` counts, shown beside its slider.
    pub unit: &'static str,
    step_requested: bool,
}

impl Playback {
    pub fn new(speed: usize, max_speed: usize, unit: &'static str) -> Self {
        Self { running: false, speed, max_speed, unit, step_requested: false }
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
        });
        ui.add(
            egui::Slider::new(&mut self.speed, 1..=self.max_speed)
                .logarithmic(true)
                .text(self.unit),
        );
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
