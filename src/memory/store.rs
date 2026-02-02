use crate::types::Message;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
pub struct MemoryStore {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug, Default)]
struct Inner {
    pub short_term: Vec<Message>,
    pub long_term: Vec<Message>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::default())),
        }
    }

    pub fn add_short(&self, m: Message) {
        self.inner.write().short_term.push(m);
    }

    pub fn add_long(&self, m: Message) {
        self.inner.write().long_term.push(m);
    }

    pub fn get_short(&self) -> Vec<Message> {
        self.inner.read().short_term.clone()
    }

    pub fn get_long(&self) -> Vec<Message> {
        self.inner.read().long_term.clone()
    }
}
