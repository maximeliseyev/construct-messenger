use crate::crypto::double_ratchet::{DoubleRatchetSession, EncryptedRatchetMessage, SerializableSession};
use crate::utils;
use crate::crypto::x3dh::{PublicKeyBundle, RegistrationBundle, X3DH};
use x25519_dalek::PublicKey;

/// Главный клиент, объединяющий все компоненты
pub struct ClientCrypto {
    // X3DH ключи
    identity_key: x25519_dalek::StaticSecret,
    signed_prekey: x25519_dalek::StaticSecret,
    signing_key: ed25519_dalek::SigningKey,

    // Double Ratchet сессии
    sessions: std::collections::HashMap<String, DoubleRatchetSession>,

    // Хранилище
    storage: Option<crate::storage::KeyStorage>,
}

impl ClientCrypto {
    pub fn new() -> Result<Self, String> {
        let identity_key = x25519_dalek::StaticSecret::random_from_rng(rand::rngs::OsRng);
        let signed_prekey = x25519_dalek::StaticSecret::random_from_rng(rand::rngs::OsRng);
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);

        Ok(Self {
            identity_key,
            signed_prekey,
            signing_key,
            sessions: std::collections::HashMap::new(),
            storage: None,
        })
    }

    /// Регистрация - делегируем X3DH
    pub fn get_registration_bundle(&self) -> RegistrationBundle {
        X3DH::generate_registration_bundle()
    }

    /// Инициализация сессии - используем X3DH + Double Ratchet
    pub fn init_session(
        &mut self,
        contact_id: &str,
        remote_bundle: &PublicKeyBundle,
    ) -> Result<String, String> {
        let identity_public = PublicKey::from(<[u8; 32]>::try_from(remote_bundle.identity_public.as_slice()).map_err(|_| "Invalid identity public key")?);
        let signed_prekey_public = PublicKey::from(<[u8; 32]>::try_from(remote_bundle.signed_prekey_public.as_slice()).map_err(|_| "Invalid signed prekey public key")?);
        let signature = <[u8; 64]>::try_from(remote_bundle.signature.as_slice()).map_err(|_| "Invalid signature")?;
        let verifying_key = <[u8; 32]>::try_from(remote_bundle.verifying_key.as_slice()).map_err(|_| "Invalid verifying key")?;
        // 1. X3DH handshake
        let root_key = X3DH::perform_x3dh(
            &self.identity_key,
            &self.signed_prekey,
            &identity_public,
            &signed_prekey_public,
            &signature,
            &verifying_key,
        )?;

        // 2. Создание Double Ratchet сессии
        let session = DoubleRatchetSession::new_x3dh_session(
            root_key,
            identity_public,
            &self.identity_key,
            contact_id.to_string(),
        )?;

        let session_id = utils::uuid::generate_v4();
        self.sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    pub fn init_double_ratchet_session(&mut self, contact_id: &str, remote_bundle: &PublicKeyBundle) -> Result<String, String> {
        self.init_session(contact_id, remote_bundle)
    }

    pub fn encrypt_ratchet_message(&mut self, session_id: &str, plaintext: &[u8]) -> Result<EncryptedRatchetMessage, String> {
        let session = self.sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        session.encrypt(plaintext)
    }

    pub fn decrypt_ratchet_message(&mut self, session_id: &str, encrypted: &EncryptedRatchetMessage) -> Result<Vec<u8>, String> {
        let session = self.sessions
            .get_mut(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        session.decrypt(encrypted)
    }

    pub fn export_session(&self, session_id: &str) -> Result<Vec<u8>, String> {
        let session = self.sessions
            .get(session_id)
            .ok_or_else(|| format!("Session not found: {}", session_id))?;

        let serializable = session.to_serializable();
        utils::serialization::to_bytes(&serializable)
    }

    pub fn restore_session(&mut self, session_data: &[u8]) -> Result<String, String> {
        let serializable: SerializableSession = utils::serialization::from_bytes(session_data)?;
        let session = DoubleRatchetSession::from_serializable(serializable)?;
        let session_id = utils::uuid::generate_v4();

        self.sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    // ... остальные методы используют модули
}
