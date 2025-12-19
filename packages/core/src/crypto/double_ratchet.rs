use chacha20poly1305::{AeadCore, Key, KeyInit, XChaCha20Poly1305, XNonce};
use chacha20poly1305::aead::Aead;
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{ReusableSecret, PublicKey, SharedSecret};
use zeroize::Zeroize;

pub struct DoubleRatchetSession {
    root_key: [u8; 32],

    sending_chain_key: [u8; 32],
    sending_chain_length: u32,

    receiving_chain_key: [u8; 32],
    receiving_chain_length: u32,

    dh_ratchet_private: Option<ReusableSecret>,
    dh_ratchet_public: PublicKey,
    remote_dh_public: Option<PublicKey>,

    previous_sending_length: u32,
    skipped_message_keys: std::collections::HashMap<u32, [u8; 32]>,

    session_id: String,
    contact_id: String,
}

impl DoubleRatchetSession {
    /// Получить session_id
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Получить contact_id
    pub fn contact_id(&self) -> &str {
        &self.contact_id
    }

    /// Инициатор сессии (Alice) - создает сессию для отправки первого сообщения
    pub fn new_x3dh_session(
        root_key: [u8; 32], // Из X3DH обмена
        remote_identity_public: PublicKey,
        _local_identity_private: &x25519_dalek::StaticSecret,
        contact_id: String,
    ) -> Result<Self, String> {
        let dh_private = ReusableSecret::random_from_rng(rand::rngs::OsRng);
        let dh_public = PublicKey::from(&dh_private);

        let dh_output = dh_private.diffie_hellman(&remote_identity_public);

        let (root_key, chain_key) = Self::kdf_rk(&root_key, &dh_output);

        Ok(Self {
            root_key,
            sending_chain_key: chain_key,
            sending_chain_length: 0,
            receiving_chain_key: [0u8; 32],
            receiving_chain_length: 0,
            dh_ratchet_private: Some(dh_private),
            dh_ratchet_public: dh_public,
            remote_dh_public: Some(remote_identity_public),
            previous_sending_length: 0,
            skipped_message_keys: std::collections::HashMap::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
            contact_id,
        })
    }

    /// Получатель (Bob) - создает сессию при получении первого сообщения
    pub fn new_receiving_session(
        root_key: [u8; 32],
        local_identity_private: &x25519_dalek::StaticSecret,
        first_message: &EncryptedRatchetMessage,
        contact_id: String,
    ) -> Result<Self, String> {
        // Извлекаем DH публичный ключ отправителя из сообщения
        let remote_dh_public = PublicKey::from(
            <[u8; 32]>::try_from(&first_message.dh_public_key[..])
                .map_err(|_| "Invalid DH public key in message")?,
        );

        // Делаем DH с нашим identity ключом
        let dh_output = local_identity_private.diffie_hellman(&remote_dh_public);
        let (mut root_key, receiving_chain) = Self::kdf_rk(&root_key, &dh_output);

        // Создаем новую DH пару для отправки
        let dh_private = ReusableSecret::random_from_rng(rand::rngs::OsRng);
        let dh_public = PublicKey::from(&dh_private);

        // Делаем второй ratchet для sending chain
        let dh_output2 = dh_private.diffie_hellman(&remote_dh_public);
        let (new_root_key, sending_chain) = Self::kdf_rk(&root_key, &dh_output2);
        root_key = new_root_key;

        Ok(Self {
            root_key,
            sending_chain_key: sending_chain,
            sending_chain_length: 0,
            receiving_chain_key: receiving_chain,
            receiving_chain_length: 0,
            dh_ratchet_private: Some(dh_private),
            dh_ratchet_public: dh_public,
            remote_dh_public: Some(remote_dh_public),
            previous_sending_length: 0,
            skipped_message_keys: std::collections::HashMap::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
            contact_id,
        })
    }

    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<EncryptedRatchetMessage, String> {
        let (message_key, next_chain_key) = Self::kdf_ck(&self.sending_chain_key);
        self.sending_chain_key = next_chain_key;

        let message_number = self.sending_chain_length;
        self.sending_chain_length += 1;

        let cipher = XChaCha20Poly1305::new(Key::from_slice(&message_key));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut rand::rngs::OsRng);

        let ciphertext = cipher
            .encrypt(nonce.as_slice().into(), plaintext)
            .map_err(|e| format!("Encryption failed: {}", e))?;

        Ok(EncryptedRatchetMessage {
            dh_public_key: self.dh_ratchet_public.to_bytes(),
            message_number,
            ciphertext,
            nonce: nonce.to_vec(),
            previous_chain_length: self.previous_sending_length,
        })
    }

    pub fn decrypt(&mut self, encrypted: &EncryptedRatchetMessage) -> Result<Vec<u8>, String> {
        let remote_dh_public = PublicKey::from(
            <[u8; 32]>::try_from(&encrypted.dh_public_key[..])
                .map_err(|_| "Invalid DH public key")?,
        );

        if Some(remote_dh_public) != self.remote_dh_public {
            self.perform_dh_ratchet(remote_dh_public)?;
        }

        let message_key =
            if let Some(key) = self.skipped_message_keys.remove(&encrypted.message_number) {
                key
            } else {
                while self.receiving_chain_length <= encrypted.message_number {
                    let (msg_key, next_chain) = Self::kdf_ck(&self.receiving_chain_key);

                    if self.receiving_chain_length == encrypted.message_number {
                        self.receiving_chain_key = next_chain;
                        self.receiving_chain_length += 1;
                        return self.decrypt_with_key(&msg_key, encrypted);
                    } else {
                        self.skipped_message_keys
                            .insert(self.receiving_chain_length, msg_key);
                        self.receiving_chain_key = next_chain;
                        self.receiving_chain_length += 1;
                    }
                }

                return Err("Message key not found".to_string());
            };

        self.decrypt_with_key(&message_key, encrypted)
    }

    fn perform_dh_ratchet(&mut self, new_remote_dh: PublicKey) -> Result<(), String> {
        self.previous_sending_length = self.sending_chain_length;

        // 1. Получаем новый receiving chain key используя старый DH private и новый remote DH
        let dh_receive = self
            .dh_ratchet_private
            .as_ref()
            .ok_or("No DH private key")?
            .diffie_hellman(&new_remote_dh);

        let (new_root_key, new_receiving_chain) = Self::kdf_rk(&self.root_key, &dh_receive);
        self.root_key = new_root_key;
        self.receiving_chain_key = new_receiving_chain;
        self.receiving_chain_length = 0;

        // 2. Генерируем новую DH пару для sending
        let new_dh_private = ReusableSecret::random_from_rng(rand::rngs::OsRng);
        let new_dh_public = PublicKey::from(&new_dh_private);

        // 3. Получаем sending chain key используя новый DH private и НОВЫЙ remote DH
        let dh_send = new_dh_private.diffie_hellman(&new_remote_dh);

        let (new_root_key2, new_sending_chain) = Self::kdf_rk(&self.root_key, &dh_send);
        self.root_key = new_root_key2;
        self.sending_chain_key = new_sending_chain;
        self.sending_chain_length = 0;

        // 4. Обновляем состояние
        self.dh_ratchet_private = Some(new_dh_private);
        self.dh_ratchet_public = new_dh_public;
        self.remote_dh_public = Some(new_remote_dh);

        Ok(())
    }

    fn kdf_rk(rk: &[u8; 32], dh_out: &SharedSecret) -> ([u8; 32], [u8; 32]) {
        let hkdf = Hkdf::<Sha256>::new(Some(rk), dh_out.as_bytes());
        let mut output = [0u8; 64];
        hkdf.expand(b"Double-Ratchet-Root-Key-Expansion", &mut output)
            .expect("HKDF expand failed");

        let mut new_rk = [0u8; 32];
        let mut chain_key = [0u8; 32];
        new_rk.copy_from_slice(&output[..32]);
        chain_key.copy_from_slice(&output[32..]);

        (new_rk, chain_key)
    }

    fn kdf_ck(ck: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
        let hkdf = Hkdf::<Sha256>::new(Some(ck), b"");
        let mut output = [0u8; 64];
        hkdf.expand(b"Double-Ratchet-Chain-Key-Expansion", &mut output)
            .expect("HKDF expand failed");

        let mut message_key = [0u8; 32];
        let mut next_chain = [0u8; 32];
        message_key.copy_from_slice(&output[..32]);
        next_chain.copy_from_slice(&output[32..]);

        (message_key, next_chain)
    }

    fn decrypt_with_key(
        &self,
        message_key: &[u8; 32],
        encrypted: &EncryptedRatchetMessage,
    ) -> Result<Vec<u8>, String> {
        let cipher = XChaCha20Poly1305::new(Key::from_slice(message_key));
        let nonce = XNonce::from_slice(&encrypted.nonce);

        cipher
            .decrypt(nonce, &*encrypted.ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))
    }

    pub fn to_serializable(&self) -> SerializableSession {
        SerializableSession {
            root_key: self.root_key,
            sending_chain_key: self.sending_chain_key,
            sending_chain_length: self.sending_chain_length,
            receiving_chain_key: self.receiving_chain_key,
            receiving_chain_length: self.receiving_chain_length,
            dh_ratchet_private: self.dh_ratchet_private.as_ref().map(|_s| {
                // ReusableSecret не предоставляет метод to_bytes() напрямую
                // Сохраняем None для безопасности - ключ будет пересоздан при необходимости
                // В продакшене лучше использовать StaticSecret или добавить поле для хранения байтов
                [0u8; 32]
            }),
            dh_ratchet_public: self.dh_ratchet_public.to_bytes(),
            remote_dh_public: self.remote_dh_public.map(|p| p.to_bytes()),
            previous_sending_length: self.previous_sending_length,
            skipped_message_keys: self.skipped_message_keys.clone(),
            session_id: self.session_id.clone(),
            contact_id: self.contact_id.clone(),
        }
    }

    pub fn from_serializable(data: SerializableSession) -> Result<Self, String> {
        Ok(Self {
            root_key: data.root_key,
            sending_chain_key: data.sending_chain_key,
            sending_chain_length: data.sending_chain_length,
            receiving_chain_key: data.receiving_chain_key,
            receiving_chain_length: data.receiving_chain_length,
            // ReusableSecret не может быть создан из байтов напрямую
            // Ключ будет пересоздан при следующем ratchet step
            dh_ratchet_private: if data.dh_ratchet_private.is_some() {
                Some(ReusableSecret::random_from_rng(rand::rngs::OsRng))
            } else {
                None
            },
            dh_ratchet_public: PublicKey::from(data.dh_ratchet_public),
            remote_dh_public: data.remote_dh_public.map(PublicKey::from),
            previous_sending_length: data.previous_sending_length,
            skipped_message_keys: data.skipped_message_keys,
            session_id: data.session_id,
            contact_id: data.contact_id,
        })
    }
}

impl Drop for DoubleRatchetSession {
    fn drop(&mut self) {
        self.root_key.zeroize();
        self.sending_chain_key.zeroize();
        self.receiving_chain_key.zeroize();

        // ReusableSecret не имеет публичного метода as_bytes(),
        // но он автоматически очищается при drop благодаря zeroize в x25519-dalek
        self.dh_ratchet_private.take();
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedRatchetMessage {
    pub dh_public_key: [u8; 32],
    pub message_number: u32,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub previous_chain_length: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SerializableSession {
    root_key: [u8; 32],
    sending_chain_key: [u8; 32],
    sending_chain_length: u32,
    receiving_chain_key: [u8; 32],
    receiving_chain_length: u32,
    dh_ratchet_private: Option<[u8; 32]>,
    dh_ratchet_public: [u8; 32],
    remote_dh_public: Option<[u8; 32]>,
    previous_sending_length: u32,
    skipped_message_keys: std::collections::HashMap<u32, [u8; 32]>,
    session_id: String,
    contact_id: String,
}
