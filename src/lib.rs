pub mod error;
pub mod storage;
mod cleaner;


use std::sync::{Arc, Weak};
pub use error::ErrorNM;
pub use storage::{
    group::Group,
    group_ttl::GroupTtlPolicy,
    item::Item,
    object::{Object, ToObject, TypeObject},
    ttl::{SystemTtl, TimeTtl},
};
pub use cleaner::*;

pub struct NestMem<C: CleanerTrait>{
    pub root: Arc<Group>,
    provide_cleaner: C
}

impl <C: CleanerTrait>NestMem<C> {
    pub fn new() -> Self {
        let root = Group::new_root();
        let provide_cleaner = C::new(Arc::downgrade(&root));

        Self {
            root,
            provide_cleaner
        }
    }

    pub fn get_root_weak(&self) -> Weak<Group> {
        Arc::downgrade(&self.root)
    }
}

impl <C: CleanerTrait>Default for NestMem<C> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "storage-walker")]
mod storagewalker;
#[cfg(feature = "storage-walker")]
pub use storagewalker::{StorageWalker, error::ErrorSW};

#[cfg(test)]
mod test;
