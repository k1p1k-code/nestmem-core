# Nextmem Core
A library that enables structured cache storage with a time-to-live (TTL) feature.

## Examples
``` rust
let nestmem = NestMem::new();
let group = Group::new_root();
let dash_object= nestmem.root.new_group("auth".to_string(), group).unwrap();
let object=dash_object.value();
let group = match object {
    Object::Group(g) => g,
    Object::Item(_i) => panic!("impossible")
};
let ttl = SystemTtl::new(TimeTtl::Seconds(3));
let item = Item::new("my_secret_string".to_string(), Some(ttl));
group.new_item("1234_kid".to_string(), item).unwrap();
let obj = group.get_object("kid_41125".to_string()).unwrap()

match obj{
  Object::Item(_item) => println!("Item")
  Object::Group(_group) => println!("Group")
}

println!(obj.get_value())
```

**The library is still in the development phase and is not ready for full-scale use.**

---
__This library is being developed for the creation of nestmem. Nestmem is a non-relational database management system featuring structured data storage in RAM.__
