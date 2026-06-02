//! Per-body trail ring buffers.

use std::collections::VecDeque;

use glam::DVec3;

/// Ring buffer of past positions for trail rendering.
#[derive(Debug, Clone)]
pub struct Trail {
    points: VecDeque<DVec3>,
    capacity: usize,
}

impl Trail {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            points: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Record position when `tick % sample_every == 0`.
    pub fn maybe_push(&mut self, position: DVec3, tick: u64, sample_every: u64) {
        if sample_every == 0 || tick % sample_every != 0 {
            return;
        }
        if self.points.len() >= self.capacity {
            self.points.pop_front();
        }
        self.points.push_back(position);
    }

    pub fn points(&self) -> impl ExactSizeIterator<Item = &DVec3> {
        self.points.iter()
    }
}
