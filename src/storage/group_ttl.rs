use crate::storage::ttl::SystemTtl;

#[derive(Debug, Clone)]
pub enum GroupTtlPolicy {
    PeriodicScheduled(SystemTtl),
    LazyItemTtl,
}
