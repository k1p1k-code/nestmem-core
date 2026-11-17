use super::{CleanerTrait, impl_drop_cleaner};

use crate::Group;
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

pub struct CleanerThread {
    bg_worker: Option<std::thread::JoinHandle<()>>,
    flag_run: Arc<AtomicBool>,
    weak_root: Weak<Group>,
}

impl CleanerThread {
    fn spawn_worker(
        weak_root: Weak<Group>,
        flag_run: Arc<AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            while flag_run.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));

                if !flag_run.load(Ordering::Relaxed) {
                    break;
                }

                match weak_root.upgrade() {
                    Some(root) => root.cleanup(),
                    None => break,
                }
            }
        })
    }
}

impl CleanerTrait for CleanerThread {
    fn new(weak_root: Weak<Group>) -> Self {
        let flag_run = Arc::new(AtomicBool::new(true));
        let bg_worker = Self::spawn_worker(weak_root.clone(), Arc::clone(&flag_run));
        Self {
            bg_worker: Some(bg_worker),
            flag_run,
            weak_root,
        }
    }

    fn run(&mut self) {
        if self.bg_worker.is_some() {
            return;
        }

        if self.weak_root.strong_count() == 0 {
            return;
        }

        self.flag_run.store(true, Ordering::Relaxed);
        let bg_worker = Self::spawn_worker(self.weak_root.clone(), self.flag_run.clone());
        self.bg_worker = Some(bg_worker);
    }

    fn stop(&mut self) {
        self.flag_run.store(false, Ordering::Relaxed);
        if let Some(handle) = self.bg_worker.take() {
            let _ = handle.join();
        }
    }
}

impl_drop_cleaner!(CleanerThread);
