# Nextmem Core
A library that enables structured cache storage with a time-to-live (TTL) feature.

## Examples
``` rust
let nestmem = NestMem::new();
let group = Group::new_root();
let group = nestmem.root.new_group("auth".to_string(), group).unwrap();
let ttl = SystemTtl::new(storage::ttl::TimeTtl::Seconds(3));
let item = Item::new("my_secret_string".to_string(), Some(ttl));
group.new_item("1234_kid".to_string(), item);
let obj = group.get_clone_object("1234_kid".to_string()).unwrap()
println!("{obj}");
```

**The library is still in the development phase and is not ready for full-scale use.**

---
__This library is being developed for the creation of nestmem. Nestmem is a non-relational database management system featuring structured data storage in RAM.__
