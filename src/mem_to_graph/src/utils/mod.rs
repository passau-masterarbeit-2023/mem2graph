use std::convert::TryInto;
//use graph_structures::{Node, PointerNode, ValueNode};


pub fn addr_to_index(addr: u64, min_addr: u64, block_size: usize) -> usize {
    ((addr - min_addr) / block_size as u64) as usize
}

pub fn index_to_addr(index: usize, min_addr: u64, block_size: usize) -> u64 {
    (index * block_size) as u64 + min_addr
}

pub fn hex_str_to_addr(hex_str: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(hex_str, 16)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Endianness {
    Big,
    Little,
}

pub fn is_pointer(data: &[u8], min_addr: u64, max_addr: u64, endianness: Endianness) -> Option<u64> {
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

// fn create_node_from_bytes(block: &[u8], addr: usize, min_addr: usize, max_addr: usize, endianness: Endianness) -> Node {
//     if let Some(potential_ptr) = is_pointer(block, min_addr, max_addr, endianness) {
//         Node::Pointer(PointerNode::new(addr, potential_ptr))
//     } else {
//         Node::Value(ValueNode::new(addr, block.to_vec()))
//     }
// }


