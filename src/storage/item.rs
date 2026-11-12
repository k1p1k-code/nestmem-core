use crate::storage::ttl::SystemTtl;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Item {
    pub value: String,
    pub ttl: Option<SystemTtl>,
}

impl Item {
    pub fn new(value: String, ttl: Option<SystemTtl>) -> Self {
        Self { value, ttl }
    }
}
