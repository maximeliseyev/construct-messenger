// Криптографический модуль
// X3DH + Double Ratchet + Signal Protocol

pub mod client;
pub mod double_ratchet;
pub mod x3dh;
pub mod keys;
pub mod session;

// Post-Quantum modules (conditionally compiled)
#[cfg(feature = "post-quantum")]
pub mod pq_x3dh;
#[cfg(feature = "post-quantum")]
pub mod pq_double_ratchet;

pub use client::ClientCrypto;
pub use double_ratchet::{DoubleRatchetSession, EncryptedRatchetMessage, SerializableSession};
pub use x3dh::{PublicKeyBundle, RegistrationBundle, X3DH};
