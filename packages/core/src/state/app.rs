// Главное состояние приложения

use crate::utils::error::Result;

pub struct AppState {
    // TODO: Реализация
}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
