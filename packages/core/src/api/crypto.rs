// API для криптографических операций

use crate::crypto::keys::KeyManager;
use crate::crypto::session::SessionManager;
use crate::crypto::x3dh::PublicKeyBundle;
use crate::crypto::ClientCrypto;
use crate::utils::error::{ConstructError, Result};
use serde::{Deserialize, Serialize};

/// Данные ключей для экспорта/импорта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBundle {
    pub identity_public: Vec<u8>,
    pub signed_prekey_public: Vec<u8>,
    pub signature: Vec<u8>,
    pub verifying_key: Vec<u8>,
}

impl From<PublicKeyBundle> for KeyBundle {
    fn from(bundle: PublicKeyBundle) -> Self {
        Self {
            identity_public: bundle.identity_public,
            signed_prekey_public: bundle.signed_prekey_public,
            signature: bundle.signature,
            verifying_key: bundle.verifying_key,
        }
    }
}

impl From<crate::crypto::RegistrationBundle> for KeyBundle {
    fn from(bundle: crate::crypto::RegistrationBundle) -> Self {
        Self {
            identity_public: bundle.identity_public,
            signed_prekey_public: bundle.signed_prekey_public,
            signature: bundle.signature,
            verifying_key: bundle.verifying_key,
        }
    }
}

impl From<KeyBundle> for PublicKeyBundle {
    fn from(bundle: KeyBundle) -> Self {
        Self {
            identity_public: bundle.identity_public,
            signed_prekey_public: bundle.signed_prekey_public,
            signature: bundle.signature,
            verifying_key: bundle.verifying_key,
        }
    }
}

/// Регистрационный bundle в base64
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationBundleB64 {
    pub identity_public: String,
    pub signed_prekey_public: String,
    pub signature: String,
    pub verifying_key: String,
}

/// Менеджер криптографии высокого уровня
pub struct CryptoManager {
    key_manager: KeyManager,
    session_manager: SessionManager,
    client: ClientCrypto,
}

impl CryptoManager {
    /// Создать новый CryptoManager
    pub fn new() -> Result<Self> {
        let mut key_manager = KeyManager::new();
        key_manager.initialize()?;

        let client = ClientCrypto::new()
            .map_err(|e| ConstructError::CryptoError(e))?;

        Ok(Self {
            key_manager,
            session_manager: SessionManager::new(),
            client,
        })
    }

    /// Получить KeyManager
    pub fn key_manager(&self) -> &KeyManager {
        &self.key_manager
    }

    /// Получить изменяемый KeyManager
    pub fn key_manager_mut(&mut self) -> &mut KeyManager {
        &mut self.key_manager
    }

    /// Получить SessionManager
    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Получить изменяемый SessionManager
    pub fn session_manager_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }

    /// Экспортировать registration bundle
    pub fn export_registration_bundle(&self) -> Result<KeyBundle> {
        let bundle = self.key_manager.export_registration_bundle()?;
        Ok(bundle.into())
    }

    /// Экспортировать registration bundle в base64
    pub fn export_registration_bundle_b64(&self) -> Result<RegistrationBundleB64> {
        let bundle = self.key_manager.export_registration_bundle()?;
        Ok(RegistrationBundleB64 {
            identity_public: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bundle.identity_public),
            signed_prekey_public: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bundle.signed_prekey_public),
            signature: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bundle.signature),
            verifying_key: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bundle.verifying_key),
        })
    }

    /// Экспортировать публичный key bundle
    pub fn export_public_bundle(&self) -> Result<KeyBundle> {
        let bundle = self.key_manager.export_public_bundle()?;
        Ok(bundle.into())
    }

    /// Ротация signed prekey
    pub fn rotate_prekey(&mut self) -> Result<()> {
        self.key_manager.rotate_signed_prekey()
    }

    /// Подписать данные
    pub fn sign_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.key_manager.sign(data)
    }

    /// Проверка наличия сессии
    pub fn has_session(&self, contact_id: &str) -> bool {
        self.session_manager.has_session(contact_id)
    }

    /// Количество активных сессий
    pub fn active_sessions_count(&self) -> usize {
        self.session_manager.session_count()
    }

    /// Очистка старых сессий
    pub fn cleanup_old_sessions(&mut self, max_age_seconds: i64) {
        self.session_manager.cleanup_sessions_older_than(max_age_seconds);
    }

    /// Инициализировать сессию с контактом
    pub fn init_session(&mut self, contact_id: &str, remote_bundle: &KeyBundle) -> Result<String> {
        let public_bundle: PublicKeyBundle = remote_bundle.clone().into();
        self.client.init_session(contact_id, &public_bundle)
            .map_err(|e| ConstructError::CryptoError(e))
    }

    /// Зашифровать сообщение для контакта
    pub fn encrypt_message(&mut self, session_id: &str, plaintext: &str) -> Result<Vec<u8>> {
        let encrypted = self.client.encrypt_ratchet_message(session_id, plaintext.as_bytes())
            .map_err(|e| ConstructError::CryptoError(e))?;

        // Сериализовать EncryptedRatchetMessage в байты
        bincode::serialize(&encrypted)
            .map_err(|e| ConstructError::SerializationError(e.to_string()))
    }

    /// Расшифровать сообщение от контакта
    pub fn decrypt_message(&mut self, session_id: &str, ciphertext: &[u8]) -> Result<String> {
        // Десериализовать EncryptedRatchetMessage из байтов
        let encrypted: crate::crypto::double_ratchet::EncryptedRatchetMessage =
            bincode::deserialize(ciphertext)
                .map_err(|e| ConstructError::SerializationError(e.to_string()))?;

        let plaintext = self.client.decrypt_ratchet_message(session_id, &encrypted)
            .map_err(|e| ConstructError::CryptoError(e))?;

        String::from_utf8(plaintext)
            .map_err(|e| ConstructError::SerializationError(format!("Invalid UTF-8: {}", e)))
    }

    /// Получить доступ к клиенту криптографии
    pub fn client(&self) -> &ClientCrypto {
        &self.client
    }

    /// Получить изменяемый доступ к клиенту криптографии
    pub fn client_mut(&mut self) -> &mut ClientCrypto {
        &mut self.client
    }

    /// Экспортировать приватные ключи для шифрования
    pub fn export_private_keys(&self) -> Result<crate::crypto::master_key::PrivateKeys> {
        // Получить identity secret key
        let identity_secret = self.key_manager.identity_secret_key()?;
        let identity_bytes = identity_secret.to_bytes();

        // Получить signing key
        let signing_key = self.key_manager.signing_secret_key()?;
        let signing_bytes = signing_key.to_bytes();

        // Получить signed prekey secret
        let prekey = self.key_manager.current_signed_prekey()?;
        let prekey_bytes = prekey.key_pair.secret.to_bytes();

        Ok(crate::crypto::master_key::PrivateKeys::new(
            identity_bytes,
            signing_bytes,
            prekey_bytes,
        ))
    }

    /// Импортировать приватные ключи после расшифровки
    pub fn import_private_keys(
        &mut self,
        keys: &crate::crypto::master_key::PrivateKeys,
        prekey_signature: Vec<u8>,
    ) -> Result<()> {
        // Конвертировать байты в ключи
        let (identity_secret, signing_key, prekey_secret) = keys.to_keys()?;

        // Импортировать в KeyManager
        self.key_manager
            .import_keys(identity_secret, signing_key, prekey_secret, prekey_signature)?;

        Ok(())
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new().expect("Failed to create CryptoManager")
    }
}

/// Создать новый crypto клиент
pub fn create_client() -> Result<ClientCrypto> {
    ClientCrypto::new().map_err(|e| ConstructError::CryptoError(e))
}

/// Получить публичные ключи для регистрации
pub fn get_registration_bundle(client: &ClientCrypto) -> Result<KeyBundle> {
    let bundle = client.get_registration_bundle();
    Ok(KeyBundle {
        identity_public: bundle.identity_public,
        signed_prekey_public: bundle.signed_prekey_public,
        signature: bundle.signature,
        verifying_key: bundle.verifying_key,
    })
}

/// Сериализовать KeyBundle в JSON
pub fn serialize_key_bundle(bundle: &KeyBundle) -> Result<String> {
    serde_json::to_string(bundle)
        .map_err(|e| ConstructError::SerializationError(e.to_string()))
}

/// Десериализовать KeyBundle из JSON
pub fn deserialize_key_bundle(json: &str) -> Result<KeyBundle> {
    serde_json::from_str(json)
        .map_err(|e| ConstructError::SerializationError(e.to_string()))
}

/// Конвертировать байты в base64
pub fn bytes_to_base64(bytes: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
}

/// Конвертировать base64 в байты
pub fn base64_to_bytes(base64_str: &str) -> Result<Vec<u8>> {
    base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_str)
        .map_err(|e| ConstructError::SerializationError(format!("Invalid base64: {}", e)))
}

/// Генерировать случайные байты для nonce, salt и т.д.
pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut bytes = vec![0u8; len];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_manager_creation() {
        let manager = CryptoManager::new();
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.active_sessions_count(), 0);
    }

    #[test]
    fn test_base64_conversion() {
        let data = b"hello world";
        let b64 = bytes_to_base64(data);
        let decoded = base64_to_bytes(&b64).unwrap();
        assert_eq!(data, decoded.as_slice());
    }

    #[test]
    fn test_random_bytes() {
        let bytes1 = generate_random_bytes(32);
        let bytes2 = generate_random_bytes(32);

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2); // Должны быть разными
    }

    #[test]
    fn test_export_registration_bundle() {
        let manager = CryptoManager::new().unwrap();
        let bundle = manager.export_registration_bundle();
        assert!(bundle.is_ok());

        let bundle = bundle.unwrap();
        assert_eq!(bundle.identity_public.len(), 32);
        assert_eq!(bundle.signed_prekey_public.len(), 32);
        assert_eq!(bundle.signature.len(), 64);
        assert_eq!(bundle.verifying_key.len(), 32);
    }
}
