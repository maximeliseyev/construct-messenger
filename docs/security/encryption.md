# PWA Encryption Strategy

The cornerstone of the PWA's security is performing all cryptographic operations on the client-side within a controlled and isolated environment.

## 1. Cryptographic Stack

-   **Primary Engine**: **Rust compiled to WebAssembly (WASM)** is the absolute priority for all cryptographic logic.
    -   **Why Rust?**: It is memory-safe by design, has no garbage collector (predictable performance), and a strong type system that prevents common vulnerabilities.
    -   **Libraries**: We should use battle-tested Rust crypto libraries like `libsignal` (for the Signal Protocol), `aes-gcm`, and `chacha20poly1305`.
    -   **Bindings**: `wasm-bindgen` is used to create a minimal, tightly-controlled bridge between the Rust/WASM module and the TypeScript UI layer.

-   **Browser API Support**: The **Web Crypto API** is used for functionality that can benefit from hardware acceleration or is not yet implemented in the WASM module.
    -   **Use Cases**: Primarily for generating cryptographically secure random numbers (`crypto.getRandomValues`).

## 2. WASM Security and Isolation

To protect cryptographic operations from potential XSS vulnerabilities in the JavaScript/UI layer, we must enforce strict isolation.

-   **Dedicated Web Worker**: The WASM module should run inside a dedicated Web Worker.
    -   **Benefit**: This isolates the memory space of the crypto engine. Private keys and intermediate cryptographic state never enter the main browser thread, where they could be accessed by a malicious script.

-   **Minimal Interface**: The API exposed by the WASM module to the TypeScript code should be high-level and abstract.
    -   **DO**: Expose functions like `encrypt_message_for_contact(contactId, plaintext)`.
    -   **DON'T**: Expose low-level functions like `encrypt_with_key(key, data)`. The JavaScript environment should never see or handle a private key.

-   **Subresource Integrity (SRI)**: The WASM module and all critical JavaScript files must be loaded with an integrity hash to prevent loading a compromised file.
    ```html
    <script src="crypto-worker.js" integrity="sha384-..."></script>
    ```
