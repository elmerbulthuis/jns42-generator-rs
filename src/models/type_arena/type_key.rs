use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_KEY: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct TypeKey(usize);
impl TypeKey {
    pub fn new() -> Self {
        Self(NEXT_KEY.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for TypeKey {
    fn default() -> Self {
        Self::new()
    }
}