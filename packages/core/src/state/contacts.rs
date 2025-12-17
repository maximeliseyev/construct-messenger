// Состояние контактов

use crate::storage::models::StoredContact;

pub struct ContactsState {
    contacts: Vec<StoredContact>,
}

impl ContactsState {
    pub fn new() -> Self {
        Self {
            contacts: Vec::new(),
        }
    }
}

impl Default for ContactsState {
    fn default() -> Self {
        Self::new()
    }
}
