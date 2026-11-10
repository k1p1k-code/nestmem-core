use serde::Serialize;
use crate::storage::item::Item;
use crate::storage::group::Group;

#[derive(Debug)]
pub enum Object {
    Item(Item),
    Group(Group)
}

impl Object {
    fn get_value(&self) -> String{
        match self {
            Object::Item(i) => {i.value.clone()},
            Object::Group(g) => {
                let items = g.withinme_items();
                let data = SendGroup{
                    items
                };
                serde_json::json!(data).to_string()
            }
        }
    }
}

#[derive(Serialize)]
pub struct SendGroup{
    pub items: Vec<Item> 
}