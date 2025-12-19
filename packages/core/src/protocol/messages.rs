// Типы сообщений протокола
// Соответствуют спецификации WebSocket API

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

/// Основной тип сообщения для чата (Double Ratchet совместимый)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// UUID v4 идентификатор сообщения
    pub id: String,
    /// UUID отправителя
    pub from: String,
    /// UUID получателя
    pub to: String,
    /// X25519 ephemeral public key (32 bytes)
    #[serde(with = "serde_bytes")]
    pub ephemeral_public_key: Vec<u8>,
    /// Номер сообщения в цепочке
    pub message_number: u32,
    /// Зашифрованное содержимое (ChaCha20-Poly1305)
    pub content: String, // Base64 encoded
    /// Unix timestamp в секундах
    pub timestamp: i64,
}

/// Регистрационный bundle с публичными ключами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationBundle {
    /// Base64 X25519 identity public key (44 chars)
    pub identity_public: String,
    /// Base64 X25519 signed prekey public (44 chars)
    pub signed_prekey_public: String,
    /// Base64 Ed25519 signature (88 chars)
    pub signature: String,
    /// Base64 Ed25519 verifying key (44 chars)
    pub verifying_key: String,
}

/// Публичный ключевой bundle пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyBundleData {
    /// UUID пользователя
    pub user_id: String,
    /// Base64 identity public key
    pub identity_public: String,
    /// Base64 signed prekey public
    pub signed_prekey_public: String,
    /// Base64 signature
    pub signature: String,
    /// Base64 verifying key
    pub verifying_key: String,
}

/// Подтверждение получения сообщения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckData {
    /// ID подтвержденного сообщения
    pub message_id: String,
    /// Серверный timestamp
    pub timestamp: i64,
}

/// Данные об ошибке
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    /// Внутренний код ошибки
    pub code: u32,
    /// Человекочитаемое сообщение
    pub message: String,
    /// Опциональный ID исходного запроса
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Типы сообщений WebSocket протокола
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ProtocolMessage {
    /// Регистрация нового пользователя
    Register {
        username: String,
        bundle: RegistrationBundle,
    },

    /// Вход в систему
    Login {
        username: String,
    },

    /// Отправка зашифрованного сообщения
    SendMessage(ChatMessage),

    /// Получение зашифрованного сообщения
    ReceiveMessage(ChatMessage),

    /// Подтверждение получения
    Ack(AckData),

    /// Ошибка
    Error(ErrorData),

    /// Запрос публичных ключей пользователя
    RequestKeyBundle {
        user_id: String,
    },

    /// Ответ с публичными ключами
    KeyBundleResponse(PublicKeyBundleData),

    /// Пинг для поддержания соединения
    Ping,

    /// Понг ответ на пинг
    Pong,
}

impl ProtocolMessage {
    /// Создать сообщение об ошибке
    pub fn error(code: u32, message: String) -> Self {
        Self::Error(ErrorData {
            code,
            message,
            request_id: None,
        })
    }

    /// Создать сообщение об ошибке с request_id
    pub fn error_with_request_id(code: u32, message: String, request_id: String) -> Self {
        Self::Error(ErrorData {
            code,
            message,
            request_id: Some(request_id),
        })
    }
}
