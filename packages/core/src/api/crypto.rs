// API для криптографических операций

use crate::crypto::ClientCrypto;
use crate::crypto::x3dh::PublicKeyBundle;
use crate::utils::error::Result;
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

/// Создать новый crypto клиент
pub fn create_client() -> Result<ClientCrypto> {
    ClientCrypto::new().map_err(|e| crate::utils::error::MessengerError::CryptoError(e))
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
        .map_err(|e| crate::utils::error::MessengerError::SerializationError(e.to_string()))
}

/// Десериализовать KeyBundle из JSON
pub fn deserialize_key_bundle(json: &str) -> Result<KeyBundle> {
    serde_json::from_str(json)
        .map_err(|e| crate::utils::error::MessengerError::SerializationError(e.to_string()))
}
