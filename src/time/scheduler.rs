//! Fixed-step physics accumulator decoupled from render rate.

/// Schedules RK4 steps from real elapsed time and time warp.
#[derive(Debug)]
pub struct PhysicsScheduler {
    accumulator_s: f64,
    pub max_steps_per_frame: u32,
    pub steps_this_frame: u32,
    pub overloaded: bool,
}

impl PhysicsScheduler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            accumulator_s: 0.0,
            max_steps_per_frame: 8,
            steps_this_frame: 0,
            overloaded: false,
        }
    }

    pub fn accumulate(&mut self, real_dt_s: f64, time_warp: f64) {
        self.accumulator_s += real_dt_s * time_warp;
        self.steps_this_frame = 0;
        self.overloaded = false;
    }

    #[must_use]
    pub fn should_step(&self, dt_s: f64) -> bool {
        self.accumulator_s >= dt_s && self.steps_this_frame < self.max_steps_per_frame
    }

    pub fn consume_step(&mut self, dt_s: f64) {
        self.accumulator_s -= dt_s;
        self.steps_this_frame += 1;
        if self.accumulator_s >= dt_s && self.steps_this_frame >= self.max_steps_per_frame {
            self.overloaded = true;
            self.accumulator_s = dt_s * f64::from(self.max_steps_per_frame - 1);
        }
    }
}

impl Default for PhysicsScheduler {
    fn default() -> Self {
        Self::new()
    }
}
