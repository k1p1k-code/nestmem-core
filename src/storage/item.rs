use std::sync::{Arc, Weak};

use crate::storage::object::{Object, ToObject};
use crate::{ErrorNM, storage::ttl::SystemTtl};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Item {
    #[serde(skip)]
    object: Weak<Object>,

    pub(crate) value: String,
    pub(crate) ttl: Option<SystemTtl>,
}

impl Item {
    pub(crate) fn new(value: String, ttl: Option<SystemTtl>, object: Weak<Object>) -> Arc<Self> {
        Arc::new(Self { object, value, ttl })
    }
    pub fn to_object(&self) -> Result<Arc<Object>, ErrorNM> {
        self.object.upgrade().ok_or(ErrorNM::ObjectNotFound)
    }
}

impl ToObject for Arc<Item> {
    fn to_object(&self) -> Result<Arc<Object>, ErrorNM> {
        self.object.upgrade().ok_or(ErrorNM::ObjectNotFound)
    }
}
