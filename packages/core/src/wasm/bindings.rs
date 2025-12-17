// WASM bindings

use wasm_bindgen::prelude::*;
use crate::crypto::ClientCrypto;
use crate::api::{crypto, messaging};
use std::cell::RefCell;
use std::collections::HashMap;

// Глобальное хранилище клиентов
thread_local! {
    static CLIENTS: RefCell<HashMap<String, ClientCrypto>> = RefCell::new(HashMap::new());
}

/// Создать нового криптографического клиента
#[wasm_bindgen]
pub fn create_crypto_client() -> Result<String, JsValue> {
    let client = crypto::create_client()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let client_id = uuid::Uuid::new_v4().to_string();

    CLIENTS.with(|clients| {
        clients.borrow_mut().insert(client_id.clone(), client);
    });

    Ok(client_id)
}

/// Получить публичные ключи клиента для регистрации (JSON)
#[wasm_bindgen]
pub fn get_registration_bundle(client_id: String) -> Result<String, JsValue> {
    CLIENTS.with(|clients| {
        let clients_ref = clients.borrow();
        let client = clients_ref.get(&client_id)
            .ok_or_else(|| JsValue::from_str("Client not found"))?;

        let bundle = crypto::get_registration_bundle(client)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        crypto::serialize_key_bundle(&bundle)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Инициализировать сессию с контактом (отправитель)
/// remote_bundle_json - JSON строка с ключами удаленной стороны
/// Возвращает session_id
#[wasm_bindgen]
pub fn init_session(
    client_id: String,
    contact_id: String,
    remote_bundle_json: String,
) -> Result<String, JsValue> {
    CLIENTS.with(|clients| {
        let mut clients_ref = clients.borrow_mut();
        let client = clients_ref.get_mut(&client_id)
            .ok_or_else(|| JsValue::from_str("Client not found"))?;

        let remote_bundle = crypto::deserialize_key_bundle(&remote_bundle_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        messaging::init_session(client, &contact_id, &remote_bundle)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Инициализировать сессию получателя при получении первого сообщения
/// first_message_json - JSON строка с первым зашифрованным сообщением от отправителя
/// Возвращает session_id
#[wasm_bindgen]
pub fn init_receiving_session(
    client_id: String,
    contact_id: String,
    remote_bundle_json: String,
    first_message_json: String,
) -> Result<String, JsValue> {
    CLIENTS.with(|clients| {
        let mut clients_ref = clients.borrow_mut();
        let client = clients_ref.get_mut(&client_id)
            .ok_or_else(|| JsValue::from_str("Client not found"))?;

        let remote_bundle = crypto::deserialize_key_bundle(&remote_bundle_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let first_message = messaging::deserialize_encrypted_message(&first_message_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        messaging::init_receiving_session(client, &contact_id, &remote_bundle, &first_message)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Зашифровать сообщение
/// Возвращает JSON с зашифрованным сообщением
#[wasm_bindgen]
pub fn encrypt_message(
    client_id: String,
    session_id: String,
    plaintext: String,
) -> Result<String, JsValue> {
    CLIENTS.with(|clients| {
        let mut clients_ref = clients.borrow_mut();
        let client = clients_ref.get_mut(&client_id)
            .ok_or_else(|| JsValue::from_str("Client not found"))?;

        let encrypted = messaging::encrypt_message(client, &session_id, &plaintext)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        messaging::serialize_encrypted_message(&encrypted)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Расшифровать сообщение
/// encrypted_json - JSON строка с зашифрованным сообщением
/// Возвращает расшифрованный текст
#[wasm_bindgen]
pub fn decrypt_message(
    client_id: String,
    session_id: String,
    encrypted_json: String,
) -> Result<String, JsValue> {
    CLIENTS.with(|clients| {
        let mut clients_ref = clients.borrow_mut();
        let client = clients_ref.get_mut(&client_id)
            .ok_or_else(|| JsValue::from_str("Client not found"))?;

        let encrypted = messaging::deserialize_encrypted_message(&encrypted_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        messaging::decrypt_message(client, &session_id, encrypted)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Удалить клиента из памяти
#[wasm_bindgen]
pub fn destroy_client(client_id: String) -> Result<(), JsValue> {
    CLIENTS.with(|clients| {
        clients.borrow_mut().remove(&client_id)
            .ok_or_else(|| JsValue::from_str("Client not found"))?;
        Ok(())
    })
}
