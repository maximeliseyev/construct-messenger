// Wire format (MessagePack сериализация)
// Используется для передачи сообщений через WebSocket

use crate::protocol::messages::ProtocolMessage;
use crate::utils::error::{ConstructError, Result};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

/// Упаковать ProtocolMessage в MessagePack формат
pub fn pack_message(message: &ProtocolMessage) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    message
        .serialize(&mut Serializer::new(&mut buffer))
        .map_err(|e| ConstructError::SerializationError(format!("MessagePack pack error: {}", e)))?;
    Ok(buffer)
}

/// Распаковать MessagePack в ProtocolMessage
pub fn unpack_message(data: &[u8]) -> Result<ProtocolMessage> {
    let mut deserializer = Deserializer::new(data);
    ProtocolMessage::deserialize(&mut deserializer)
        .map_err(|e| ConstructError::SerializationError(format!("MessagePack unpack error: {}", e)))
}

/// Упаковать произвольные данные в MessagePack
pub fn pack_raw<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    data.serialize(&mut Serializer::new(&mut buffer))
        .map_err(|e| ConstructError::SerializationError(format!("MessagePack pack error: {}", e)))?;
    Ok(buffer)
}

/// Распаковать MessagePack в произвольный тип
pub fn unpack_raw<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T> {
    let mut deserializer = Deserializer::new(data);
    T::deserialize(&mut deserializer)
        .map_err(|e| ConstructError::SerializationError(format!("MessagePack unpack error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::messages::{AckData, ErrorData};

    #[test]
    fn test_pack_unpack_ping() {
        let msg = ProtocolMessage::Ping;
        let packed = pack_message(&msg).unwrap();
        let unpacked = unpack_message(&packed).unwrap();

        matches!(unpacked, ProtocolMessage::Ping);
    }

    #[test]
    fn test_pack_unpack_error() {
        let msg = ProtocolMessage::error(404, "Not found".to_string());
        let packed = pack_message(&msg).unwrap();
        let unpacked = unpack_message(&packed).unwrap();

        if let ProtocolMessage::Error(err) = unpacked {
            assert_eq!(err.code, 404);
            assert_eq!(err.message, "Not found");
        } else {
            panic!("Expected Error variant");
        }
    }

    #[test]
    fn test_pack_unpack_ack() {
        let ack = AckData {
            message_id: "test-id".to_string(),
            timestamp: 1234567890,
        };
        let msg = ProtocolMessage::Ack(ack);
        let packed = pack_message(&msg).unwrap();
        let unpacked = unpack_message(&packed).unwrap();

        if let ProtocolMessage::Ack(ack_data) = unpacked {
            assert_eq!(ack_data.message_id, "test-id");
            assert_eq!(ack_data.timestamp, 1234567890);
        } else {
            panic!("Expected Ack variant");
        }
    }
}
