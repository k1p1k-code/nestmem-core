pub mod error;
pub mod storage;

use std::sync::{Arc, Weak};
#[cfg(any(feature = "sync-cleaner", feature = "tokio-cleaner"))]
use std::time::Duration;

pub use error::ErrorNM;
pub use storage::{
    group::Group,
    group_ttl::GroupTtlPolicy,
    item::Item,
    object::{Object, ToObject, TypeObject},
    ttl::{SystemTtl, TimeTtl},
};


pub struct NestMem {
    pub root: Arc<Group>,
    #[cfg(all(feature = "sync-cleaner", not(feature = "tokio-cleaner")))]
    _bg_worker_lazy: std::thread::JoinHandle<()>,
    #[cfg(feature = "tokio-cleaner")]
    _bg_worker_lazy: Option<tokio::task::JoinHandle<()>>,
}

impl NestMem {
    pub fn new() -> Self {
        let root = Group::new_root();
        #[cfg(any(feature = "sync-cleaner", feature = "tokio-cleaner"))]
        let weak_root = Arc::downgrade(&root);

        #[cfg(all(feature = "sync-cleaner", not(feature = "tokio-cleaner")))]
        let bg_worker_lazy = std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(100));
                match weak_root.upgrade() {
                    Some(root) => root.cleanup(),
                    None => break,
                }
            }
        });

        #[cfg(feature = "tokio-cleaner")]
        let bg_worker_lazy = tokio::runtime::Handle::try_current().ok().map(|handle| {
            handle.spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                loop {
                    interval.tick().await;
                    match weak_root.upgrade() {
                        Some(root) => root.cleanup(),
                        None => break,
                    }
                }
            })
        });

        Self {
            root,
            #[cfg(any(feature = "sync-cleaner", feature = "tokio-cleaner"))]
            _bg_worker_lazy: bg_worker_lazy,
        }
    }

    pub fn get_root_weak(&self) -> Weak<Group> {
        Arc::downgrade(&self.root)
    }
}

impl Default for NestMem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "storage-walker")]
mod storagewalker;
#[cfg(feature = "storage-walker")]
pub use storagewalker::{error::ErrorSW, StorageWalker};

#[cfg(test)]
mod test;
