// Типы сообщений протокола

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    Register {
        username: String,
        public_key: Vec<u8>,
    },
    Login {
        username: String,
    },
    SendMessage {
        to: String,
        content: Vec<u8>,
    },
}
