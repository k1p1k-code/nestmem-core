use std::{
    sync::{Arc, Barrier, atomic::Ordering},
    thread,
    time::Duration,
};

use crate::*;

#[test]
fn test_group_creation_and_permissions() {
    let nestmem = NestMem::new();
    assert_eq!(nestmem.root.to_object().unwrap_err(), ErrorNM::NoFindObject);
    assert!(nestmem.root.is_attached.load(Ordering::Acquire));

    let leaf_group = Object::group(
        "leaf".to_string(),
        false,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    )
    .unwrap();

    assert!(leaf_group.is_attached.load(Ordering::Acquire));
    assert_eq!(nestmem.root.len(), 1);

    let dup_res = Object::group(
        "leaf".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    );
    assert_eq!(dup_res.unwrap_err(), ErrorNM::GroupExists);

    let sub_res = Object::group(
        "sub".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        Arc::downgrade(&leaf_group),
    );
    assert_eq!(sub_res.unwrap_err(), ErrorNM::NoPermissionCreateGroup);

    let item_res = Object::item(
        "key1".to_string(),
        "val1".to_string(),
        None,
        Arc::downgrade(&leaf_group),
    );
    assert!(item_res.is_ok());
}

#[test]
fn test_items_lookup_and_filtering() {
    let nestmem = NestMem::new();

    let group = Object::group(
        "data".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    )
    .unwrap();

    Object::item(
        "i1".to_string(),
        "v1".to_string(),
        None,
        Arc::downgrade(&group),
    )
    .unwrap();
    Object::item(
        "i2".to_string(),
        "v2".to_string(),
        None,
        Arc::downgrade(&group),
    )
    .unwrap();
    Object::group(
        "sub_g".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        Arc::downgrade(&group),
    )
    .unwrap();

    let dup_item = Object::item(
        "i1".to_string(),
        "v3".to_string(),
        None,
        Arc::downgrade(&group),
    );
    assert_eq!(dup_item.unwrap_err(), ErrorNM::ItemExists);

    assert_eq!(group.len(), 3);
    assert_eq!(group.ls().len(), 3);

    let items_only = group.withinme_items();
    assert_eq!(items_only.len(), 2);

    assert!(group.get_clone_object("i1").is_ok());
    assert_eq!(
        group.get_clone_object("missing").unwrap_err(),
        ErrorNM::NoFindObject
    );
}

#[test]
fn test_contains_group_recursive() {
    let nestmem = NestMem::new();

    let g1 = Object::group(
        "g1".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    )
    .unwrap();

    let g2 = Object::group(
        "g2".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        Arc::downgrade(&g1),
    )
    .unwrap();

    let g3 = Object::group(
        "g3".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        Arc::downgrade(&g2),
    )
    .unwrap();

    let separate_group = Object::group(
        "other".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    )
    .unwrap();

    assert!(g1.contains_group(Arc::clone(&g1)));
    assert!(g1.contains_group(Arc::clone(&g2)));
    assert!(g1.contains_group(Arc::clone(&g3)));

    assert!(!g1.contains_group(separate_group));
}

#[test]
fn test_delete_obj_and_to_object_lifecycle() {
    let nestmem = NestMem::new();

    let group = Object::group(
        "users".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    )
    .unwrap();

    let item = Object::item(
        "admin".to_string(),
        "secret".to_string(),
        None,
        Arc::downgrade(&group),
    )
    .unwrap();

    let group_obj = group.to_object().unwrap();
    let stored_obj = nestmem.root.get_clone_object("users").unwrap();
    assert!(Arc::ptr_eq(&group_obj, &stored_obj));
    assert!(item.to_object().is_ok());

    group.delete_obj("admin").unwrap();
    assert_eq!(
        group.delete_obj("admin").unwrap_err(),
        ErrorNM::NoFindObject
    );
    assert_eq!(item.to_object().unwrap_err(), ErrorNM::NoFindObject);

    nestmem.root.delete_obj("users").unwrap();
    assert!(!group.is_attached.load(Ordering::Acquire));

    drop(group_obj);
    drop(stored_obj);
    assert_eq!(group.to_object().unwrap_err(), ErrorNM::NoFindObject);
}

#[test]
fn test_cleanup_ttl() {
    let nestmem = NestMem::new();

    let group = Object::group(
        "cache".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        nestmem.get_root_weak(),
    )
    .unwrap();

    let short_ttl = SystemTtl::new(TimeTtl::Seconds(1));
    let long_ttl = SystemTtl::new(TimeTtl::Seconds(60));

    let expired_item = Object::item(
        "temp".to_string(),
        "123".to_string(),
        Some(short_ttl),
        Arc::downgrade(&group),
    )
    .unwrap();

    let alive_item = Object::item(
        "persist".to_string(),
        "456".to_string(),
        Some(long_ttl),
        Arc::downgrade(&group),
    )
    .unwrap();

    thread::sleep(Duration::from_millis(1200));

    nestmem.root.cleanup();

    assert_eq!(group.len(), 1);
    assert!(group.get_clone_object("persist").is_ok());
    assert_eq!(
        group.get_clone_object("temp").unwrap_err(),
        ErrorNM::NoFindObject
    );

    assert_eq!(expired_item.to_object().unwrap_err(), ErrorNM::NoFindObject);
    assert!(alive_item.to_object().is_ok());
}

#[test]
fn test_concurrent_access_and_uniqueness() {
    let nestmem = Arc::new(NestMem::new());
    let threads = 16;
    let barrier = Arc::new(Barrier::new(threads));
    let mut handles = Vec::with_capacity(threads);

    for i in 0..threads {
        let nm = Arc::clone(&nestmem);
        let b = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            b.wait();
            let race_group = Object::group(
                "race_group".to_string(),
                true,
                GroupTtlPolicy::LazyItemTtl,
                nm.get_root_weak(),
            );

            let my_group = Object::group(
                format!("worker_{i}"),
                true,
                GroupTtlPolicy::LazyItemTtl,
                nm.get_root_weak(),
            )
            .unwrap();

            Object::item(
                "data".to_string(),
                format!("val_{i}"),
                None,
                Arc::downgrade(&my_group),
            )
            .unwrap();

            race_group.is_ok()
        }));
    }

    nestmem.root.cleanup();

    let winners: usize = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .filter(|&won| won)
        .count();

    assert_eq!(winners, 1);
    assert_eq!(nestmem.root.len(), threads + 1);
}
