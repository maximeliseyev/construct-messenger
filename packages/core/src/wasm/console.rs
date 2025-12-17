// Консольное логирование для WASM

#[cfg(target_arch = "wasm32")]
pub fn init_logging() {
    use tracing_subscriber::fmt;

    // Настраиваем логирование для WASM
    // TODO: Более продвинутая настройка
}

#[cfg(target_arch = "wasm32")]
pub fn log(message: &str) {
    web_sys::console::log_1(&message.into());
}

#[cfg(target_arch = "wasm32")]
pub fn error(message: &str) {
    web_sys::console::error_1(&message.into());
}
