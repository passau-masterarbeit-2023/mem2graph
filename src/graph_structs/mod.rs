use std::fmt::Debug;

use serde_derive::{Serialize, Deserialize};

use crate::{params::{BLOCK_BYTE_SIZE, MALLOC_HEADER_ENDIANNESS}, utils};

pub mod annotations;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    ValueNode(ValueNode),
    ChunkHeaderNode(ChunkHeaderNode),
    PointerNode(PointerNode),
    FooterNode(FooterNode),
}

/// Header flags for a header block
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct  HeaderFlags {
    /// P: Previous chunk is in use (allocated by application)
    pub p: bool, 
    /// M: This chunk was allocated using mmap
    pub m: bool,
    /// A: The main arena uses the application's heap
    pub a: bool,
}

impl HeaderFlags {  
    /// Parse the header block of a chunk, just for the flags
    pub fn parse_chunk_header_flags(block: &[u8; BLOCK_BYTE_SIZE]) -> HeaderFlags {
        let size_and_flags = utils::block_bytes_to_addr(block, MALLOC_HEADER_ENDIANNESS) as usize;
        
        // get flags
        let p = (size_and_flags & 0x01) != 0;
        let m = (size_and_flags & 0x02) != 0;
        let a = (size_and_flags & 0x04) != 0;
        HeaderFlags { p, m, a }
    }

    /// Current flag P indicates if the previous chunk is in use (allocated by application) or free
    pub fn is_preceding_chunk_free(&self) -> bool {
        !self.p
    }
}

/// Format flags as a string: flags: [a: 0, m: 0, p: 0]
impl std::fmt::Display for HeaderFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "[a: {}, m: {}, p: {}]",
            self.a as u8, self.m as u8, self.p as u8
        )
    }
}

/// Parse the header block of a chunk
pub fn parse_chunk_header(block: &[u8; BLOCK_BYTE_SIZE]) -> (usize, HeaderFlags) {
    let size_and_flags = utils::block_bytes_to_addr(block, MALLOC_HEADER_ENDIANNESS) as usize;
    
    // get size
    let size = size_and_flags & !0x07;  // Clear the last 3 bits to get the size

    // get flags
    let p = (size_and_flags & 0x01) != 0;
    let m = (size_and_flags & 0x02) != 0;
    let a = (size_and_flags & 0x04) != 0;
    let flags = HeaderFlags { p, m, a };

    return (size, flags);
}

impl Node {
    /// NOTE: If you forget to match a new Node variant, this function will panic.
    pub fn get_address(&self) -> u64 {
        match self {
            Node::ChunkHeaderNode(chunk_header_node) => {
                chunk_header_node.addr
            }
            Node::ValueNode(base_value_node) => {
                base_value_node.addr
            }
            Node::PointerNode(base_pointer_node) => {
                base_pointer_node.addr
            }
            Node::FooterNode(footer_node) => {
                footer_node.addr
            }
        }
    }

    /// returns the address of the node and its type annotation
    pub fn str_addr_and_type(&self) -> String {
        match self {
            Node::ChunkHeaderNode(chunk_header_node) => {
                format!(
                    "CHN({:#x})",
                    chunk_header_node.addr,
                )
            }
            Node::ValueNode(base_value_node) => {
                format!(
                    "VN({:#x})",
                    base_value_node.addr,
                )
            }
            Node::PointerNode(base_pointer_node) => {
                format!(
                    "PN({:#x})",
                    base_pointer_node.addr,
                )
            }
            Node::FooterNode(footer_node) => {
                format!(
                    "FN({:#x})",
                    footer_node.addr,
                )
            }
        }
    }

    /// Check if a node is a pointer node
    pub fn is_pointer(&self) -> bool {
        match self {
            Node::PointerNode(_) => true,
            _ => false,
        }
    }

    /// Check if a node is a footer node
    #[allow(dead_code)]
    pub fn is_footer(&self) -> bool {
        match self {
            Node::FooterNode(_) => true,
            _ => false,
        }
    }

    /// Check if a node is a value node
    pub fn is_value(&self) -> bool {
        match self {
            Node::ValueNode(_) => true,
            _ => false,
        }
    }

    /// Check if a node is a chunk header node
    pub fn is_chn (&self) -> bool {
        match self {
            Node::ChunkHeaderNode(_) => true,
            _ => false,
        }
    }

    pub fn points_to(&self) -> Option<u64> {
        match self {
            Node::PointerNode(base_pointer_node) => {
                Some(base_pointer_node.points_to)
            }
            _ => None,
        }
    }

    pub fn get_value(&self) -> Option<[u8; BLOCK_BYTE_SIZE]> {
        match self {
            Node::ValueNode(base_value_node) => {
                Some(base_value_node.value.clone())
            }
            _ => None,
        }
    }

    /// returns the CHN address of the node
    pub fn get_parent_chn_addr(&self) -> Option<u64> {
        match self {
            Node::ValueNode(base_value_node) => {
                Some(base_value_node.chn_addr)
            }
            Node::PointerNode(base_pointer_node) => {
                Some(base_pointer_node.chn_addr)
            }
            Node::FooterNode(footer_node) => {
                Some(footer_node.chn_addr)
            }
            _ => None,
        }
    }
}


impl std::fmt::Debug for Node {
    /// return a formatted string of the node, for debugging purposes
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::ChunkHeaderNode(chunk_header_node) => {
                write!(
                    f, "CHN: {} [VNs: {}, PNs: {}, flags: {:?}]", 
                    chunk_header_node.addr, 
                    chunk_header_node.nb_value_nodes, 
                    chunk_header_node.nb_pointer_nodes,
                    chunk_header_node.flags
                )
            }
            Node::ValueNode(base_value_node) => {
                write!(
                    f, "VN: {} [value: \"{}\"]", 
                    base_value_node.addr, hex::encode(&base_value_node.value)
                )
            }
            Node::PointerNode(base_pointer_node) => {
                write!(
                    f, "PN: {} [label: \"{}\"]", 
                    base_pointer_node.addr, base_pointer_node.points_to
                )
                    
            }
            Node::FooterNode(footer_node) => {
                write!(
                    f, "FN: {} [size: {}, flags: {:?}]",
                    footer_node.addr, footer_node.byte_size, footer_node.flags
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkHeaderNode {
    pub addr: u64,
    pub byte_size: usize,
    pub flags: HeaderFlags,
    pub is_free: bool,
    pub nb_pointer_nodes: usize,
    pub nb_value_nodes: usize,
    pub start_data_bytes_entropy: f64,
    pub chunk_number_in_heap: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FooterNode {
    pub addr: u64,
    pub byte_size: usize,
    pub flags: HeaderFlags,
    pub chn_addr: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueNode {
    pub addr: u64,
    pub value: [u8; BLOCK_BYTE_SIZE],
    pub chn_addr: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointerNode {
    pub addr: u64,
    pub points_to: u64,
    pub chn_addr: u64,
}

pub const DEFAULT_CHUNK_EDGE_WEIGHT: usize = 1;

pub struct Edge {
    pub from: u64,
    pub to: u64,
    pub edge_type: EdgeType,
    pub weight: usize, // Number of edge pointers between the two nodes, default is 1 for a DataStructure edge.
}

pub enum EdgeType {
    ChunkEdge,
    PointerEdge,
}


impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::ChunkEdge => write!(f, "chunk"),
            EdgeType::PointerEdge => write!(f, "ptr"),
        }
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "    {:?} -> {:?} [label=\"{}\" weight={}]",
            self.from, self.to, self.edge_type, self.weight
        )
    }
}

/// display the name of the node (without the annotation)
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::ChunkHeaderNode(_) => {
                write!(
                    f, "    {:?}", 
                    self.str_addr_and_type(),
                )
            }
            Node::ValueNode(_) => {
                write!(
                    f, "    {:?}", 
                    self.str_addr_and_type(),
                )
            }
            Node::PointerNode(_) => {
                write!(
                    f, "    {:?}",
                    self.str_addr_and_type(),
                )
            }
            Node::FooterNode(_) => {
                write!(
                    f, "    {:?}",
                    self.str_addr_and_type(),
                )
            }
        }
    } 
}

