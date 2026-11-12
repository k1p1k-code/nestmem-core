use std::sync::{Arc, Weak};

use crate::error::ErrorNM;
use crate::storage::group_ttl::GroupTtlPolicy;
use crate::storage::item::Item;
use crate::storage::object::{Object, TypeObject};
use dashmap::DashMap;

#[derive(Debug)]
pub struct Group {
    self_weak: Weak<Group>,
    pub objects: DashMap<String, Object>,
    ttl_policy: GroupTtlPolicy,
    allowed_groups: bool,
}
impl Group {
    fn create_obj(&self, obj: TypeObject) -> Object {
        Object {
            parent: self.self_weak.clone(),
            typ: obj,
        }
    }

    pub fn new_root() -> Arc<Self> {
        Arc::new_cyclic(|weak_self| Group {
            self_weak: weak_self.clone(),
            objects: DashMap::new(),
            ttl_policy: GroupTtlPolicy::LazyItemTtl,
            allowed_groups: true,
        })
    }

    pub fn new(ttl_policy: GroupTtlPolicy, allowed_groups: bool) -> Arc<Self> {
        Arc::new_cyclic(|weak_self| Group {
            self_weak: weak_self.clone(),
            objects: DashMap::new(),
            ttl_policy,
            allowed_groups,
        })
    }

    pub fn get_clone_object<K: AsRef<str>>(&self, key: K) -> Result<Object, ErrorNM> {
        self.objects
            .get(key.as_ref())
            .map(|r| r.value().clone())
            .ok_or(ErrorNM::NoFindObject)
    }

    pub fn withinme_items(&self) -> Vec<Arc<Item>> {
        self.objects
            .iter()
            .filter_map(|r| match &r.value().typ {
                TypeObject::Item(e) => Some(e.clone()),
                TypeObject::Group(_) => None,
            })
            .collect()
    }

    fn new_object(&self, key: String, object: Object) {
        self.objects.insert(key.clone(), object);
    }

    pub fn new_item(&self, key: String, item: Item) {
        self.new_object(key, self.create_obj(TypeObject::Item(Arc::new(item))))
    }

    pub fn new_group(&self, name: String, group: Arc<Group>) -> Result<Arc<Group>, ErrorNM> {
        if !self.allowed_groups {
            return Err(ErrorNM::NoPermissionCreateGroup);
        }
        let obj = self.create_obj(TypeObject::Group(group.clone()));
        self.new_object(name, obj);
        Ok(group)
    }

    pub fn delete_obj(&self, key: String) -> Result<(), ErrorNM> {
        if !self.objects.remove(&key).is_some() {
            return Err(ErrorNM::NoFindObject);
        }
        Ok(())
    }

    pub fn cleanup(&self) {
        let mut group_store = vec![];
        self.objects.retain(|_key, object| {
            if let TypeObject::Item(item) = &object.typ {
                match &self.ttl_policy {
                    GroupTtlPolicy::LazyItemTtl => {
                        return item.ttl.as_ref().is_none_or(|ttl| !ttl.is_expire());
                    }

                    GroupTtlPolicy::PeriodicScheduled(period_ttl) => {
                        return !period_ttl.is_expire();
                    }
                }
            }
            if let TypeObject::Group(group) = &object.typ {
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
