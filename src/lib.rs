pub mod error;
pub mod storage;


use std::{
    sync::Arc,
    thread::{JoinHandle, sleep},
    time::Duration,
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
    pub root: Group,
    bg_worker_lazy: Option<JoinHandle<()>>,
}

impl NestMem {
    pub fn new() -> Arc<Self> {
        let mut mem = Self {
            root: Group::new_root(),
            bg_worker_lazy: None,
        };

        let arc_mem = Arc::new_cyclic(|me: &std::sync::Weak<Self>| {
            let worker_weak = me.clone();

            let handle = std::thread::spawn(move || {
                sleep(Duration::from_millis(10));
                loop {
                    println!("1");
                    if let Some(db) = worker_weak.upgrade() {
                        db.root.cleanup();
                    }
                    sleep(Duration::from_millis(100));
                }
            });

            mem.bg_worker_lazy = Some(handle);
            mem
        });

        arc_mem
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
        sleep(Duration::from_secs(5));
        match group.get_clone_object("1234_kid".to_string()) {
            Ok(e) => {
                println!("{:?}", e);
                panic!("TTl lazy no work")
            }
            Err(e) => assert_eq!(e, ErrorNM::NoFindObject, "I must not find her"),
        }
    }
}
