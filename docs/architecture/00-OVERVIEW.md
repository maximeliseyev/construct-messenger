# 🏗️ Architectural Model: Rust as the "Core" of the Application

This document provides a high-level overview of the project's architecture, which is centered around a Rust core compiled to WebAssembly (WASM). This approach ensures that critical business logic, cryptography, and state management are handled in a performant and secure environment, while the user interface remains a thin layer built with web technologies.

```
┌─────────────────────────────────────────────────┐
│            TypeScript/Svelte (UI Layer)         │
│  • Component Rendering (5-10% of code)          │
│  • User Input Handling                          │
│  • DOM Management                               │
│  • Routing (SvelteKit)                          │
└────────────────┬────────────────────────────────┘
                 │ Thin Wrapper (wasm-bindgen)
┌────────────────▼────────────────────────────────┐
│           Rust/WASM (Core Layer)                │
│  • All Cryptography (100%)                      │
│  • State Management (sessions, keys)            │
│  • Messenger Business Logic                     │
│  • Data Validation                              │
│  • Network Logic (protocols, serialization)     │
│  • Storage Interface (IndexedDB)                │
└────────────────┬────────────────────────────────┘
                 │ MessagePack/Protobuf
┌────────────────▼────────────────────────────────┐
│               Server API                        │
│  • Public Key Storage                           │
│  • Message Routing                              │
│  • Notifications                                │
└─────────────────────────────────────────────────┘
```

## 📊 Responsibility Distribution

### **Rust/WASM (80-90% of code) - "What to do"**
The Rust core is responsible for the heavy lifting. It implements the "what" of the application's logic, including:
- All cryptographic operations.
- State management for sessions, contacts, and settings.
- The complete business logic of the messenger.
- Secure data validation and serialization.
- Network protocols and communication.
- A secure API for interacting with browser storage (IndexedDB).

### **TypeScript (10-20% of code) - "How to show it"**
The TypeScript layer is a thin client whose primary job is to present the UI and delegate all complex operations to the Rust core. Its responsibilities are limited to:
- Rendering UI components.
- Capturing user input and forwarding it to the WASM module.
- Performing simple UI-level validation (e.g., checking for empty fields).
- Updating the UI based on results and events from the Rust core.
