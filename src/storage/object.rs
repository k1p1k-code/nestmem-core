use enum_dispatch::enum_dispatch;
use serde::Serialize;
use std::sync::{Arc, Weak};

use crate::storage::group::Group;
use crate::storage::item::Item;
use crate::{ErrorNM, GroupTtlPolicy, SystemTtl};

#[enum_dispatch]
pub trait ToObject {
    fn to_object(&self) -> Result<Arc<Object>, ErrorNM>;
}

#[derive(Debug, Clone)]
pub struct Object {
    pub(crate) parent: Weak<Group>,
    pub(crate) typ: TypeObject,
}

#[enum_dispatch(ToObject)]
#[derive(Debug, Clone)]
pub enum TypeObject {
    Item(Arc<Item>),
    Group(Arc<Group>),
}

impl Object {
    pub fn get_value(&self) -> String {
        match &self.typ {
            TypeObject::Item(i) => i.value.clone(),
            TypeObject::Group(g) => {
                let items = g.withinme_items();
                let data = SendGroup { items };
                serde_json::json!(data).to_string()
            }
        }
    }

    pub fn item(
        name: String,
        value: String,
        ttl: Option<SystemTtl>,
        parent: Weak<Group>,
    ) -> Result<Arc<Item>, ErrorNM> {
        let arc_parent = parent.upgrade().ok_or(ErrorNM::ParentNotExist)?;
        let mut created_item = None;
        let arc_object = Arc::new_cyclic(|weak_self| {
            let item = Item::new(value, ttl, weak_self.clone());
            created_item = Some(item.clone());
            Object {
                parent,
                typ: TypeObject::Item(item),
            }
        });
        let item = created_item.unwrap();
        arc_parent.new_item(name, arc_object)?;
        Ok(item)
    }

    pub fn group(
        name: String,
        allowed_groups: bool,
        ttl_policy: GroupTtlPolicy,
        parent: Weak<Group>,
    ) -> Result<Arc<Group>, ErrorNM> {
        let arc_parent = parent.upgrade().ok_or(ErrorNM::ParentNotExist)?;
        let mut created_group = None;
        let arc_object = Arc::new_cyclic(|weak_self| {
            let group = Group::new(weak_self.clone(), ttl_policy, allowed_groups);
            created_group = Some(group.clone());
            Object {
                parent,
                typ: TypeObject::Group(group),
            }
        });

        let group = created_group.unwrap();
        arc_parent.new_group(name, arc_object)?;
        Ok(group)
    }

    pub fn delete(&self) -> Result<(), ErrorNM> {
        let parent = self.parent.upgrade().ok_or(ErrorNM::ObjectNotFound)?;
        let mut deleted = false;
        parent
            .objects
            .retain(|_key, obj| match (&self.typ, &obj.typ) {
                (TypeObject::Group(g1), TypeObject::Group(g2)) => {
                    if Arc::ptr_eq(g1, g2) {
                        g2.is_attached
                            .store(false, std::sync::atomic::Ordering::Release);
                        deleted = true;
                        return false;
                    }
                    true
                }
                (TypeObject::Item(i1), TypeObject::Item(i2)) => {
                    if Arc::ptr_eq(i1, i2) {
                        deleted = true;
                        return false;
                    }
                    true
                }

                _ => true,
            });
        if !deleted {
            return Err(ErrorNM::ObjectNotFound);
        }
        Ok(())
    }
}

impl ToObject for Object {
    fn to_object(&self) -> Result<Arc<Object>, ErrorNM> {
        self.typ.to_object()
    }
}

#[derive(Serialize)]
pub struct SendGroup {
    pub items: Vec<Arc<Item>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NestMem;

    #[test]
    fn test_to_object_lifecycle() {
        let nestmem = NestMem::new();
        assert_eq!(nestmem.root.to_object().unwrap_err(), ErrorNM::ObjectNotFound);
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
        let group_obj = group.to_object().expect("Object group must life");
        let item_obj = item.to_object().expect("Object item must life");
        let stored_group_obj = nestmem.root.get_clone_object("users").unwrap();
        assert!(Arc::ptr_eq(&group_obj, &stored_group_obj));

        drop(item_obj);
        group.delete_obj("admin").unwrap();

        assert_eq!(item.to_object().unwrap_err(), ErrorNM::ObjectNotFound);

        group_obj.delete().unwrap();
        drop(group_obj);
        drop(stored_group_obj);
        assert_eq!(group.to_object().unwrap_err(), ErrorNM::ObjectNotFound);
    }
}
