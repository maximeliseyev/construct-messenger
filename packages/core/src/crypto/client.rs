use crate::crypto::double_ratchet::{DoubleRatchetSession, EncryptedRatchetMessage, SerializableSession};
use crate::utils;
use crate::crypto::x3dh::{PublicKeyBundle, RegistrationBundle, X3DH};
use x25519_dalek::PublicKey;

#[cfg(feature = "post-quantum")]
use pqcrypto_kyber::{keypair as kyber_keypair, encapsulate};
#[cfg(feature = "post-quantum")]
use pqcrypto_dilithium::{keypair as dilithium_keypair, sign};
#[cfg(feature = "post-quantum")]
use crate::crypto::pq_x3dh::PQX3DHBundle;


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

    // Post-Quantum keys (conditionally compiled)
    #[cfg(feature = "post-quantum")]
    kyber_secret: pqcrypto_kyber::SecretKey,
    #[cfg(feature = "post-quantum")]
    kyber_prekey_secret: pqcrypto_kyber::SecretKey,
    #[cfg(feature = "post-quantum")]
    dilithium_secret: pqcrypto_dilithium::SecretKey,
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

    #[cfg(feature = "post-quantum")]
    pub fn new_with_pqc() -> Result<Self, String> {
        // Классические ключи
        let identity_key = x25519_dalek::StaticSecret::random_from_rng(rand::rngs::OsRng);
        let signed_prekey = x25519_dalek::StaticSecret::random_from_rng(rand::rngs::OsRng);
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        
        // Пост-квантовые ключи
        let (_, kyber_sk) = kyber_keypair().map_err(|e| e.to_string())?;
        let (_, kyber_prekey_sk) = kyber_keypair().map_err(|e| e.to_string())?;
        let (_, dilithium_sk) = dilithium_keypair().map_err(|e| e.to_string())?;
        
        Ok(Self {
            identity_key,
            signed_prekey,
            signing_key,
            sessions: std::collections::HashMap::new(),
            storage: None,
            kyber_secret: kyber_sk,
            kyber_prekey_secret: kyber_prekey_sk,
            dilithium_secret: dilithium_sk,
        })
    }
    
    #[cfg(feature = "post-quantum")]
    pub fn perform_pq_x3dh(&self, remote_bundle: &PQX3DHBundle) -> Result<[u8; 64], String> {
        // This is a placeholder as per the markdown, and needs more implementation details
        
        // 1. Классический X3DH (needs to be implemented correctly)
        // let classical_secret = X3DH::perform_x3dh(...)
        unimplemented!("Classical part of PQX3DH is not implemented yet");

        // 2. Пост-квантовый обмен (needs proper key conversion and error handling)
        // let public_key = pqcrypto_kyber::PublicKey::from_bytes(&remote_bundle.kyber_public_key)?;
        // let (kyber_ciphertext, kyber_shared) = encapsulate(&public_key)?;
        // ... and for the prekey ...
        unimplemented!("Post-quantum part of PQX3DH is not implemented yet");
        
        // 3. Комбинируем через HKDF
        // let combined = ...;
        // let final_key = ...;
        // Ok(final_key)
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
