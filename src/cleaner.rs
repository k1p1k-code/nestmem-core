use std::sync::Weak;
use crate::Group;


pub trait CleanerTrait{
    fn new(weak_root: Weak<Group>) -> Self;
}

pub struct NoCleaner;
impl CleanerTrait for NoCleaner {
    fn new(_: Weak<Group>) -> Self {
        Self
    }
}

#[cfg(feature = "sync-cleaner")]
pub struct CleanerThread{
    _bg_worker: std::thread::JoinHandle<()>,
}

#[cfg(feature = "sync-cleaner")]
impl CleanerTrait for CleanerThread{
    fn new(weak_root: Weak<Group>) -> Self {
        let bg_worker = std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                match weak_root.upgrade() {
                    Some(root) => root.cleanup(),
                    None => break,
                }
            }
        });
        Self {
            _bg_worker: bg_worker
         }
    }
}

#[cfg(feature = "tokio-cleaner")]
pub struct CleanerTokio{
    _bg_worker: tokio::task::JoinHandle<()>
}

#[cfg(feature = "tokio-cleaner")]
impl CleanerTrait for CleanerTokio{
    fn new(weak_root: Weak<Group>) -> Self {
        let bg_worker = tokio::runtime::Handle::try_current().ok().map(|handle| {
            handle.spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
                loop {
                    interval.tick().await;
                    match weak_root.upgrade() {
                        Some(root) => root.cleanup(),
                        None => break,
                    }
                }
            })
        }).unwrap();
        Self {
            _bg_worker: bg_worker
         }
    }
}

