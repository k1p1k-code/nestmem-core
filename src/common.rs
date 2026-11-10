use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Serialize)]
enum SystemTtl {
    Minutes,
    Seconds
}  

#[derive(Debug, Serialize)]
pub struct Ttl{
    time_ttl: usize,
    system_ttl: SystemTtl
}

#[derive(Debug, Serialize)]
pub struct Item{
    pub value: String,
    pub ttl: Option<Ttl>
}

#[derive(Debug)]
pub struct Group{
    name: String,
    objects: HashMap<String, Object>,
    ttl: Option<Ttl>,
    allowed_groups: bool
}
impl Group {
    pub fn new_main() -> Self{
        Group { 
            name: "main".to_string(),
            objects: HashMap::new(),
            ttl: None,
            allowed_groups: true }
    }

    pub fn new(name: String, ttl: Option<Ttl>, allowed_groups: bool) -> Self{
        Group { name, objects: HashMap::new(), ttl: ttl, allowed_groups: allowed_groups }
    }

    pub fn get_name(&self) -> String{
        self.name.clone()
    }

    pub fn withinme_items(&self) -> Vec<&Item>{
        self.objects
            .values()
            .filter_map(|i|{match i {
                    Object::Item(e) => Some(e),
                    Object::Group(_) => None
                }
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
                    group: g.get_name(),
                    items
                };
                serde_json::json!(data).to_string()
            }
        }
    }
}


#[derive(Serialize)]
pub struct SendGroup<'a>{
    pub group: String,
    pub items: Vec<&'a Item> 
}