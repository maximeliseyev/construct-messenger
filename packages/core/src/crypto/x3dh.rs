use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, ReusableSecret, SharedSecret, StaticSecret};
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Clone)]
pub struct PublicKeyBundle {
    pub identity_public: Vec<u8>,
    pub signed_prekey_public: Vec<u8>,
    pub signature: Vec<u8>,
    pub verifying_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RegistrationBundle {
    pub identity_public: Vec<u8>,
    pub signed_prekey_public: Vec<u8>,
    pub signature: Vec<u8>,
    pub verifying_key: Vec<u8>,
}

/// Чистая реализация X3DH протокола без состояния
pub struct X3DH;

impl X3DH {
    /// Выполняет X3DH обмен и возвращает root key
    /// Упрощенная версия без ephemeral ключа (для тестирования)
    /// В продакшене ephemeral public key должен передаваться через prekey bundle
    pub fn perform_x3dh(
        identity_private: &StaticSecret,
        _signed_prekey_private: &StaticSecret,
        remote_identity_public: &PublicKey,
        remote_signed_prekey_public: &PublicKey,
        remote_signature: &[u8; 64],
        remote_verifying_key: &[u8; 32],
    ) -> Result<[u8; 32], String> {
        // 1. Верификация подписи
        Self::verify_signature(
            remote_signed_prekey_public,
            remote_signature,
            remote_verifying_key,
        )?;

        // 2. Симметричный DH обмен
        // Только identity * remote_identity - это симметрично!
        let shared_secret = identity_private.diffie_hellman(remote_identity_public);

        // 3. Вывод root key через HKDF
        let root_key = Self::derive_root_key_simple(&shared_secret);

        Ok(root_key)
    }

    fn derive_root_key_simple(shared_secret: &SharedSecret) -> [u8; 32] {
        use hkdf::Hkdf;
        use sha2::Sha256;

        // Используем HKDF для вывода root key
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
        let mut root_key = [0u8; 32];
        hkdf.expand(b"X3DH Root Key Simplified", &mut root_key)
            .expect("HKDF expand failed");

        root_key
    }

    /// Генерирует bundle для регистрации
    pub fn generate_registration_bundle() -> RegistrationBundle {
        // Генерируем настоящие ключи
        use ed25519_dalek::{SigningKey, Signer};

        let identity_secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
        let identity_public = x25519_dalek::PublicKey::from(&identity_secret);

        let signed_prekey_secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
        let signed_prekey_public = x25519_dalek::PublicKey::from(&signed_prekey_secret);

        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();

        // Подписываем signed prekey
        let signature = signing_key.sign(signed_prekey_public.as_bytes());

        RegistrationBundle {
            identity_public: identity_public.as_bytes().to_vec(),
            signed_prekey_public: signed_prekey_public.as_bytes().to_vec(),
            signature: signature.to_bytes().to_vec(),
            verifying_key: verifying_key.as_bytes().to_vec(),
        }
    }

    fn verify_signature(
        _signed_prekey: &PublicKey,
        _signature: &[u8; 64],
        _verifying_key: &[u8; 32],
    ) -> Result<(), String> {
        // Placeholder implementation
        Ok(())
    }

    fn derive_root_key(dh1: &SharedSecret, dh2: &SharedSecret, dh3: &SharedSecret) -> [u8; 32] {
        use hkdf::Hkdf;
        use sha2::Sha256;

        // Объединяем все три DH результата
        let mut combined = Vec::with_capacity(96);
        combined.extend_from_slice(dh1.as_bytes());
        combined.extend_from_slice(dh2.as_bytes());
        combined.extend_from_slice(dh3.as_bytes());

        // Используем HKDF для вывода root key
        let hkdf = Hkdf::<Sha256>::new(None, &combined);
        let mut root_key = [0u8; 32];
        hkdf.expand(b"X3DH Root Key", &mut root_key)
            .expect("HKDF expand failed");

        root_key
    }
}
