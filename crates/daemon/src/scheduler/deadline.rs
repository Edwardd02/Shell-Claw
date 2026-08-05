use std::time::Instant;

pub struct DeadlineTracker {
    deadline: Instant,
}

impl DeadlineTracker {
    pub fn new(deadline_ms: u64) -> Self {
        Self {
            deadline: Instant::now() + std::time::Duration::from_millis(deadline_ms),
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }

    pub fn remaining_ms(&self) -> u64 {
        let now = Instant::now();
        if now >= self.deadline {
            0
        } else {
            self.deadline
                .duration_since(now)
                .as_millis()
                .min(u64::MAX as u128) as u64
        }
    }
}
