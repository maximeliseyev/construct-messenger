// In-memory storage для тестов и non-WASM платформ

use crate::utils::error::Result;

pub struct KeyStorage {
    // TODO: Реализация
}

impl KeyStorage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for KeyStorage {
    fn default() -> Self {
        Self::new()
    }
}
