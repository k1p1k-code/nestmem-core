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
        parent
            .objects
            .retain(|_key, obj| match (&self.typ, &obj.typ) {
                (TypeObject::Group(g1), TypeObject::Group(g2)) => !Arc::ptr_eq(g1, g2),
                (TypeObject::Item(i1), TypeObject::Item(i2)) => !Arc::ptr_eq(i1, i2),
                _ => true,
            });
        Ok(())
    }
}

#[derive(Serialize)]
pub struct SendGroup {
    pub items: Vec<Arc<Item>>,
}
