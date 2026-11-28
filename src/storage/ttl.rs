use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemTtl {
    #[serde(skip, default = "Instant::now")]
    pub create_at: Instant,
    pub time: TimeTtl,
}

impl SystemTtl {
    pub fn new(time: TimeTtl) -> Self {
        Self {
            create_at: Instant::now(),
            time,
        }
    }

    pub fn is_expire(&self) -> bool {
        self.create_at.elapsed() >= self.time.to_duration()
    }

    pub fn remaining_secs(&self) -> u64 {
        self.time
            .to_duration()
            .saturating_sub(self.create_at.elapsed())
            .as_secs()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TimeTtl {
    Minutes(u64),
    Seconds(u64),
}

impl TimeTtl {
    pub fn to_duration(&self) -> Duration {
        match self {
            TimeTtl::Minutes(t) => Duration::from_mins(*t),
            TimeTtl::Seconds(t) => Duration::from_secs(*t),
        }
    }
}
