use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum SystemTtl {
    Minutes(u64),
    Seconds(u64)
}  

