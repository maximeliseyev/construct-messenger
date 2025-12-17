// Wire format (MessagePack сериализация)

use crate::utils::error::Result;

pub fn pack_message(data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Реализация MessagePack
    Ok(data.to_vec())
}

pub fn unpack_message(data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Реализация MessagePack
    Ok(data.to_vec())
}
