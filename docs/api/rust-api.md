# Rust Core API

This document outlines the public API of the `construct-core` Rust crate, which is exposed to the frontend via WebAssembly.

*This document is intended to be auto-generated from Rust doc comments in the future.*

## Core Entrypoint

The main entrypoint for the WASM module.

```rust
#[wasm_bindgen]
pub struct MessengerApi {
    // ...
}
```

## Public Functions

-   `generate_registration_keys()`
-   `init_session()`
-   `encrypt_message()`
-   `decrypt_message()`
