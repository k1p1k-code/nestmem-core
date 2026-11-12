pub mod error;
pub mod storage;


use std::{
    sync::Arc, thread::{self, JoinHandle}, time::Duration,
};

pub use error::ErrorNM;
pub use storage::{
    group::Group,
    item::Item,
    object::Object,
    ttl::SystemTtl,
    group_ttl::GroupTtlPolicy
};


pub struct NestMem {
    pub root: Arc<Group>,
    bg_worker_lazy: JoinHandle<()>, 
}

impl NestMem {
    pub fn new() -> Self { // Больше не Arc<Self>!
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
            bg_worker_lazy,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_work_ttl_lazy() {
        let nestmem = NestMem::new();
        let group = Group::new_root();
        let group = nestmem.root.new_group("auth".to_string(), group).unwrap();
        let ttl = SystemTtl::new(storage::ttl::TimeTtl::Seconds(3));
        let item = Item::new("my_secret_string".to_string(), Some(ttl));
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
