use std::convert::TryInto;
use std::path::PathBuf;

use crate::graph_structs::{Node, PointerNode, ValueNode, BasePointerNode, BaseValueNode};

/// convert an address to an index
/// NOTE: addresses are represented as u64
pub fn addr_to_index(addr: u64, min_addr: u64, block_size: usize) -> usize {
    ((addr - min_addr) / block_size as u64) as usize
}

/// convert an index to an address
/// NOTE: indexes are represented as usize
pub fn index_to_addr(index: usize, min_addr: u64, block_size: usize) -> u64 {
    (index * block_size) as u64 + min_addr
}

/// convert a hex string to an address represented as a u64
/// WARN: necessary to specify the string endianness for the conversion
/// WARN: Due to little endian needing to have a fixed length of 16 characters, 
///       the hex string will be padded with 0s to the right if it is less than 16 characters
/// NOTE: always returns a big endian address as a u64
pub fn hex_str_to_addr(hex_str: &str, endianness: Endianness) -> Result<u64, std::num::ParseIntError> {
    match endianness {
        Endianness::Big => Ok(u64::from_str_radix(hex_str, 16)?),
        Endianness::Little => {
            //assert_eq(hex_str.len(), 16, "Little endian hex string ({}) must be 16 characters long", hex_str);
            // append 0s to the right if the hex string is less than 16 characters
            let mut padded_hex_str = hex_str.to_string();
            while padded_hex_str.len() < 16 {
                padded_hex_str.push('0');
            }
            let addr = u64::from_str_radix(padded_hex_str.as_str(), 16)?;
            //log::debug!("Little endian padded hex string {}", padded_hex_str);
            Ok(addr.swap_bytes())
        },
    }
}

/// convert a hex string to a block of bytes following the specified endianness
/// NOTE: always returns a block of bytes in big endian
/// NOTE: remember that our heap dump vectors are in little endian
pub fn hex_str_to_block_bytes(hex_str: &str, endianness: Endianness) -> [u8; crate::params::BLOCK_BYTE_SIZE] {
    hex_str_to_addr(hex_str, endianness).unwrap().to_be_bytes()
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Endianness {
    Big,
    Little,
}

/// convert a block of bytes to a pointer if it is a valid pointer
/// NOTE: A valid pointer is a pointer that is within the heap dump range
/// NOTE: remember that our heap dump vectors are in little endian
pub fn convert_block_to_pointer_if_possible(data: &[u8], min_addr: u64, max_addr: u64, endianness: Endianness) -> Option<u64> {
    let potential_ptr_int = match endianness {
        Endianness::Big => u64::from_be_bytes(data.try_into().unwrap()),
        Endianness::Little => u64::from_le_bytes(data.try_into().unwrap()),
    };

    if potential_ptr_int >= min_addr && potential_ptr_int <= max_addr {
        Some(potential_ptr_int)
    } else {
        None
    }
}

/// create a node from a block of bytes, following the specified endianness
/// NOTE: no need to provide endianess for the pointer conversion directly, 
/// it uses the global pointer endianness
pub fn create_node_from_bytes(
    block: &[u8; crate::params::BLOCK_BYTE_SIZE],
    addr: u64,
    min_addr: u64,
    max_addr: u64,
    endianness: Option<Endianness>
) -> Node {
    let potential_ptr = convert_block_to_pointer_if_possible(
        block, min_addr, max_addr, endianness.unwrap_or(crate::params::PTR_ENDIANNESS)
    );
    if potential_ptr.is_some() {
        Node::PointerNode(
            PointerNode::BasePointerNode(
                BasePointerNode {
                    addr,
                    points_to: potential_ptr.unwrap(),
                }
            )
        )
    } else {
        Node::ValueNode(
            ValueNode::BaseValueNode(
                BaseValueNode {
                    addr,
                    value: *block,
                }
            )
        )
    }
}

/// Convert a path to a heap dump file to a path to a associated json file
pub fn heap_dump_path_to_json_path(heap_dump_raw_file_path: &PathBuf) -> PathBuf {
    let original_heap_path_str = heap_dump_raw_file_path.to_str().unwrap().to_string();
    let json_path = PathBuf::from(
        original_heap_path_str.replace("-heap.raw", ".json")
    );

    if !json_path.exists() {
        log::error!("File doesn't exist: {:?}", json_path);
    } else {
        log::info!(" 📋 associated json file path: {:?}", json_path);
    }
    return json_path;
}