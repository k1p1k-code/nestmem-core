use std::sync::Arc;

use crate::error::ErrorNM;
use crate::storage::group_ttl::GroupTtlPolicy;
use crate::storage::item::Item;
use crate::storage::object::Object;
use dashmap::DashMap;

// pub type ResultObjWork<'a> = Result<Ref<'a, String, Object>, ErrorNM>;

#[derive(Debug, Clone)]
pub struct Group {
    objects: DashMap<String, Object>,
    ttl_policy: GroupTtlPolicy,
    allowed_groups: bool,
}
impl Group {
    pub fn new_root() -> Self {
        Group {
            objects: DashMap::new(),
            ttl_policy: GroupTtlPolicy::LazyItemTtl,
            allowed_groups: true,
        }
    }

    pub fn new(ttl_policy: Option<GroupTtlPolicy>, allowed_groups: bool) -> Self {
        let ttl_policy = match ttl_policy {
            Some(e) => e,
            None => GroupTtlPolicy::LazyItemTtl,
        };

        Group {
            objects: DashMap::new(),
            ttl_policy,
            allowed_groups,
        }
    }

    pub fn get_clone_object<K: AsRef<str>>(&self, key: K) -> Result<Object, ErrorNM> {
        self.objects
            .get(key.as_ref())
            .map(|r| r.value().clone())
            .ok_or(ErrorNM::NoFindObject)
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

    fn new_object(&self, key: String, object: Object) {
        self.objects.insert(key.clone(), object);
    }

    pub fn new_item(&self, key: String, item: Item) {
        self.new_object(key, Object::Item(item))
    }

    pub fn new_group(&self, name: String, group: Group) -> Result<Arc<Group>, ErrorNM> {
        if !self.allowed_groups {
            return Err(ErrorNM::NoPermissionCreateGroup);
        }
        let arc_group = Arc::new(group);
        self.new_object(name, Object::Group(arc_group.clone()));
        Ok(arc_group)
    }

    pub fn cleanup(&self) {
        let mut group_store = vec![];
        self.objects.retain(|_key, object| {
            if let Object::Item(item) = object {
                match &self.ttl_policy {
                    GroupTtlPolicy::LazyItemTtl => {
                        return item.ttl.as_ref().is_some_and(|ttl| !ttl.is_expire());
                    }

                    GroupTtlPolicy::PeriodicScheduled(period_ttl) => {
                        return !period_ttl.is_expire();
                    }
                }
            }
            if let Object::Group(group) = object {
                group_store.push(Arc::clone(group));
                return true;
            }
            true
        });
        for i in group_store {
            i.cleanup();
        }
    }
}
