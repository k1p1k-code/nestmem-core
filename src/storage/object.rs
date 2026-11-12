use std::sync::{Arc, Weak};

use crate::ErrorNM;
use crate::storage::group::Group;
use crate::storage::item::Item;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Object {
    pub(crate) parent: Weak<Group>,
    pub typ: TypeObject,
}

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

    pub fn delete(&self) -> Result<(), ErrorNM> {
        let parent = self.parent.upgrade().ok_or(ErrorNM::NoFindObject)?;
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
            return Err(ErrorNM::NoFindObject);
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct SendGroup {
    pub items: Vec<Arc<Item>>,
}

mod tests {
    #[test]
    fn test_object_delete() {
        use crate::{NestMem, Group, GroupTtlPolicy, Item};
        let nestmem = NestMem::new();

        let sub_group = Group::new(GroupTtlPolicy::LazyItemTtl, true);
        let sub_group = nestmem
            .root
            .new_group("sub_group".to_string(), sub_group)
            .unwrap();

        nestmem
            .root
            .new_item("item1".to_string(), Item::new("val1".to_string(), None));
        nestmem
            .root
            .new_item("item2".to_string(), Item::new("val2".to_string(), None));

        let obj_item1 = nestmem.root.get_clone_object("item1").unwrap();
        assert!(obj_item1.delete().is_ok());
        assert!(nestmem.root.get_clone_object("item1").is_err());
        assert!(obj_item1.delete().is_err());
        assert!(nestmem.root.get_clone_object("item2").is_ok());

        let obj_group = nestmem.root.get_clone_object("sub_group").unwrap();
        assert!(obj_group.delete().is_ok());
        assert!(nestmem.root.get_clone_object("sub_group").is_err());
        assert!(
            nestmem
                .root
                .new_group("sub_group_moved".to_string(), sub_group)
                .is_ok()
        );
    }
}
