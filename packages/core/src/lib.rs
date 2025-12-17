// Construct Messenger Core
// Rust/WASM engine with end-to-end encryption

#![warn(clippy::all)]
#![allow(clippy::too_many_arguments)]

// Модули
pub mod api;
pub mod crypto;
pub mod protocol;
pub mod storage;
pub mod state;
pub mod utils;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

// Re-exports для удобства
pub use api::MessengerAPI;
pub use crypto::ClientCrypto;
pub use utils::error::Result;

// WASM экспорты
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init() {
    // Настройка panic hook для лучшей отладки в браузере
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Инициализация логирования
    wasm::console::init_logging();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
