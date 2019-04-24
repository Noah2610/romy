//! Standard serialization encoding for the project

use byteorder::{LittleEndian, ReadBytesExt};

/// Encodes an object as a series of bytes
/// 
/// # Arguments
/// * `object` - the object to encode
pub fn encode(object: &impl serde::Serialize) -> Vec<u8> {
    bincode::serialize(object).unwrap()
}

/// Encodes an object as a series of bytes, tacking on the size of the data as a u64 at the front
/// 
/// # Arguments
/// * `object` - the object to encode
pub fn encode_with_size(object: &impl serde::Serialize) -> Vec<u8> {
    let mut data = Vec::new();
    let serial = encode(object);
    data.extend((serial.len() as u64).to_le_bytes().iter());
    data.extend(serial.iter());
    data
}

/// Decodes an object from a series of bytes
/// 
/// # Arguments
/// * `data` - the data to decode 
pub fn decode<'a, T: serde::Deserialize<'a>>(data: &'a [u8]) -> T {
    bincode::deserialize(data).unwrap()
}

/// Decodes an object from a series of bytes that has had a size tacked on the front as a u64
/// 
/// # Arguments
/// * `data` - the data to decode 
pub fn decode_with_size<'a, T: serde::Deserialize<'a>>(data: &'a [u8]) -> T {
    decode(&data[8..])
}

/// Decodes an object from a series of bytes given as a pointer that has had a size tacked on the
/// front as a u64
/// 
/// # Arguments
/// * `data` - the data to decode 
pub unsafe fn decode_with_size_ptr<'a, T: serde::Deserialize<'a>>(data: *const u8) -> T {
    let size = std::slice::from_raw_parts(data, 8)
        .read_u64::<LittleEndian>()
        .unwrap();
    let data = std::slice::from_raw_parts(data.offset(8), size as usize);
    decode(&data)
}
