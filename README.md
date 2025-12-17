# Construct Messenger

Secure end-to-end encrypted messenger built with Rust and TypeScript.

## Architecture

This is a monorepo containing:

- **`packages/core`** - Rust/WASM core engine with E2EE (80-90% of logic)
- **`apps/pwa`** - Progressive Web App frontend (TypeScript/Svelte)
- **`crates/`** - Auxiliary Rust libraries
- **`docs/`** - Documentation

### Design Philosophy

Following the principle of **"Rust as the Engine, TypeScript as the UI"**:

- **Rust handles:** All cryptography, business logic, state management, networking, data processing
- **TypeScript handles:** UI rendering, user input, DOM manipulation, routing

## Quick Start

### Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Node.js 18+
- pnpm 8+
- wasm-pack

```bash
# Install Rust target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack

# Install Node dependencies
pnpm install
```

### Development

```bash
# Build WASM core
pnpm build:wasm

# Run tests
pnpm test          # All tests
pnpm test:wasm     # Rust tests only

# Development
pnpm dev           # Start all apps in development mode
```

### Project Structure

```
construct-messenger/
├── packages/core/          # Rust/WASM core
│   ├── src/
│   │   ├── api/           # Public API for TypeScript
│   │   ├── crypto/        # X3DH + Double Ratchet
│   │   ├── protocol/      # Network protocol
│   │   ├── storage/       # IndexedDB interface
│   │   ├── state/         # State management
│   │   ├── wasm/          # WASM-specific code
│   │   └── utils/         # Utilities
│   └── Cargo.toml
│
├── apps/pwa/              # PWA frontend
│   └── (to be created)
│
├── crates/                # Auxiliary Rust crates
│   └── (to be created)
│
└── docs/                  # Documentation
    ├── architecture/
    ├── api/
    └── security/
```

## Security

This messenger implements:

- **X3DH** (Extended Triple Diffie-Hellman) for key agreement
- **Double Ratchet** for forward secrecy
- **Signal Protocol** for end-to-end encryption
- **ChaCha20-Poly1305** for AEAD encryption
- **Ed25519** for signatures
- **X25519** for key exchange

All cryptographic operations are performed in Rust/WASM, isolated from JavaScript.

## Testing

```bash
# Rust unit tests
cargo test --workspace

# Rust integration tests
cargo test --workspace --test '*'

# WASM tests in browser
cd packages/core && wasm-pack test --headless --firefox

# TypeScript tests
pnpm test
```

## Building

```bash
# Production build
pnpm build

# WASM only
pnpm build:wasm

# With optimizations
cd packages/core
wasm-pack build --target web --release
```

## Tech Stack

### Core (Rust)
- `x25519-dalek` - Key exchange
- `ed25519-dalek` - Signatures
- `chacha20poly1305` - Encryption
- `wasm-bindgen` - JavaScript bindings
- `serde` - Serialization

### Frontend (TypeScript)
- Svelte/SvelteKit - UI framework
- TypeScript - Type safety
- Vite - Build tool

## License

MIT

