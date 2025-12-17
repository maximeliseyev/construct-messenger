// Типы ошибок

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConstructError {
    #[error("Cryptography error: {0}")]
    CryptoError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, ConstructError>;

// Alias для совместимости
pub type MessengerError = ConstructError;

// Для WASM-биндингов
#[cfg(target_arch = "wasm32")]
impl From<ConstructError> for wasm_bindgen::JsValue {
    fn from(error: ConstructError) -> Self {
        wasm_bindgen::JsValue::from_str(&error.to_string())
    }
}
