# 🎯 The Philosophy: Rust as the Engine (80-90% of Logic)

The core philosophy of this project is to implement the vast majority of application logic, especially security-sensitive and performance-critical parts, in Rust. This is the same battle-tested approach used by modern secure applications like Signal and 1Password. Rust becomes the "powerplant" for all business logic and cryptography, while TypeScript remains a thin, declarative wrapper for the UI.

## What is Fully Implemented in Rust

### **1. Cryptography (Exclusively in Rust)**
All cryptographic operations, from key generation to encryption and protocol implementation (Signal Protocol), are confined within the Rust module. This is non-negotiable for security.

```rust
// ALL cryptography remains in Rust
mod crypto {
    pub struct SignalProtocol {
        x3dh: X3DH,
        double_ratchet: DoubleRatchet,
        prekey_bundles: PreKeyManager,
    }

    impl SignalProtocol {
        // The entire Signal protocol in Rust
        pub fn encrypt_message(&self, session: &Session, plaintext: &[u8]) -> Ciphertext {
            // Triple Diffie-Hellman
            // Chain Key derivation
            // Message Key generation
            // AEAD encryption (ChaCha20-Poly1305)
        }
    }
}
```

### **2. State and Data Management**
The entire application state, including chat sessions, contact lists, user settings, and message queues, is managed by Rust. TypeScript only receives representations of this state for rendering purposes.

```rust
// All state in Rust
struct AppState {
    // Chat sessions
    sessions: HashMap<String, ChatSession>,
    // Contacts
    contacts: ContactList,
    // Settings
    settings: UserSettings,
    // Message queues
    outgoing_queue: MessageQueue,
    incoming_queue: MessageQueue,
    // Network state
    connection: ConnectionState,
}
// TypeScript only displays this state
```

### **3. Network Logic and Protocols**
The complete communication protocol, including WebSocket handling, message serialization (e.g., MessagePack), and complex network patterns like retry logic and acknowledgments, is implemented in Rust.

```rust
// The entire communication protocol in Rust
pub struct NetworkProtocol {
    ws_client: WebSocketClient,
    message_packer: MessagePack,
    retry_logic: ExponentialBackoff,

    // All logic for retries, ACKs, timeouts
}
```

### **4. Storage Interaction**
Rust provides a secure, high-level API for interacting with browser storage (IndexedDB). The TypeScript layer calls this API but never has direct access to the raw stored data or encryption keys.

```rust
// Rust provides a safe API to IndexedDB
#[wasm_bindgen]
pub struct SecureStorage {
    db: IndexedDbWrapper,
    encryption_key: [u8; 32],
}
```

## 📦 Advantages of This Approach

| Aspect              | Rust/WASM                                  | TypeScript                       |
|---------------------|--------------------------------------------|----------------------------------|
| **Security**        | Memory-safe, zeroization, constant-time crypto | Vulnerable to XSS, supply-chain attacks |
| **Performance**     | Near-native speed, optimized for WASM      | 10-100x slower for heavy computation |
| **Auditability**    | All critical code is centralized and verifiable | Logic is scattered across components |
| **Testability**     | Unit tests, property-based testing, fuzzing | Primarily integration/E2E tests |
| **Portability**     | The same core can target Web, iOS, Android, Desktop | Browser only                     |

This architecture makes the application safer, faster, more reliable, and easier to maintain by enforcing a clean separation of concerns.
