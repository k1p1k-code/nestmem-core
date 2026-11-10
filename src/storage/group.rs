use dashmap::DashMap;
use crate::storage::object::Object;
use crate::storage::item::Item;
use crate::storage::group_ttl::GroupTtlPolicy;

#[derive(Debug)]
pub struct Group{
    name: String,
    objects: DashMap<String, Object>,
    ttl_policy: GroupTtlPolicy,
    allowed_groups: bool
}
impl Group {
    pub fn new_root() -> Self{
        Group { 
            name: "root".to_string(),
            objects: DashMap::new(),
            ttl_policy: GroupTtlPolicy::ItemTtl,
            allowed_groups: true }
    }

    pub fn new(name: String, ttl_policy: Option<GroupTtlPolicy>, allowed_groups: bool) -> Self{
        let ttl_policy = match ttl_policy {
            Some(e) => e,
            None => GroupTtlPolicy::ItemTtl
        };
        
        Group {
            name,
            objects: DashMap::new(),
            ttl_policy,
            allowed_groups: allowed_groups }
    }

    pub fn get_name(&self) -> String{
        self.name.clone()
    }

    pub fn withinme_items(&self) -> Vec<Item> {
        self.objects
            .iter()
            .filter_map(|r| match r.value() {
                Object::Item(e) => Some(e.clone()),
                Object::Group(_) => None,
            })
            .collect()

    }

    pub fn new_object(&mut self, key: String, object: Object){
        self.objects.insert(key, object);
    } 

    pub fn new_item(&mut self, key: String, item: Item){
        self.new_object(key, Object::Item(item));
    }

    pub fn new_group(&mut self, key: String, group: Group) -> Result<(), &'static str>{
        if !self.allowed_groups{
            return Err("allowed_groups: false")
        }
        self.new_object(key, Object::Group(group));
        Ok(())
    }

}