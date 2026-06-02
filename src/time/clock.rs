//! Real-time clock, pause, and time-warp controls.

use std::time::Instant;

/// Tracks wall-clock timing and simulation pacing.
#[derive(Debug)]
pub struct SimulationClock {
    last_tick: Instant,
    pub paused: bool,
    /// Multiplier on real elapsed time fed into the physics accumulator.
    pub time_warp: f64,
}

impl SimulationClock {
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            paused: false,
            time_warp: 1.0,
        }
    }

    /// Elapsed real time since the last frame (seconds), zero if paused.
    #[must_use]
    pub fn elapsed_real_s(&mut self) -> f64 {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick);
        self.last_tick = now;
        if self.paused {
            return 0.0;
        }
        dt.as_secs_f64()
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn increase_warp(&mut self) {
        self.time_warp = (self.time_warp * 1.5).min(256.0);
    }

    pub fn decrease_warp(&mut self) {
        self.time_warp = (self.time_warp / 1.5).max(0.1);
    }
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self::new()
    }
}
