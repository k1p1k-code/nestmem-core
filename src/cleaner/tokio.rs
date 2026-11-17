use super::{CleanerTrait, impl_drop_cleaner};

use crate::Group;
use std::sync::Weak;

pub struct CleanerTokio {
    bg_worker: Option<tokio::task::JoinHandle<()>>,
    weak_root: Weak<Group>,
}

impl CleanerTokio {
    fn spawn_worker(weak_root: Weak<Group>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
            loop {
                interval.tick().await;
                match weak_root.upgrade() {
                    Some(root) => root.cleanup(),
                    None => break,
                }
            }
        })
    }
}

impl CleanerTrait for CleanerTokio {
    fn new(weak_root: Weak<Group>) -> Self {
        let bg_worker = Self::spawn_worker(weak_root.clone());
        Self {
            bg_worker: Some(bg_worker),
            weak_root,
        }
    }

    fn run(&mut self) {
        if let Some(handle) = &self.bg_worker
            && !handle.is_finished()
        {
            return;
        }

        if self.weak_root.strong_count() == 0 {
            return;
        }

        self.bg_worker = Some(Self::spawn_worker(self.weak_root.clone()));
    }

    fn stop(&mut self) {
        if let Some(handle) = self.bg_worker.take() {
            handle.abort();
        }
    }
}

impl_drop_cleaner!(CleanerTokio);
