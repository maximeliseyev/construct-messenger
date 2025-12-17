# Key Management in the PWA

Securely managing cryptographic keys in a hostile browser environment is a critical challenge. Our strategy relies on a combination of browser storage APIs and in-memory best practices.

## 1. Storage Backend: IndexedDB

-   **Primary Storage**: **IndexedDB** is the only suitable storage mechanism in the browser for persistently storing sensitive data like private keys or message history. `localStorage` and `sessionStorage` are not secure enough.

-   **Encryption at Rest**: Data must **always** be encrypted *before* being written to IndexedDB. The database itself is accessible to other scripts from the same origin (e.g., browser extensions, XSS).
    -   We use a master storage key derived from a user's passphrase to encrypt the database contents.

## 2. Key Derivation and Storage Encryption

A strong key is derived from the user's passphrase to encrypt the local IndexedDB vault.

```typescript
// Example of deriving a storage key
class Keystore {
  private async deriveStorageKey(passphrase: string, salt: Uint8Array): Promise<CryptoKey> {
    // Use a strong KDF like Argon2 (in WASM) or PBKDF2.
    // The key is derived in memory and never stored directly.
    return window.crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: salt,
        iterations: 310000, // OWASP recommendation
        hash: 'SHA-256'
      },
      passphraseKey,
      { name: 'AES-GCM', length: 256 },
      false, // Not extractable
      ['encrypt', 'decrypt']
    );
  }

  private async encryptForStorage(data: Uint8Array, storageKey: CryptoKey): Promise<EncryptedData> {
    // Encrypt data before writing to IndexedDB
    const iv = crypto.getRandomValues(new Uint8Array(12));
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      storageKey,
      data
    );
    return { encrypted, iv };
  }
}
```

## 3. In-Memory Key Management

-   **Minimize Lifetime**: Private keys should only be held in memory for the absolute minimum time required to perform an operation.
-   **Web Worker Isolation**: As mentioned in the `encryption.md` guide, all operations involving private keys must happen inside an isolated Web Worker to protect them from the main thread.
-   **No Seed Phrases**: The user's primary seed phrase should **never** be stored persistently in the browser. It should be entered by the user for session initialization only. For linking new devices, a QR code mechanism (similar to Signal) is preferred.
