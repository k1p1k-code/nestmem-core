pub mod error;
pub mod storage;

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

pub use error::ErrorNM;
pub use storage::{
    group::Group, group_ttl::GroupTtlPolicy, item::Item, object::Object, ttl::SystemTtl,
};

pub struct NestMem {
    pub root: Arc<Group>,
    _bg_worker_lazy: JoinHandle<()>,
}

impl NestMem {
    pub fn new() -> Self {
        let root = Arc::new(Group::new_root());
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
        let group = Group::new_root();
        let group = nestmem.root.new_group("auth".to_string(), group).unwrap();
        let item = Item::new("my_secret_string".to_string(), None);
        group.new_item("1234_kid".to_string(), item);
        let obj = group.get_clone_object("1234_kid".to_string()).unwrap();
        println!("{:?}", obj);
    }

    #[test]
    fn check_work_ttl_lazy() {
        let nestmem = NestMem::new();
        let group = Group::new_root();
        let group = nestmem.root.new_group("auth".to_string(), group).unwrap();
        let ttl = SystemTtl::new(storage::ttl::TimeTtl::Seconds(3));
        let item = Item::new("my_secret_string".to_string(), Some(ttl));
        #[allow(clippy::unnecessary_to_owned)]
        group.new_item("1234_kid".to_string(), item);
        thread::sleep(Duration::from_secs(5));
        match group.get_clone_object("1234_kid".to_string()) {
            Ok(e) => {
                println!("{:?}", e);
                panic!("TTl lazy no work")
            }
            Err(e) => assert_eq!(e, ErrorNM::NoFindObject, "I must not find her"),
        }
    }
}
