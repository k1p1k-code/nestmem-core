use crate::storage::ttl::{self, SystemTtl};
use serde::{Serialize, de::value};
use std::time::Instant;

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
