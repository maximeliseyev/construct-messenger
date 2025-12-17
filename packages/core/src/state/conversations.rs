// Состояние бесед

use crate::storage::models::StoredMessage;

pub struct ConversationsState {
    messages: Vec<StoredMessage>,
}

impl ConversationsState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

impl Default for ConversationsState {
    fn default() -> Self {
        Self::new()
    }
}
