use std::fmt::Debug;

use serde_derive::{Serialize, Deserialize};

use crate::{params::{BLOCK_BYTE_SIZE, MALLOC_HEADER_ENDIANNESS}, utils};


#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Node {
    KeyNode(KeyNode),
    ValueNode(ValueNode),
    ChunkHeaderNode(ChunkHeaderNode),
    PointerNode(PointerNode),
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

    pub fn is_preceding_chunk_free(&self) -> bool {
        !self.p
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

/// Anotations for special nodes used in graph_data.
/// Allow labelling for embedding, and attribute and coloring for graph generation.
/// NOTE : Take the address of the node as parameter, to permit to display it in the label.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialNodeAnnotation {
    SessionStateNodeAnnotation(u64),
    SshStructNodeAnnotation(u64),
    /// combination of the SessionStateNodeAnnotation and SshStructNodeAnnotation (handle the superposition of the two annotations)
    SessionStateAndSSHStructNodeAnnotation(u64),
    KeyNodeAnnotation(u64),

    /// combination of the SessionStateNodeAnnotation and KeyNodeAnnotation (handle the superposition of the two annotations)
    KeyNodeAndSessionStateNodeAnnotation(u64),
}

impl SpecialNodeAnnotation {
    fn get_name(&self) -> String {
        match self {
            SpecialNodeAnnotation::SessionStateNodeAnnotation(_) => {
                "SSN".to_string()
            }
            SpecialNodeAnnotation::SshStructNodeAnnotation(_) => {
                "SSHN".to_string()
            }
            SpecialNodeAnnotation::SessionStateAndSSHStructNodeAnnotation(_) => {
                "SSN_SSHN".to_string()
            }
            SpecialNodeAnnotation::KeyNodeAnnotation(_) => {
                "KN".to_string()
            }
            SpecialNodeAnnotation::KeyNodeAndSessionStateNodeAnnotation(_) => {
                "KN_SSN".to_string()
            }
        }
    }

    fn get_color(&self) -> String {
        match self {
            SpecialNodeAnnotation::SessionStateNodeAnnotation(_) => {
                "red".to_string()
            }
            SpecialNodeAnnotation::SshStructNodeAnnotation(_) => {
                "red".to_string()
            }
            SpecialNodeAnnotation::SessionStateAndSSHStructNodeAnnotation(_) => {
                "red".to_string()
            }
            SpecialNodeAnnotation::KeyNodeAnnotation(_) => {
                "green".to_string()
            }
            SpecialNodeAnnotation::KeyNodeAndSessionStateNodeAnnotation(_) => {
                "green".to_string()
            }
        }
    }


    /// get the dot attributes for the node
    pub fn annotate_dot_attributes(&self) -> String {
        format!(
            "[label=\"{}\" color=\"{}\" style=filled];",
            self.get_name(),
            self.get_color(),
        )
    }


    /// get the default dot attributes for the node
    /// WARN : Key node should not be annotated with default attributes
    pub fn get_default_dot_attributes(node : &Node) -> String {
        match node {
            Node::ChunkHeaderNode(_) => {
                "[label=\"CHN\" color=\"black\"];".to_string()
            }
            Node::ValueNode(_) => {
                "[label=\"VN\" color=\"grey\"];".to_string()
            }
            Node::KeyNode(_) => {
                panic!("KeyNode should not be annotated with default attributes")
            }
            Node::PointerNode(_) => {
                "[label=\"PN\" color=\"orange\"];".to_string()
                
            }
        }
    }

    /// get the address of the node
    pub fn get_address(&self) -> u64 {
        match self {
            SpecialNodeAnnotation::SessionStateNodeAnnotation(addr) => {
                *addr
            }
            SpecialNodeAnnotation::SshStructNodeAnnotation(addr) => {
                *addr
            }
            SpecialNodeAnnotation::KeyNodeAnnotation(addr) => {
                *addr
            }
            SpecialNodeAnnotation::SessionStateAndSSHStructNodeAnnotation(addr) => {
                *addr
            }
            SpecialNodeAnnotation::KeyNodeAndSessionStateNodeAnnotation(addr) => {
                *addr
            }
        }
    }
}

impl Debug for SpecialNodeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialNodeAnnotation::SessionStateNodeAnnotation(addr) => {
                write!(
                    f, "SSN({:#x})", 
                    addr,
                )
            }
            SpecialNodeAnnotation::SshStructNodeAnnotation(addr) => {
                write!(
                    f, "SSHN({:#x})", 
                    addr,
                )
            }
            SpecialNodeAnnotation::KeyNodeAnnotation(addr) => {
                write!(
                    f, "KN({:#x})", 
                    addr,
                )
            }
            SpecialNodeAnnotation::SessionStateAndSSHStructNodeAnnotation(addr) => {
                write!(
                    f, "SSN_SSHN({:#x})", 
                    addr,
                )
            }
            SpecialNodeAnnotation::KeyNodeAndSessionStateNodeAnnotation(addr) => {
                write!(
                    f, "KN_SSN({:#x})", 
                    addr,
                )
            }
        }
    }
}

impl Node {
    /// Check whether a node is important or not.
    #[cfg(test)]
    pub fn is_important(&self) -> bool {
        match self {
            Node::KeyNode(_) => true,
            _ => false,
        }
    }

    /// NOTE: If you forget to match a new Node variant, this function will panic.
    pub fn get_address(&self) -> u64 {
        match self {
            Node::ChunkHeaderNode(chunk_header_node) => {
                chunk_header_node.addr
            }
            Node::ValueNode(base_value_node) => {
                base_value_node.addr
            }
            Node::KeyNode(key_node) => {
                key_node.addr
            }
            Node::PointerNode(base_pointer_node) => {
                base_pointer_node.addr
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
            Node::KeyNode(key_node) => {
                format!(
                    "KN_{}({:#x})", 
                    key_node.key_data.name, key_node.addr
                )
            }
            Node::PointerNode(base_pointer_node) => {
                format!(
                    "PN({:#x})",
                    base_pointer_node.addr,
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

    pub fn is_key(&self) -> bool {
        match self {
            Node::KeyNode(_) => true,
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
            Node::KeyNode(key_node) => {
                Some(key_node.value.clone())
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
            Node::KeyNode(key_node) => {
                Some(key_node.chn_addr)                
            },
            Node::PointerNode(base_pointer_node) => {
                Some(base_pointer_node.chn_addr)
            }
            _ => None,
        }
    }
}

    /// return a formatted string of the node, for debugging purposes
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::ChunkHeaderNode(chunk_header_node) => {
                write!(
                    f, "CHN: {} [VNs: {} PNs: {}]", 
                    chunk_header_node.addr, chunk_header_node.nb_value_nodes, chunk_header_node.nb_pointer_nodes
                )
            }
            Node::ValueNode(base_value_node) => {
                write!(
                    f, "VN: {} [value=\"{}\"]", 
                    base_value_node.addr, hex::encode(&base_value_node.value)
                )
            }
            Node::KeyNode(key_node) => {
                write!(
                    f, "KN: {} [found_key=\"{}\" json_key=\"{}\"]", 
                    key_node.addr, hex::encode(&key_node.key), hex::encode(&key_node.key_data.key) 
                )
            }
            Node::PointerNode(base_pointer_node) => {
                write!(
                    f, "PN: {} [label=\"{}\"]", 
                    base_pointer_node.addr, base_pointer_node.points_to
                )
                    
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkHeaderNode {
    pub addr: u64,
    pub byte_size: usize,
    pub flags: HeaderFlags,
    pub is_free: bool,
    pub nb_pointer_nodes: usize,
    pub nb_value_nodes: usize,
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

// Key data from JSON file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyData {
    pub name: String,
    pub key: Vec<u8>,
    pub addr: u64,
    pub len: usize,
    pub real_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyNode {
    pub addr: u64,
    pub chn_addr: u64,
    pub value: [u8; BLOCK_BYTE_SIZE], // first block of key
    pub key: Vec<u8>, // found in heap dump, full key (not just the first block)
    pub key_data: KeyData, // found in JSON file
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
            Node::KeyNode(_) => {
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
        }
    } 
}

