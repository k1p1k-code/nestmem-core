use serde::Serialize;
use crate::storage::ttl::SystemTtl;

#[derive(Debug, Serialize, Clone)]
pub struct Item{
    pub value: String,
    pub ttl: Option<SystemTtl>
}