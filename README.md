# Nestmem Core
A library that enables structured cache storage with a time-to-live (TTL) feature.

## Examples
``` rust
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
```
feature storage-walker
``` rust
use crate::{GroupTtlPolicy, Object};
let root = Group::new_root();
let shop = Object::group(
    "shop".to_string(),
    true,
    GroupTtlPolicy::LazyItemTtl,
    Arc::downgrade(&root),
)
.unwrap();
let products = Object::group(
    "products".to_string(),
    true,
    GroupTtlPolicy::LazyItemTtl,
    Arc::downgrade(&shop),
)
.unwrap();
let mut shop_walker = StorageWalker::from_group(shop);
shop_walker.cd_obj(products.clone()).unwrap();
shop_walker.up().unwrap();
let mut products_walker = StorageWalker::from_group(products.clone());
let _orders = Object::group(
    "orders".to_string(),
    true,
    GroupTtlPolicy::LazyItemTtl,
    Arc::downgrade(&products),
)
.unwrap();
products_walker.cd_str("orders").unwrap();
```
**The library is still in the development phase and is not ready for full-scale use.**

---
__This library is being developed for the creation of nestmem. Nestmem is a non-relational database management system featuring structured data storage in RAM.__
