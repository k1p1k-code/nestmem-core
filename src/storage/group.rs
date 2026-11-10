use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use crate::error::ErrorNM;
use crate::storage::object::Object;
use crate::storage::item::Item;
use crate::storage::group_ttl::GroupTtlPolicy;


pub type ResultObjWork<'a> = Result<Ref<'a, String, Object>, ErrorNM>;

#[derive(Debug)]
pub struct Group{
    objects: DashMap<String, Object>,
    ttl_policy: GroupTtlPolicy,
    allowed_groups: bool,
}
impl Group {
    pub fn new_root() -> Self{
        Group { 
            objects: DashMap::new(),
            ttl_policy: GroupTtlPolicy::LazyItemTtl,
            allowed_groups: true 
        }
    }

    pub fn new(ttl_policy: Option<GroupTtlPolicy>, allowed_groups: bool) -> Self{
        let ttl_policy = match ttl_policy {
            Some(e) => e,
            None => GroupTtlPolicy::LazyItemTtl
        };
        
        Group {
            objects: DashMap::new(),
            ttl_policy,
            allowed_groups: allowed_groups }
    }


    pub fn get_object<K: AsRef<str>>(&self, key: K) -> Result<Ref<'_, String, Object>, ErrorNM>{
        self.objects.get(key.as_ref())
            .ok_or_else(||{ErrorNM::NoFindObject})
        
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

    fn new_object(&self, key: String, object: Object) -> ResultObjWork<'_>{
        self.objects.insert(key.clone(), object);
        self.get_object(key)
    } 

    pub fn new_item(&self, key: String, item: Item) -> ResultObjWork<'_>{
        self.new_object(key, Object::Item(item))
    }

    pub fn new_group(&self, name: String, group: Group) -> ResultObjWork<'_>{
        if !self.allowed_groups{
            return Err(ErrorNM::NoPermissionCreateGroup)
        }
        self.new_object(name, Object::Group(group))

    }

    pub fn cleanup(&self){
        self.objects.retain(|_key, object|{
            if let Object::Item(item) = object{
                match &self.ttl_policy {
                    GroupTtlPolicy::LazyItemTtl => {
                        if let Some(ttl) = &item.ttl{
                            if ttl.is_expire(){
                                return false
                            }
                        }
                    }

                    GroupTtlPolicy::PeriodicScheduled(period_ttl) => {
                        if period_ttl.is_expire(){
                            return false;
                        }
                    }
                }
            }
            
            else if let Object::Group(group) = object{
                group.cleanup();
            }
            true
        });
        
    }

}