// API для отправки и получения сообщений

use crate::utils::error::Result;

/// Отправить сообщение контакту
pub async fn send_message(contact_id: &str, text: &str) -> Result<String> {
    // TODO: Реализация
    Ok(format!("Message sent to {}: {}", contact_id, text))
}

/// Получить историю сообщений
pub async fn get_messages(contact_id: &str) -> Result<Vec<String>> {
    // TODO: Реализация
    Ok(vec![])
}
