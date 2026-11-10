use crate::storage::ttl::SystemTtl;

#[derive(Debug)]
pub enum GroupTtlPolicy{
    PeriodicScheduled(SystemTtl),
    Lazy(SystemTtl),
    ItemTtl
}