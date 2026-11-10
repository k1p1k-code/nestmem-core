pub mod storage;
pub mod error;
pub use error::ErrorNM;


use std::{sync::Arc, thread::{JoinHandle, sleep}, time::Duration};

use storage::group::Group;
use storage::object::Object;

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
                    if let Some(db) = worker_weak.upgrade() {
                        db.root.cleanup();
                    } else {
                        break;
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
mod tests{
    use crate::storage::{group, item::{self, Item}, object, ttl::SystemTtl};

use super::*;
    #[test]
    fn check_work_ttl_lazy(){
        let nestmem = NestMem::new();
        let group = Group::new_root();
        let dash_object= nestmem.root.new_group("auth".to_string(), group).unwrap();
        let object=dash_object.value();
        let group = match object {
            Object::Group(g) => g,
            Object::Item(_i) => panic!("impossible")
        };
        let ttl = SystemTtl::new(storage::ttl::TimeTtl::Seconds(3));
        let item = Item::new("my_secret_string".to_string(), Some(ttl));
        group.new_item("1234_kid".to_string(), item).unwrap();
        sleep(Duration::from_secs(4));
        match group.get_object("1234_kid".to_string()){
            Ok(e) => {println!("{:?}", e); panic!("TTl lazy no work")},
            Err(e) => assert_eq!(e, ErrorNM::NoFindObject, "I must not find her"),


        }
    }
}