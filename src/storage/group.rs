use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use crate::error::ErrorNM;
use crate::storage::group_ttl::GroupTtlPolicy;
use crate::storage::item::Item;
use crate::storage::object::{Object, ToObject, TypeObject};
use dashmap::{DashMap, Entry};

#[derive(Debug)]
pub struct Group {
    self_weak: Weak<Group>,
    object: Option<Weak<Object>>,
    pub(crate) is_attached: AtomicBool,
    pub(crate) objects: DashMap<String, Arc<Object>>,
    ttl_policy: GroupTtlPolicy,
    allowed_groups: bool,
}

impl Group {
    pub fn contains_group(&self, group: &Arc<Group>) -> bool {
        if std::ptr::eq(self.self_weak.as_ptr(), Arc::as_ptr(group)) {
            return true;
        }
        self.objects.iter().any(|entry| {
            if let TypeObject::Group(child_group) = &entry.value().typ {
                Arc::ptr_eq(child_group, group) || child_group.contains_group(group)
            } else {
                false
            }
        })
    }

    pub fn new_root() -> Arc<Self> {
        Arc::new_cyclic(|weak_self| Group {
            self_weak: weak_self.clone(),
            object: None,
            is_attached: AtomicBool::new(true),
            objects: DashMap::new(),
            ttl_policy: GroupTtlPolicy::LazyItemTtl,
            allowed_groups: true,
        })
    }

    pub fn new(
        object: Weak<Object>,
        ttl_policy: GroupTtlPolicy,
        allowed_groups: bool,
    ) -> Arc<Self> {
        Arc::new_cyclic(|weak_self| Group {
            self_weak: weak_self.clone(),
            object: Some(object),
            is_attached: AtomicBool::new(false),
            objects: DashMap::new(),
            ttl_policy,
            allowed_groups,
        })
    }

    pub fn get_clone_object<K: AsRef<str>>(&self, key: K) -> Result<Arc<Object>, ErrorNM> {
        self.objects
            .get(key.as_ref())
            .map(|r| r.value().clone())
            .ok_or(ErrorNM::NoFindObject)
    }

    pub fn ls(&self) -> Vec<Arc<Object>> {
        self.objects
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
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

    pub fn new_item(&self, key: String, obj: Arc<Object>) -> Result<Arc<Item>, ErrorNM> {
        // self.new_object(key, obj);
        let TypeObject::Item(item) = &obj.typ else {
            return Err(ErrorNM::InvalidObject);
        };
        let item = Arc::clone(item);
        match self.objects.entry(key) {
            Entry::Occupied(_) => Err(ErrorNM::ItemExists),
            Entry::Vacant(vacant) => {
                vacant.insert(obj);
                Ok(item)
            }
        }
    }

    pub fn new_group(&self, key: String, obj: Arc<Object>) -> Result<Arc<Group>, ErrorNM> {
        if !self.allowed_groups {
            return Err(ErrorNM::NoPermissionCreateGroup);
        }

        let TypeObject::Group(group) = &obj.typ else {
            return Err(ErrorNM::InvalidObject);
        };
        let group = Arc::clone(group);
        match self.objects.entry(key) {
            Entry::Occupied(_) => Err(ErrorNM::GroupExists),
            Entry::Vacant(vacant) => {
                if group
                    .is_attached
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
                {
                    return Err(ErrorNM::GroupExists);
                }

                vacant.insert(obj);
                Ok(group)
            }
        }
    }

    pub fn delete_obj(&self, key: impl AsRef<str>) -> Result<(), ErrorNM> {
        let (_key, object) = self
            .objects
            .remove(key.as_ref())
            .ok_or(ErrorNM::NoFindObject)?;

        if let TypeObject::Group(group) = &object.typ {
            group.is_attached.store(false, Ordering::Release);
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

    pub fn find_obj_for_key(&self, key: impl AsRef<str>) -> Result<Arc<Object>, ErrorNM> {
        self.objects
            .get(key.as_ref())
            .map(|guard| Arc::clone(guard.value())) // или guard.value().clone()
            .ok_or(ErrorNM::NoFindObject)
    }
}

impl ToObject for Arc<Group> {
    fn to_object(&self) -> Result<Arc<Object>, ErrorNM> {
        self.object
            .as_ref()
            .ok_or(ErrorNM::NoFindObject)?
            .upgrade()
            .ok_or(ErrorNM::NoFindObject)
    }
}
