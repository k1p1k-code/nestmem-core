pub mod error;
pub mod storage;

use std::{
    sync::{Arc, Weak},
    thread::{self, JoinHandle},
    time::Duration,
};

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
    _bg_worker_lazy: JoinHandle<()>,
}

impl NestMem {
    pub fn new() -> Self {
        let root = Group::new_root();
        let weak_root = Arc::downgrade(&root);

        let bg_worker_lazy = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100));
                match weak_root.upgrade() {
                    Some(root) => root.cleanup(),
                    None => break,
                }
            }
        });

        Self {
            root,
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unnecessary_to_owned)]
    use super::*;
    #[test]
    fn check_defualt() {
        let nestmem = NestMem::new();
        let _group = Object::group(
            "Auth".to_string(),
            false,
            GroupTtlPolicy::LazyItemTtl,
            nestmem.get_root_weak(),
        )
        .unwrap();
        assert_eq!(1, nestmem.root.len())
    }

    #[test]
    fn check_work_ttl_lazy() {
        let nestmem = NestMem::new();
        let group = Object::group(
            "auth".to_string(),
            false,
            GroupTtlPolicy::LazyItemTtl,
            nestmem.get_root_weak(),
        )
        .unwrap();

        let ttl = SystemTtl::new(storage::ttl::TimeTtl::Seconds(3));
        let _item_obj = Object::item(
            "1234_kid".to_string(),
            "my_secret_string".to_string(),
            Some(ttl),
            Arc::downgrade(&group),
        )
        .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(5));

        match group.get_clone_object("1234_kid") {
            Ok(e) => {
                println!("{:?}", e);
                panic!("TTL lazy no work");
            }
            Err(e) => assert_eq!(e, ErrorNM::NoFindObject, "I must not find her"),
        }
    }
}
