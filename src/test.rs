use super::*;

#[test]
#[cfg(not(feature = "tokio-cleaner"))]
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
#[cfg(not(feature = "tokio-cleaner"))]
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


#[tokio::test]
#[cfg(feature = "tokio-cleaner")]
async fn check_default_async() {
    let nestmem = NestMem::new();
    let _group = Object::group(
        "Auth".to_string(),
        false,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    )
    .unwrap();
    assert_eq!(1, nestmem.root.len());
}

#[tokio::test]
#[cfg(feature = "tokio-cleaner")]
async fn check_work_ttl_async() {
    let nestmem = NestMem::new();
    let group = Object::group(
        "auth".to_string(),
        false,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    )
    .unwrap();

    let ttl = SystemTtl::new(storage::ttl::TimeTtl::Seconds(2));
    let _item_obj = Object::item(
        "1234_kid".to_string(),
        "my_secret_string".to_string(),
        Some(ttl),
        Arc::downgrade(&group),
    )
    .unwrap();

    assert!(group.get_clone_object("1234_kid").is_ok());

    tokio::time::sleep(Duration::from_secs(3)).await;

    match group.get_clone_object("1234_kid") {
        Ok(e) => panic!("Async TTL did not clean up object: {:?}", e),
        Err(e) => assert_eq!(e, ErrorNM::NoFindObject, "Object must be expired"),
    }
}

#[tokio::test]
#[cfg(feature = "tokio-cleaner")]
async fn check_worker_stops_after_drop() {
    let weak_root = {
        let nestmem = NestMem::new();
        nestmem.get_root_weak()
    }; 

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(weak_root.upgrade().is_none());
}
