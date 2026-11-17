#[cfg(feature = "sync-cleaner")]
mod sync;
#[cfg(feature = "sync-cleaner")]
pub use sync::CleanerThread;

#[cfg(feature = "tokio-cleaner")]
mod tokio;
#[cfg(feature = "tokio-cleaner")]
pub use tokio::CleanerTokio;

use crate::Group;
use std::sync::Weak;
pub trait CleanerTrait {
    fn new(weak_root: Weak<Group>) -> Self;
    fn run(&mut self);
    fn stop(&mut self);
}

pub struct NoCleaner;
impl CleanerTrait for NoCleaner {
    fn new(_: Weak<Group>) -> Self {
        Self
    }
    fn run(&mut self) {}
    fn stop(&mut self) {}
}

#[cfg(any(feature = "sync-cleaner", feature = "tokio-cleaner"))]
macro_rules! impl_drop_cleaner {
    ($($t:ty),*) => {
        $(
            impl Drop for $t {
                fn drop(&mut self) {
                    self.stop();
                }
            }
        )*
    };
}

#[cfg(any(feature = "sync-cleaner", feature = "tokio-cleaner"))]
pub(crate) use impl_drop_cleaner;
