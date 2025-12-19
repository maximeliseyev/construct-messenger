// Главное состояние приложения

use crate::api::contacts::{Contact, ContactManager};
use crate::api::crypto::CryptoManager;
use crate::storage::models::*;
use crate::utils::error::{ConstructError, Result};
use crate::utils::time::current_timestamp;
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use crate::storage::indexeddb::IndexedDbStorage;

#[cfg(not(target_arch = "wasm32"))]
use crate::storage::memory::MemoryStorage;

use crate::state::conversations::ConversationsManager;
use crate::protocol::messages::ChatMessage;

#[cfg(target_arch = "wasm32")]
use crate::protocol::transport::WebSocketTransport;

#[cfg(target_arch = "wasm32")]
use crate::protocol::messages::ProtocolMessage;

/// Состояние подключения к серверу
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// Состояние UI
#[derive(Debug, Clone)]
pub struct UiState {
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub notification: Option<String>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            is_loading: false,
            error_message: None,
            notification: None,
        }
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn set_notification(&mut self, notification: String) {
        self.notification = Some(notification);
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Главное состояние всего приложения
pub struct AppState {
    // === Идентификация пользователя ===
    user_id: Option<String>,
    username: Option<String>,

    // === Менеджеры ===
    crypto_manager: CryptoManager,
    contact_manager: ContactManager,
    conversations_manager: ConversationsManager,

    // === Хранилище ===
    #[cfg(target_arch = "wasm32")]
    storage: IndexedDbStorage,

    #[cfg(not(target_arch = "wasm32"))]
    storage: MemoryStorage,

    // === Сетевое соединение ===
    #[cfg(target_arch = "wasm32")]
    transport: Option<WebSocketTransport>,

    // === Состояние соединения ===
    connection_state: ConnectionState,

    // === Кеш сообщений (в памяти) ===
    message_cache: HashMap<String, Vec<StoredMessage>>,

    // === Состояние UI ===
    active_conversation: Option<String>,
    ui_state: UiState,
}

impl AppState {
    /// Создать новое состояние приложения
    #[cfg(target_arch = "wasm32")]
    pub async fn new(db_name: &str) -> Result<Self> {
        let mut storage = IndexedDbStorage::new(db_name);
        storage.init().await?;

        let crypto_manager = CryptoManager::new()?;
        let contact_manager = ContactManager::new();
        let conversations_manager = ConversationsManager::new();

        Ok(Self {
            user_id: None,
            username: None,
            crypto_manager,
            contact_manager,
            conversations_manager,
            storage,
            transport: None,
            connection_state: ConnectionState::Disconnected,
            message_cache: HashMap::new(),
            active_conversation: None,
            ui_state: UiState::new(),
        })
    }

    /// Создать новое состояние приложения (non-WASM версия)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(_db_name: &str) -> Result<Self> {
        let storage = MemoryStorage::new();
        let crypto_manager = CryptoManager::new()?;
        let contact_manager = ContactManager::new();
        let conversations_manager = ConversationsManager::new();

        Ok(Self {
            user_id: None,
            username: None,
            crypto_manager,
            contact_manager,
            conversations_manager,
            storage,
            connection_state: ConnectionState::Disconnected,
            message_cache: HashMap::new(),
            active_conversation: None,
            ui_state: UiState::new(),
        })
    }

    // === Инициализация пользователя ===

    /// Инициализировать нового пользователя
    #[cfg(target_arch = "wasm32")]
    pub async fn initialize_user(&mut self, username: String, password: String) -> Result<String> {
        use crate::crypto::master_key;

        self.ui_state.set_loading(true);

        // Валидация пароля
        master_key::validate_password(&password)?;

        // 1. Генерировать user_id
        let user_id = uuid::Uuid::new_v4().to_string();

        // 2. Получить registration bundle из crypto_manager
        let _bundle = self.crypto_manager.export_registration_bundle_b64()?;

        // 3. Экспортировать и зашифровать приватные ключи
        let private_keys = self.crypto_manager.export_private_keys()?;

        // Получить подпись prekey для сохранения
        let prekey = self.crypto_manager.key_manager().current_signed_prekey()?;
        let prekey_signature = prekey.signature.clone();

        // Генерировать соль и деривировать мастер-ключ
        let salt = master_key::generate_salt();
        let master_key_derived = master_key::derive_master_key(&password, &salt)?;

        // Зашифровать приватные ключи
        let stored_keys = master_key::encrypt_private_keys(
            &private_keys,
            &master_key_derived,
            salt,
            user_id.clone(),
            prekey_signature,
        )?;

        // Сохранить зашифрованные ключи
        self.storage.save_private_keys(stored_keys).await?;

        // 4. Сохранить метаданные в storage
        let metadata = StoredAppMetadata {
            user_id: user_id.clone(),
            username: username.clone(),
            last_sync: current_timestamp(),
            settings: Vec::new(),
        };

        self.storage.save_metadata(metadata).await?;

        // 5. Установить user_id и username
        self.user_id = Some(user_id.clone());
        self.username = Some(username);

        self.ui_state.set_loading(false);
        Ok(user_id)
    }

    /// Инициализировать нового пользователя (non-WASM версия)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn initialize_user(&mut self, username: String, password: String) -> Result<String> {
        use crate::crypto::master_key;

        self.ui_state.set_loading(true);

        // Валидация пароля
        master_key::validate_password(&password)?;

        let user_id = uuid::Uuid::new_v4().to_string();
        let _bundle = self.crypto_manager.export_registration_bundle_b64()?;

        // Экспортировать и зашифровать приватные ключи
        let private_keys = self.crypto_manager.export_private_keys()?;
        let prekey = self.crypto_manager.key_manager().current_signed_prekey()?;
        let prekey_signature = prekey.signature.clone();

        let salt = master_key::generate_salt();
        let master_key_derived = master_key::derive_master_key(&password, &salt)?;

        let stored_keys = master_key::encrypt_private_keys(
            &private_keys,
            &master_key_derived,
            salt,
            user_id.clone(),
            prekey_signature,
        )?;

        self.storage.save_private_keys(stored_keys)?;

        let metadata = StoredAppMetadata {
            user_id: user_id.clone(),
            username: username.clone(),
            last_sync: current_timestamp(),
            settings: Vec::new(),
        };

        self.storage.save_metadata(metadata)?;

        self.user_id = Some(user_id.clone());
        self.username = Some(username);

        self.ui_state.set_loading(false);
        Ok(user_id)
    }

    /// Загрузить существующего пользователя
    #[cfg(target_arch = "wasm32")]
    pub async fn load_user(&mut self, user_id: String, password: String) -> Result<()> {
        use crate::crypto::master_key;

        self.ui_state.set_loading(true);

        // 1. Загрузить метаданные
        let metadata = self.storage.load_metadata(&user_id).await?
            .ok_or_else(|| ConstructError::StorageError("User not found".to_string()))?;

        self.user_id = Some(user_id.clone());
        self.username = Some(metadata.username);

        // 2. Загрузить и расшифровать приватные ключи
        let stored_keys = self.storage.load_private_keys(&user_id).await?
            .ok_or_else(|| ConstructError::StorageError("Private keys not found".to_string()))?;

        // Деривировать мастер-ключ из пароля
        let master_key_derived = master_key::derive_master_key(&password, &stored_keys.salt)?;

        // Расшифровать приватные ключи
        let private_keys = master_key::decrypt_private_keys(&stored_keys, &master_key_derived)?;

        // Импортировать в CryptoManager
        self.crypto_manager.import_private_keys(&private_keys, stored_keys.prekey_signature)?;

        // 3. Загрузить контакты
        let stored_contacts = self.storage.load_all_contacts().await?;
        for stored in stored_contacts {
            let contact = Contact {
                id: stored.id,
                username: stored.username,
                public_key_bundle: None,
                added_at: stored.added_at,
                last_message_at: stored.last_message_at,
            };
            self.contact_manager.add_contact(contact).ok();
        }

        // 4. Загрузить и восстановить сессии
        let sessions = self.storage.load_all_sessions().await?;
        for stored_session in sessions {
            self.crypto_manager
                .session_manager_mut()
                .deserialize_session(stored_session.contact_id, &stored_session.session_data)?;
        }

        self.ui_state.set_loading(false);
        Ok(())
    }

    /// Загрузить существующего пользователя (non-WASM версия)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_user(&mut self, user_id: String, password: String) -> Result<()> {
        use crate::crypto::master_key;

        self.ui_state.set_loading(true);

        let metadata = self.storage.load_metadata(&user_id)?
            .ok_or_else(|| ConstructError::StorageError("User not found".to_string()))?;

        self.user_id = Some(user_id.clone());
        self.username = Some(metadata.username);

        // Загрузить и расшифровать приватные ключи
        let stored_keys = self.storage.load_private_keys(&user_id)?
            .ok_or_else(|| ConstructError::StorageError("Private keys not found".to_string()))?;

        let master_key_derived = master_key::derive_master_key(&password, &stored_keys.salt)?;
        let private_keys = master_key::decrypt_private_keys(&stored_keys, &master_key_derived)?;

        self.crypto_manager.import_private_keys(&private_keys, stored_keys.prekey_signature)?;

        let stored_contacts = self.storage.load_all_contacts()?;
        for stored in stored_contacts {
            let contact = Contact {
                id: stored.id,
                username: stored.username,
                public_key_bundle: None,
                added_at: stored.added_at,
                last_message_at: stored.last_message_at,
            };
            self.contact_manager.add_contact(contact).ok();
        }

        // Загрузить и восстановить сессии
        let sessions = self.storage.load_all_sessions()?;
        for stored_session in sessions {
            self.crypto_manager
                .session_manager_mut()
                .deserialize_session(stored_session.contact_id, &stored_session.session_data)?;
        }

        self.ui_state.set_loading(false);
        Ok(())
    }

    // === Управление контактами ===

    /// Добавить контакт
    #[cfg(target_arch = "wasm32")]
    pub async fn add_contact(&mut self, contact_id: String, username: String) -> Result<()> {
        // 1. Добавить в ContactManager
        let contact = crate::api::contacts::create_contact(contact_id.clone(), username.clone());
        self.contact_manager.add_contact(contact)?;

        // 2. Сохранить в storage
        let stored = StoredContact {
            id: contact_id,
            username,
            public_key_bundle: None,
            added_at: current_timestamp(),
            last_message_at: None,
        };
        self.storage.save_contact(stored).await?;

        Ok(())
    }

    /// Добавить контакт (non-WASM версия)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn add_contact(&mut self, contact_id: String, username: String) -> Result<()> {
        let contact = crate::api::contacts::create_contact(contact_id.clone(), username.clone());
        self.contact_manager.add_contact(contact)?;

        let stored = StoredContact {
            id: contact_id,
            username,
            public_key_bundle: None,
            added_at: current_timestamp(),
            last_message_at: None,
        };
        self.storage.save_contact(stored)?;

        Ok(())
    }

    /// Получить все контакты
    pub fn get_contacts(&self) -> Vec<&Contact> {
        self.contact_manager.get_all_contacts()
    }

    // === Работа с сообщениями ===

    /// Отправить сообщение
    #[cfg(target_arch = "wasm32")]
    pub async fn send_message(&mut self, to_contact_id: &str, session_id: &str, plaintext: &str) -> Result<String> {
        let user_id = self.user_id.as_ref()
            .ok_or_else(|| ConstructError::SessionError("User not initialized".to_string()))?;

        // 1. Проверить наличие сессии
        if !self.crypto_manager.has_session(to_contact_id) {
            return Err(ConstructError::SessionError(format!(
                "No session with contact {}. Please initialize session first.", to_contact_id
            )));
        }

        // 2. Зашифровать сообщение
        let encrypted_bytes = self.crypto_manager.encrypt_message(session_id, plaintext)?;

        // Десериализовать для извлечения метаданных
        let encrypted_msg: crate::crypto::double_ratchet::EncryptedRatchetMessage =
            bincode::deserialize(&encrypted_bytes)
                .map_err(|e| ConstructError::SerializationError(e.to_string()))?;

        let msg_id = uuid::Uuid::new_v4().to_string();
        let timestamp = current_timestamp();

        // Сохранить encrypted_content для storage
        let encrypted_content = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &encrypted_bytes);

        // 3. Создать ChatMessage для отправки
        let chat_msg = ChatMessage {
            id: msg_id.clone(),
            from: user_id.clone(),
            to: to_contact_id.to_string(),
            ephemeral_public_key: encrypted_msg.dh_public_key.to_vec(),
            message_number: encrypted_msg.message_number,
            content: encrypted_content.clone(),
            timestamp,
        };

        // 4. Упаковать в ProtocolMessage и отправить через WebSocket
        let protocol_msg = ProtocolMessage::SendMessage(chat_msg);

        if let Some(transport) = &self.transport {
            if !transport.is_connected() {
                return Err(ConstructError::NetworkError("Not connected to server".to_string()));
            }

            transport.send(&protocol_msg)?;
        } else {
            return Err(ConstructError::NetworkError("Transport not initialized".to_string()));
        }

        // 5. Сохранить в storage как Sent
        let stored_msg = StoredMessage {
            id: msg_id.clone(),
            conversation_id: to_contact_id.to_string(),
            from: user_id.clone(),
            to: to_contact_id.to_string(),
            encrypted_content,
            timestamp,
            status: MessageStatus::Sent,
        };

        self.storage.save_message(stored_msg.clone()).await?;

        // 6. Обновить кеш и conversations manager
        self.conversations_manager.add_message(to_contact_id, stored_msg.clone());
        self.update_message_cache(to_contact_id, stored_msg).await?;

        Ok(msg_id)
    }

    /// Отправить сообщение (non-WASM версия)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn send_message(&mut self, to_contact_id: &str, _session_id: &str, plaintext: &str) -> Result<String> {
        let user_id = self.user_id.as_ref()
            .ok_or_else(|| ConstructError::SessionError("User not initialized".to_string()))?;

        let msg_id = uuid::Uuid::new_v4().to_string();
        let stored_msg = StoredMessage {
            id: msg_id.clone(),
            conversation_id: to_contact_id.to_string(),
            from: user_id.clone(),
            to: to_contact_id.to_string(),
            encrypted_content: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, plaintext.as_bytes()),
            timestamp: current_timestamp(),
            status: MessageStatus::Pending,
        };

        self.storage.save_message(stored_msg.clone())?;
        self.conversations_manager.add_message(to_contact_id, stored_msg.clone());

        let cache = self.message_cache
            .entry(to_contact_id.to_string())
            .or_insert_with(Vec::new);
        cache.push(stored_msg);

        Ok(msg_id)
    }

    /// Обработать входящее сообщение
    #[cfg(target_arch = "wasm32")]
    pub async fn receive_message(&mut self, chat_msg: ChatMessage, session_id: &str) -> Result<()> {
        // 1. Декодировать и расшифровать сообщение
        let encrypted_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &chat_msg.content
        ).map_err(|e| ConstructError::SerializationError(format!("Invalid base64: {}", e)))?;

        let plaintext = self.crypto_manager.decrypt_message(session_id, &encrypted_bytes)?;

        // 2. Сохранить в storage
        let stored_msg = StoredMessage {
            id: chat_msg.id.clone(),
            conversation_id: chat_msg.from.clone(),
            from: chat_msg.from.clone(),
            to: chat_msg.to.clone(),
            encrypted_content: chat_msg.content.clone(),
            timestamp: chat_msg.timestamp,
            status: MessageStatus::Delivered,
        };

        self.storage.save_message(stored_msg.clone()).await?;

        // 3. Обновить кеш
        self.conversations_manager.add_message(&chat_msg.from, stored_msg.clone());
        self.update_message_cache(&chat_msg.from, stored_msg).await?;

        // 4. Инкрементировать unread_count если беседа не активна
        if self.active_conversation.as_deref() != Some(&chat_msg.from) {
            if let Some(conversation) = self.conversations_manager.get_mut(&chat_msg.from) {
                conversation.increment_unread();
            }
        }

        // TODO: 5. Уведомить UI через callback или event

        Ok(())
    }

    /// Обработать входящее сообщение (non-WASM заглушка)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn receive_message(&mut self, _chat_msg: ChatMessage, _session_id: &str) -> Result<()> {
        Ok(())
    }

    /// Обновить кеш сообщений
    #[cfg(target_arch = "wasm32")]
    async fn update_message_cache(&mut self, conversation_id: &str, msg: StoredMessage) -> Result<()> {
        let cache = self.message_cache
            .entry(conversation_id.to_string())
            .or_insert_with(Vec::new);
        cache.push(msg);
        cache.sort_by_key(|m| m.timestamp);
        Ok(())
    }

    /// Загрузить беседу
    #[cfg(target_arch = "wasm32")]
    pub async fn load_conversation(&mut self, contact_id: &str) -> Result<Vec<StoredMessage>> {
        // 1. Проверить кеш
        if let Some(messages) = self.message_cache.get(contact_id) {
            return Ok(messages.clone());
        }

        // 2. Загрузить из storage
        let messages = self.storage
            .load_messages_for_conversation(contact_id, 50, 0)
            .await?;

        // 3. Закешировать
        self.message_cache.insert(contact_id.to_string(), messages.clone());

        // 4. Обновить conversations manager
        for msg in &messages {
            self.conversations_manager.add_message(contact_id, msg.clone());
        }

        Ok(messages)
    }

    /// Загрузить беседу (non-WASM версия)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_conversation(&mut self, contact_id: &str) -> Result<Vec<StoredMessage>> {
        if let Some(messages) = self.message_cache.get(contact_id) {
            return Ok(messages.clone());
        }

        let messages = self.storage
            .load_messages_for_conversation(contact_id, 50, 0)?;

        self.message_cache.insert(contact_id.to_string(), messages.clone());

        for msg in &messages {
            self.conversations_manager.add_message(contact_id, msg.clone());
        }

        Ok(messages)
    }

    /// Установить активную беседу
    pub fn set_active_conversation(&mut self, contact_id: Option<String>) {
        self.active_conversation = contact_id;
    }

    /// Получить активную беседу
    pub fn get_active_conversation(&self) -> Option<&str> {
        self.active_conversation.as_deref()
    }

    // === Управление соединением ===

    /// Подключиться к серверу WebSocket
    #[cfg(target_arch = "wasm32")]
    pub fn connect(&mut self, server_url: &str) -> Result<()> {
        if self.connection_state == ConnectionState::Connected {
            return Err(ConstructError::NetworkError("Already connected".to_string()));
        }

        self.connection_state = ConnectionState::Connecting;

        let mut transport = WebSocketTransport::new(server_url);
        transport.connect()?;

        // Настроить базовые callbacks
        self.setup_transport_callbacks(&mut transport)?;

        self.transport = Some(transport);
        self.connection_state = ConnectionState::Connected;

        Ok(())
    }

    /// Настроить WebSocket callbacks
    /// ПРИМЕЧАНИЕ: Полная интеграция с AppState требует использования Arc<Mutex<AppState>>
    /// Здесь мы создаем базовые callbacks для демонстрации структуры
    #[cfg(target_arch = "wasm32")]
    fn setup_transport_callbacks(&self, transport: &mut WebSocketTransport) -> Result<()> {
        use crate::wasm::console;

        // Callback для успешного подключения
        transport.set_on_open(|| {
            console::log("WebSocket connected successfully");
        })?;

        // Callback для входящих сообщений
        // TODO: Для полной реализации нужно использовать Arc<Mutex<AppState>>
        // чтобы иметь возможность вызывать receive_message из замыкания
        transport.set_on_message(|msg| {
            // Простая обработка для демонстрации
            console::log(&format!("Received message: {:?}", msg));

            // TODO: Полная реализация должна выглядеть так:
            // if let ProtocolMessage::ReceiveMessage(chat_msg) = msg {
            //     // Найти session_id для контакта
            //     // app_state.receive_message(chat_msg, session_id).await
            // }
        })?;

        // Callback для ошибок
        transport.set_on_error(|err| {
            console::log(&format!("WebSocket error: {}", err));
        })?;

        // Callback для закрытия соединения
        transport.set_on_close(|code, reason| {
            console::log(&format!("WebSocket closed: {} - {}", code, reason));
        })?;

        Ok(())
    }

    /// Подключиться к серверу (non-WASM заглушка)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn connect(&mut self, _server_url: &str) -> Result<()> {
        Err(ConstructError::NetworkError("WebSocket only available in WASM".to_string()))
    }

    /// Отключиться от сервера
    #[cfg(target_arch = "wasm32")]
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(transport) = &mut self.transport {
            transport.close()?;
        }

        self.transport = None;
        self.connection_state = ConnectionState::Disconnected;

        Ok(())
    }

    /// Отключиться от сервера (non-WASM заглушка)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn disconnect(&mut self) -> Result<()> {
        self.connection_state = ConnectionState::Disconnected;
        Ok(())
    }

    /// Установить состояние соединения
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state;
    }

    /// Получить состояние соединения
    pub fn connection_state(&self) -> ConnectionState {
        self.connection_state
    }

    /// Проверить, подключен ли к серверу
    pub fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    // === Геттеры для UI ===

    pub fn get_user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    pub fn get_username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn ui_state(&self) -> &UiState {
        &self.ui_state
    }

    pub fn ui_state_mut(&mut self) -> &mut UiState {
        &mut self.ui_state
    }

    pub fn crypto_manager(&self) -> &CryptoManager {
        &self.crypto_manager
    }

    pub fn crypto_manager_mut(&mut self) -> &mut CryptoManager {
        &mut self.crypto_manager
    }

    pub fn conversations_manager(&self) -> &ConversationsManager {
        &self.conversations_manager
    }

    pub fn conversations_manager_mut(&mut self) -> &mut ConversationsManager {
        &mut self.conversations_manager
    }

    // === Очистка ===

    /// Очистить все данные
    #[cfg(target_arch = "wasm32")]
    pub async fn clear_all_data(&mut self) -> Result<()> {
        // Очистить кеши
        self.message_cache.clear();
        self.conversations_manager.clear_all();
        self.contact_manager.clear_all();

        // Сбросить состояние
        self.user_id = None;
        self.username = None;
        self.active_conversation = None;
        self.connection_state = ConnectionState::Disconnected;

        // TODO: Очистить IndexedDB полностью

        Ok(())
    }

    /// Очистить все данные (non-WASM версия)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear_all_data(&mut self) -> Result<()> {
        self.message_cache.clear();
        self.conversations_manager.clear_all();
        self.contact_manager.clear_all();
        self.storage.clear_all()?;

        self.user_id = None;
        self.username = None;
        self.active_conversation = None;
        self.connection_state = ConnectionState::Disconnected;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_app_state_creation() {
        let state = AppState::new("test_db");
        assert!(state.is_ok());

        let state = state.unwrap();
        assert!(state.get_user_id().is_none());
        assert_eq!(state.connection_state(), ConnectionState::Disconnected);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_app_state_initialize_user() {
        let mut state = AppState::new("test_db").unwrap();
        let user_id = state.initialize_user("alice".to_string(), "testpass123".to_string()).unwrap();

        assert!(!user_id.is_empty());
        assert_eq!(state.get_username(), Some("alice"));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_app_state_contacts() {
        let mut state = AppState::new("test_db").unwrap();
        state.initialize_user("alice".to_string(), "testpass123".to_string()).unwrap();

        state.add_contact("contact1".to_string(), "bob".to_string()).unwrap();

        let contacts = state.get_contacts();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].username, "bob");
    }
}
