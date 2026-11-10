use crate::storage::ttl::SystemTtl;

#[derive(Debug)]
pub enum GroupTtlPolicy{
    PeriodicScheduled(SystemTtl),
    LazyItemTtl
}