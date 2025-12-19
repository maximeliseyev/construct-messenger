// Валидация входящих данных

use crate::protocol::messages::{ChatMessage, ProtocolMessage, RegistrationBundle};
use crate::utils::error::{ConstructError, Result};

/// Валидация имени пользователя
pub fn validate_username(username: &str) -> Result<()> {
    if username.len() < 3 || username.len() > 32 {
        return Err(ConstructError::ValidationError(
            "Username must be between 3 and 32 characters".to_string(),
        ));
    }

    // Проверка на допустимые символы (буквы, цифры, подчеркивание, дефис)
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ConstructError::ValidationError(
            "Username can only contain alphanumeric characters, underscores, and hyphens"
                .to_string(),
        ));
    }

    Ok(())
}

/// Валидация UUID v4
pub fn validate_uuid(uuid: &str) -> Result<()> {
    // Простая проверка формата UUID
    if uuid.len() != 36 {
        return Err(ConstructError::ValidationError(
            "Invalid UUID format: incorrect length".to_string(),
        ));
    }

    let parts: Vec<&str> = uuid.split('-').collect();
    if parts.len() != 5 || parts[0].len() != 8 || parts[1].len() != 4 || parts[2].len() != 4 || parts[3].len() != 4 || parts[4].len() != 12 {
        return Err(ConstructError::ValidationError(
            "Invalid UUID format".to_string(),
        ));
    }

    Ok(())
}

/// Валидация ChatMessage
pub fn validate_chat_message(msg: &ChatMessage) -> Result<()> {
    // Проверка UUID
    validate_uuid(&msg.id)?;
    validate_uuid(&msg.from)?;
    validate_uuid(&msg.to)?;

    // Проверка ephemeral key (должен быть 32 байта для X25519)
    if msg.ephemeral_public_key.len() != 32 {
        return Err(ConstructError::ValidationError(
            "Ephemeral public key must be 32 bytes".to_string(),
        ));
    }

    // Проверка зашифрованного содержимого
    if msg.content.is_empty() {
        return Err(ConstructError::ValidationError(
            "Message content cannot be empty".to_string(),
        ));
    }

    // Проверка timestamp (не должен быть в будущем)
    let now = crate::utils::time::current_timestamp();
    if msg.timestamp > now + 60 {
        // Разрешаем небольшую погрешность в 60 секунд
        return Err(ConstructError::ValidationError(
            "Message timestamp is in the future".to_string(),
        ));
    }

    Ok(())
}

/// Валидация RegistrationBundle
pub fn validate_registration_bundle(bundle: &RegistrationBundle) -> Result<()> {
    // Base64 X25519 public key должен быть 44 символа
    if bundle.identity_public.len() != 44 {
        return Err(ConstructError::ValidationError(
            "Identity public key must be 44 characters (32 bytes base64)".to_string(),
        ));
    }

    if bundle.signed_prekey_public.len() != 44 {
        return Err(ConstructError::ValidationError(
            "Signed prekey public must be 44 characters (32 bytes base64)".to_string(),
        ));
    }

    // Ed25519 signature должна быть 88 символов (64 bytes base64)
    if bundle.signature.len() != 88 {
        return Err(ConstructError::ValidationError(
            "Signature must be 88 characters (64 bytes base64)".to_string(),
        ));
    }

    // Ed25519 verifying key должен быть 44 символа
    if bundle.verifying_key.len() != 44 {
        return Err(ConstructError::ValidationError(
            "Verifying key must be 44 characters (32 bytes base64)".to_string(),
        ));
    }

    Ok(())
}

/// Валидация ProtocolMessage
pub fn validate_protocol_message(msg: &ProtocolMessage) -> Result<()> {
    match msg {
        ProtocolMessage::Register { username, bundle } => {
            validate_username(username)?;
            validate_registration_bundle(bundle)?;
        }
        ProtocolMessage::Login { username } => {
            validate_username(username)?;
        }
        ProtocolMessage::SendMessage(chat_msg) | ProtocolMessage::ReceiveMessage(chat_msg) => {
            validate_chat_message(chat_msg)?;
        }
        ProtocolMessage::RequestKeyBundle { user_id } => {
            validate_uuid(user_id)?;
        }
        ProtocolMessage::KeyBundleResponse(bundle_data) => {
            validate_uuid(&bundle_data.user_id)?;
            // Проверка ключей
            if bundle_data.identity_public.len() != 44 {
                return Err(ConstructError::ValidationError(
                    "Invalid identity public key length".to_string(),
                ));
            }
        }
        // Ping, Pong, Ack, Error не требуют специальной валидации
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("us").is_err()); // Слишком короткое
        assert!(validate_username("a".repeat(33).as_str()).is_err()); // Слишком длинное
        assert!(validate_username("user@123").is_err()); // Недопустимые символы
    }

    #[test]
    fn test_validate_uuid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_uuid("invalid-uuid").is_err());
        assert!(validate_uuid("").is_err());
    }

    #[test]
    fn test_validate_chat_message() {
        let msg = ChatMessage {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            from: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            to: "550e8400-e29b-41d4-a716-446655440002".to_string(),
            ephemeral_public_key: vec![0u8; 32],
            message_number: 1,
            content: "encrypted_content".to_string(),
            timestamp: crate::utils::time::current_timestamp(),
        };

        assert!(validate_chat_message(&msg).is_ok());

        // Тест с неверным ephemeral key
        let mut bad_msg = msg.clone();
        bad_msg.ephemeral_public_key = vec![0u8; 16]; // Неверная длина
        assert!(validate_chat_message(&bad_msg).is_err());
    }
}
