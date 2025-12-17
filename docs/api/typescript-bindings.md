# TypeScript Bindings & WASM Integration

This guide explains how the TypeScript frontend integrates with the Rust/WASM core for cryptographic operations.

## 1. WASM Initialization

The first step is to load and initialize the WASM module. This should be done once when the application starts.

```typescript
// wasm-loader.ts
import init from './construct_crypto_bg.wasm';

let wasmInitialized = false;

export async function initWasm(): Promise<void> {
  if (wasmInitialized) return;
  await init(); // `init` is exported by the wasm-pack generated JS file
  wasmInitialized = true;
}
```

## 2. The `window.constructCrypto` Interface

A global object `window.constructCrypto` is exposed to provide a clean, typed interface to the underlying Rust functions.

```typescript
// wasm-bindings.ts

// Extend the Window interface for TypeScript
declare global {
  interface Window {
    constructCrypto: {
      generateRegistrationKeys(): Promise<RegistrationBundle>;
      initSession(contactId: string, bundle: PublicKeyBundleData): Promise<string>; // Returns session ID
      encryptMessage(sessionId: string, plaintext: string): Promise<EncryptedResult>;
      decryptMessage(sessionId: string, encrypted: EncryptedResult): Promise<string>; // Returns plaintext
    };
  }
}

// Wrapper function to expose Rust functions
export function setupCryptoBindings(cryptoModule: CryptoModule) {
  window.constructCrypto = {
    generateRegistrationKeys: async () => {
      const result = await cryptoModule.generate_registration_keys();
      return serde_wasm_bindgen.from_value(result);
    },
    // ... other wrapped functions
  };
}
```

## 3. Usage Examples

### Registration
Generating keys for a new user. The private keys are managed internally by the Rust module and stored securely in IndexedDB.

```typescript
// api/auth.ts
export async function register(username: string, password: string): Promise<void> {
  // 1. Generate keys and bundle via WASM
  const bundle = await window.constructCrypto.generateRegistrationKeys();

  // 2. Serialize bundle for the server
  const bundleBase64 = MessagePackSerializer.serializeToBase64(bundle);

  // 3. Send the registration request to the server
  const message: ClientMessage = {
    type: 'register',
    payload: {
      username,
      password,
      publicKey: bundleBase64
    }
  };

  await webSocketManager.send(message);
}
```

### Sending an Encrypted Message
The end-to-end flow for sending a message involves fetching the recipient's keys, initializing a session, encrypting the message, and sending it over the WebSocket.

```typescript
// api/messaging.ts
export async function sendEncryptedMessage(recipientId: string, plaintext: string): Promise<void> {
  // 1. Fetch recipient's public key bundle from the server
  const bundle = await getPublicKeyBundle(recipientId);

  // 2. Initialize Double Ratchet session via WASM
  const sessionId = await window.constructCrypto.initSession(recipientId, bundle);

  // 3. Encrypt the message using the session
  const encrypted = await window.constructCrypto.encryptMessage(sessionId, plaintext);

  // 4. Create the ChatMessage structure
  const message: ChatMessage = {
    id: generateUUID(),
    from: getCurrentUserId(),
    to: recipientId,
    ephemeralPublicKey: encrypted.ephemeralPublicKey,
    messageNumber: encrypted.messageNumber,
    content: uint8ArrayToBase64(encrypted.ciphertext),
    timestamp: Math.floor(Date.now() / 1000)
  };

  // 5. Send via WebSocket
  await webSocketManager.send({ type: 'sendMessage', payload: message });
}
```

This clear separation ensures that no cryptographic logic or private keys ever leak into the JavaScript/TypeScript environment, maximizing security.
