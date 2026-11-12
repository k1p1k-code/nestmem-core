use serde::Serialize;
use std::time::{self, Duration, Instant};

#[derive(Debug, Serialize, Clone)]
pub struct SystemTtl {
    #[serde(skip)]
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
}

#[derive(Debug, Serialize, Clone)]
pub enum TimeTtl {
    Minutes(u64),
    Seconds(u64),
}

impl TimeTtl {
    fn to_duration(&self) -> Duration {
        match self {
            TimeTtl::Minutes(t) => Duration::from_mins(*t),
            TimeTtl::Seconds(t) => Duration::from_secs(*t),
        }
    }
}
