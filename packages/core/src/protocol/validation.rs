// Валидация входящих данных

use crate::utils::error::{ConstructError, Result};

pub fn validate_username(username: &str) -> Result<()> {
    if username.len() < 3 || username.len() > 32 {
        return Err(ConstructError::ValidationError(
            "Username must be between 3 and 32 characters".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_message(message: &str) -> Result<()> {
    if message.is_empty() {
        return Err(ConstructError::ValidationError(
            "Message cannot be empty".to_string(),
        ));
    }
    Ok(())
}
