# 🛠️ Build System and Development Process

Our project uses a hybrid build system that combines Rust's `cargo` with Node.js's `pnpm` and `turbo` for managing the monorepo and orchestrating builds.

## Key Configuration Files

### 1. Root `Cargo.toml` (Workspace)
This file defines all the Rust crates in the project and sets up shared dependencies.

```toml
[workspace]
members = [
    "packages/core",
    "crates/crypto-utils",
    # ... other crates
]
resolver = "2"

[workspace.dependencies]
# Shared dependencies for all crates
serde = { version = "1.0", features = ["derive"] }
wasm-bindgen = "0.2"
thiserror = "1.0"
```

### 2. Root `package.json` (Monorepo)
This file defines the workspaces and provides top-level scripts for common development tasks, orchestrated by `turbo`.

```json
{
  "name": "construct-messenger",
  "private": true,
  "workspaces": [
    "apps/*",
    "packages/*"
  ],
  "scripts": {
    "dev": "turbo run dev",
    "build": "turbo run build",
    "build:wasm": "cd packages/core && wasm-pack build --target web --release",
    "test": "turbo run test",
    "test:wasm": "cargo test --workspace",
    "lint": "turbo run lint",
    "audit": "cargo audit && npm audit"
  },
  "devDependencies": {
    "turbo": "^2.0.0",
    "typescript": "^5.0.0"
  }
}
```

### 3. `packages/core/Cargo.toml`
This file contains specific settings for the core Rust crate, including feature flags for different build targets (WASM, desktop, mobile) and optimized release profiles.

```toml
[package]
name = "construct-core"
edition = "2021"

[lib]
crate-type = ["cdylib"] # For WASM

[features]
default = ["wasm"]
wasm = ["dep:wasm-bindgen", "dep:js-sys", "dep:web-sys"]
desktop = ["dep:tokio"]
mobile = ["dep:jni"]

[dependencies]
# WASM-specific dependencies
wasm-bindgen = { version = "0.2", optional = true }
js-sys = { version = "0.3", optional = true }
web-sys = { version = "0.3", optional = true, features = ["console"] }

[profile.release]
lto = true
opt-level = "s"
codegen-units = 1
panic = "abort"
```

## 🔄 Development Process

### Common Commands
```bash
# Clone and install dependencies
git clone <repo>
cd construct-messenger
pnpm install
cargo fetch

# Run the PWA in development mode
pnpm dev

# Build the WASM core
pnpm build:wasm

# Run all tests (TS + Rust)
pnpm test

# Run only Rust tests
pnpm test:wasm

# Check for security vulnerabilities
pnpm audit
```

This setup allows for a streamlined development experience where both Rust and TypeScript code can be built, tested, and managed with a unified set of commands.
