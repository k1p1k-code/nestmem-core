pub(crate) mod error;
use error::ErrorSW;

use std::sync::Arc;

use crate::{ErrorNM, Group, NestMem, ToObject, TypeObject};

pub struct StorageWalker {
    current_dir: Arc<Group>,
    root_dir: Arc<Group>,
}

impl StorageWalker {
    pub fn from_nestmem(nestmem: &NestMem) -> Self {
        Self {
            current_dir: nestmem.root.clone(),
            root_dir: nestmem.root.clone(),
        }
    }

    pub fn from_group(group: Arc<Group>) -> Self {
        Self {
            current_dir: group.clone(),
            root_dir: group.clone(),
        }
    }

    pub fn up(&mut self) -> Result<(), ErrorSW> {
        let object = self.current_dir.to_object()?;
        let new_current_dir = object.parent.upgrade().ok_or(ErrorNM::ParentNotExist)?;
        if !self.root_dir.contains_group(&new_current_dir) {
            return Err(ErrorSW::UpHead);
        }

        self.current_dir = new_current_dir;
        Ok(())
    }

    /// This only works with subfolders within the current directory.
    pub fn cd_obj(&mut self, obj: Arc<Group>) -> Result<(), ErrorSW> {
        if !self.root_dir.contains_group(&obj) {
            return Err(ErrorSW::UnknownDir);
        }
        self.current_dir = obj;
        Ok(())
    }

    pub fn cd_str(&mut self, name: impl AsRef<str>) -> Result<(), ErrorSW> {
        let TypeObject::Group(group) = &self.current_dir.find_obj_for_key(name)?.typ else {
            return Err(ErrorSW::InvalidObject);
        };
        self.current_dir = group.clone();

        Ok(())
    }
}

#[test]
fn check_up_cd() {
    use crate::{GroupTtlPolicy, Object};
    let root = Group::new_root();
    let shop = Object::group(
        "shop".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        Arc::downgrade(&root),
    )
    .unwrap();
    let products = Object::group(
        "products".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        Arc::downgrade(&shop),
    )
    .unwrap();
    let mut shop_walker = StorageWalker::from_group(shop);
    assert!(shop_walker.up().is_err());
    assert!(shop_walker.cd_obj(root).is_err());
    shop_walker.cd_obj(products.clone()).unwrap();
    shop_walker.up().unwrap();
    let mut products_walker = StorageWalker::from_group(products.clone());
    let _orders = Object::group(
        "orders".to_string(),
        true,
        GroupTtlPolicy::LazyItemTtl,
        Arc::downgrade(&products),
    )
    .unwrap();
    assert!(products_walker.up().is_err());
    products_walker.cd_str("orders").unwrap();
}
