// Публичный API для TypeScript/JavaScript
// Высокоуровневые методы для работы с мессенджером

pub mod messaging;
pub mod contacts;
pub mod crypto;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Главный API для мессенджера
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct MessengerAPI {
    // Внутренние компоненты будут добавлены позже
}

impl MessengerAPI {
    pub fn new() -> Self {
        Self {}
    }

    /// Инициализация мессенджера
    pub async fn initialize(&mut self) -> crate::utils::error::Result<()> {
        // TODO: Реализация
        Ok(())
    }
}

impl Default for MessengerAPI {
    fn default() -> Self {
        Self::new()
    }
}
