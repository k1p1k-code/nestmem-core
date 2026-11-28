use serde::{Deserialize, Serialize};

use crate::storage::ttl::SystemTtl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupTtlPolicy {
    PeriodicScheduled(SystemTtl),
    LazyItemTtl,
}
