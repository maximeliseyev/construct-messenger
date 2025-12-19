// Управление ключами
// Хранение и ротация криптографических ключей

use crate::utils::error::{ConstructError, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroizing;

/// Пара ключей X25519
#[derive(Clone)]
pub struct X25519KeyPair {
    pub secret: StaticSecret,
    pub public: PublicKey,
}

impl X25519KeyPair {
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    pub fn from_secret(secret: StaticSecret) -> Self {
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }
}

/// Пара ключей Ed25519 для подписи
#[derive(Clone)]
pub struct Ed25519KeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Ed25519KeyPair {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }
}

/// Хранилище prekey с метаданными
#[derive(Clone)]
pub struct PrekeyStore {
    pub key_pair: X25519KeyPair,
    pub signature: Vec<u8>,
    pub created_at: i64,
    pub key_id: u32,
}

/// Менеджер криптографических ключей
pub struct KeyManager {
    /// Identity ключ (долговременный)
    identity_key: Option<X25519KeyPair>,

    /// Signing ключ для подписей
    signing_key: Option<Ed25519KeyPair>,

    /// Текущий signed prekey
    current_signed_prekey: Option<PrekeyStore>,

    /// История старых prekey для обратной совместимости
    old_prekeys: HashMap<u32, PrekeyStore>,

    /// Счетчик для key_id
    next_prekey_id: u32,
}

impl KeyManager {
    /// Создать новый KeyManager
    pub fn new() -> Self {
        Self {
            identity_key: None,
            signing_key: None,
            current_signed_prekey: None,
            old_prekeys: HashMap::new(),
            next_prekey_id: 1,
        }
    }

    /// Инициализировать с новыми ключами
    pub fn initialize(&mut self) -> Result<()> {
        self.identity_key = Some(X25519KeyPair::generate());
        self.signing_key = Some(Ed25519KeyPair::generate());
        self.rotate_signed_prekey()?;
        Ok(())
    }

    /// Получить identity public key
    pub fn identity_public_key(&self) -> Result<&PublicKey> {
        self.identity_key
            .as_ref()
            .map(|k| &k.public)
            .ok_or_else(|| ConstructError::CryptoError("Identity key not initialized".to_string()))
    }

    /// Получить identity secret key
    pub fn identity_secret_key(&self) -> Result<&StaticSecret> {
        self.identity_key
            .as_ref()
            .map(|k| &k.secret)
            .ok_or_else(|| ConstructError::CryptoError("Identity key not initialized".to_string()))
    }

    /// Получить verifying key
    pub fn verifying_key(&self) -> Result<&VerifyingKey> {
        self.signing_key
            .as_ref()
            .map(|k| &k.verifying_key)
            .ok_or_else(|| ConstructError::CryptoError("Signing key not initialized".to_string()))
    }

    /// Получить текущий signed prekey
    pub fn current_signed_prekey(&self) -> Result<&PrekeyStore> {
        self.current_signed_prekey
            .as_ref()
            .ok_or_else(|| ConstructError::CryptoError("No signed prekey available".to_string()))
    }

    /// Ротация signed prekey
    pub fn rotate_signed_prekey(&mut self) -> Result<()> {
        let signing_key = self.signing_key.as_ref().ok_or_else(|| {
            ConstructError::CryptoError("Signing key not initialized".to_string())
        })?;

        // Генерируем новый prekey
        let key_pair = X25519KeyPair::generate();
        let signature = signing_key.sign(key_pair.public.as_bytes());

        let key_id = self.next_prekey_id;
        self.next_prekey_id += 1;

        let prekey_store = PrekeyStore {
            key_pair,
            signature: signature.to_bytes().to_vec(),
            created_at: crate::utils::time::current_timestamp(),
            key_id,
        };

        // Сохраняем старый prekey в историю
        if let Some(old_prekey) = self.current_signed_prekey.take() {
            self.old_prekeys.insert(old_prekey.key_id, old_prekey);
        }

        self.current_signed_prekey = Some(prekey_store);

        // Очищаем старые prekeys (старше 30 дней)
        self.cleanup_old_prekeys(30 * 24 * 3600);

        Ok(())
    }

    /// Получить prekey по ID
    pub fn get_prekey(&self, key_id: u32) -> Option<&PrekeyStore> {
        if let Some(current) = &self.current_signed_prekey {
            if current.key_id == key_id {
                return Some(current);
            }
        }
        self.old_prekeys.get(&key_id)
    }

    /// Очистка старых prekeys
    fn cleanup_old_prekeys(&mut self, max_age_seconds: i64) {
        let now = crate::utils::time::current_timestamp();
        self.old_prekeys
            .retain(|_, prekey| now - prekey.created_at < max_age_seconds);
    }

    /// Экспорт регистрационного bundle
    pub fn export_registration_bundle(&self) -> Result<crate::crypto::RegistrationBundle> {
        let identity_public = self.identity_public_key()?.as_bytes().to_vec();
        let verifying_key = self.verifying_key()?.as_bytes().to_vec();
        let prekey = self.current_signed_prekey()?;

        Ok(crate::crypto::RegistrationBundle {
            identity_public,
            signed_prekey_public: prekey.key_pair.public.as_bytes().to_vec(),
            signature: prekey.signature.clone(),
            verifying_key,
        })
    }

    /// Экспорт публичного key bundle
    pub fn export_public_bundle(&self) -> Result<crate::crypto::PublicKeyBundle> {
        let identity_public = self.identity_public_key()?.as_bytes().to_vec();
        let verifying_key = self.verifying_key()?.as_bytes().to_vec();
        let prekey = self.current_signed_prekey()?;

        Ok(crate::crypto::PublicKeyBundle {
            identity_public,
            signed_prekey_public: prekey.key_pair.public.as_bytes().to_vec(),
            signature: prekey.signature.clone(),
            verifying_key,
        })
    }

    /// Подписать данные
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let signing_key = self.signing_key.as_ref().ok_or_else(|| {
            ConstructError::CryptoError("Signing key not initialized".to_string())
        })?;

        let signature = signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    /// Количество сохраненных старых prekeys
    pub fn old_prekeys_count(&self) -> usize {
        self.old_prekeys.len()
    }

    /// Получить signing key для экспорта
    pub fn signing_secret_key(&self) -> Result<&SigningKey> {
        self.signing_key
            .as_ref()
            .map(|k| &k.signing_key)
            .ok_or_else(|| ConstructError::CryptoError("Signing key not initialized".to_string()))
    }

    /// Импортировать существующие ключи
    pub fn import_keys(
        &mut self,
        identity_secret: StaticSecret,
        signing_key: SigningKey,
        prekey_secret: StaticSecret,
        prekey_signature: Vec<u8>,
    ) -> Result<()> {
        // Установить identity key
        let identity_public = PublicKey::from(&identity_secret);
        self.identity_key = Some(X25519KeyPair {
            secret: identity_secret,
            public: identity_public,
        });

        // Установить signing key
        let verifying_key = signing_key.verifying_key();
        self.signing_key = Some(Ed25519KeyPair {
            signing_key,
            verifying_key,
        });

        // Установить prekey
        let prekey_public = PublicKey::from(&prekey_secret);
        let key_id = self.next_prekey_id;
        self.next_prekey_id += 1;

        self.current_signed_prekey = Some(PrekeyStore {
            key_pair: X25519KeyPair {
                secret: prekey_secret,
                public: prekey_public,
            },
            signature: prekey_signature,
            created_at: crate::utils::time::current_timestamp(),
            key_id,
        });

        Ok(())
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_initialization() {
        let mut manager = KeyManager::new();
        assert!(manager.initialize().is_ok());
        assert!(manager.identity_public_key().is_ok());
        assert!(manager.verifying_key().is_ok());
        assert!(manager.current_signed_prekey().is_ok());
    }

    #[test]
    fn test_prekey_rotation() {
        let mut manager = KeyManager::new();
        manager.initialize().unwrap();

        let first_prekey_id = manager.current_signed_prekey().unwrap().key_id;

        manager.rotate_signed_prekey().unwrap();

        let second_prekey_id = manager.current_signed_prekey().unwrap().key_id;
        assert_ne!(first_prekey_id, second_prekey_id);

        // Старый prekey должен быть доступен
        assert!(manager.get_prekey(first_prekey_id).is_some());
    }

    #[test]
    fn test_export_bundle() {
        let mut manager = KeyManager::new();
        manager.initialize().unwrap();

        let bundle = manager.export_registration_bundle();
        assert!(bundle.is_ok());

        let bundle = bundle.unwrap();
        assert_eq!(bundle.identity_public.len(), 32);
        assert_eq!(bundle.signed_prekey_public.len(), 32);
        assert_eq!(bundle.signature.len(), 64);
        assert_eq!(bundle.verifying_key.len(), 32);
    }
}
