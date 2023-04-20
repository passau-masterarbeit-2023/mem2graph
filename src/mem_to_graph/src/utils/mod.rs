use std::convert::TryInto;
use std::path::PathBuf;

use serde_json::Value;
use crate::params::PTR_ENDIANNESS;

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

/// convert a block of bytes to a u64 address
pub fn block_bytes_to_addr(block_bytes: &[u8; crate::params::BLOCK_BYTE_SIZE], endianness: Endianness) -> u64 {
    let mut addr = 0u64;
    for (i, byte) in block_bytes.iter().enumerate() {
        match endianness {
            Endianness::Big => addr += (*byte as u64) << (8 * (7 - i)),
            Endianness::Little => addr += (*byte as u64) << (8 * i),
        }
    }
    addr
}

/// convert a json value to an address represented as a u64 (intended from a hex string)
/// WARN: all addresses in the json file are big endian
pub fn json_value_to_addr(json_value: &Value) -> u64 {
    let addr_str = json_value.as_str().unwrap();
    hex_str_to_addr(addr_str, Endianness::Big).unwrap()
}

/// convert a json value to a usize (intented from a decimal string)
pub fn json_value_to_usize(json_value: &Value) -> usize {
    let int_str = json_value.as_str().unwrap();
    u64::from_str_radix(int_str, 10).unwrap() as usize
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

/// convert a hex string to a block of bytes
pub fn hex_str_to_block_bytes(hex_str: &str) -> [u8; crate::params::BLOCK_BYTE_SIZE] {
    assert_eq!(hex_str.len(), crate::params::BLOCK_BYTE_SIZE * 2, "Hex string ({}) must be {} characters long", hex_str, crate::params::BLOCK_BYTE_SIZE * 2);
    let padded_hex_str = hex_str.to_string();
    let mut block_bytes = [0u8; crate::params::BLOCK_BYTE_SIZE];
    for (i, byte) in padded_hex_str.as_bytes().chunks(2).enumerate() {
        block_bytes[i] = u8::from_str_radix(std::str::from_utf8(byte).unwrap(), 16).unwrap();
    }
    block_bytes
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Endianness {
    Big,
    Little,
}

/// convert a block of bytes to a pointer if it is a valid pointer
/// NOTE: A valid pointer is a pointer that is within the heap dump range
/// NOTE: remember that our heap dump vectors are in the format given as a program argument
pub fn convert_block_to_pointer_if_possible(data: &[u8], min_addr: u64, max_addr: u64) -> Option<u64> {
    // WARN: THIS IS THE ONLY PLACE WHERE THE POINTER ENDIANNESS IS USED
    

    let potential_ptr_int = match PTR_ENDIANNESS {
        Endianness::Big => u64::from_be_bytes(data.try_into().unwrap()),
        Endianness::Little => u64::from_le_bytes(data.try_into().unwrap()),
    };

    // check if the potential pointer is within the heap dump range
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
    dtn_addr: u64,
    min_addr: u64,
    max_addr: u64,
) -> Node {
    let potential_ptr = convert_block_to_pointer_if_possible(
        block, min_addr, max_addr
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
                    dtn_addr,
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
    }
    return json_path;
}