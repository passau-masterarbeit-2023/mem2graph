use serde_derive::{Serialize, Deserialize};

use crate::params::BLOCK_BYTE_SIZE;


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerNode{
    BasePointerNode(BasePointerNode),
    SessionStateNode(SessionStateNode),
    SshStructNode(SshStructNode),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueNode{
    BaseValueNode(BaseValueNode),
    KeyNode(KeyNode),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Node {
    DataStructureNode(DataStructureNode),
    ValueNode(ValueNode),
    PointerNode(PointerNode),
}

impl Node {
    /// Check whether a node is important or not.
    pub fn is_important(&self) -> bool {
        match self {
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::KeyNode(_) => true,
                    _ => false,
                }
            }
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::SessionStateNode(_) => true,
                    PointerNode::SshStructNode(_) => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// NOTE: If you forget to match a new Node variant, this function will panic.
    pub fn get_address(&self) -> u64 {
        match self {
            Node::DataStructureNode(data_structure_node) => {
                data_structure_node.addr
            }
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(base_value_node) => {
                        base_value_node.addr
                    }
                    ValueNode::KeyNode(key_node) => {
                        key_node.addr
                    }
                }
            }
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::BasePointerNode(base_pointer_node) => {
                        base_pointer_node.addr
                    }
                    PointerNode::SessionStateNode(session_state_node) => {
                        session_state_node.addr
                    }
                    PointerNode::SshStructNode(ssh_struct_node) => {
                        ssh_struct_node.addr
                    }
                }
            }
        }
    }

    /// returns the address of the node and its type annotation
    pub fn str_addr_and_type(&self) -> String {
        match self {
            Node::DataStructureNode(data_structure_node) => {
                format!(
                    "DTN({:#x})",
                    data_structure_node.addr,
                )
            }
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(base_value_node) => {
                        format!(
                            "VN({:#x})",
                            base_value_node.addr,
                        )
                    }
                    ValueNode::KeyNode(key_node) => {
                        format!(
                            "KN_{}({:#x})", 
                            key_node.key_data.name, key_node.addr
                        )
                    }
                }
            }
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::BasePointerNode(base_pointer_node) => {
                        format!(
                            "PN({:#x})",
                            base_pointer_node.addr,
                        )
                    }
                    PointerNode::SessionStateNode(session_state_node) => {
                        format!(
                            "SSN({:#x})", 
                            session_state_node.addr
                        )
                    }
                    PointerNode::SshStructNode(ssh_struct_node) => {
                        format!(
                            "SSHN({:#x})", 
                            ssh_struct_node.addr
                        )
                    }
                }
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

    pub fn points_to(&self) -> Option<u64> {
        match self {
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::BasePointerNode(base_pointer_node) => {
                        Some(base_pointer_node.points_to)
                    }
                    PointerNode::SessionStateNode(session_state_node) => {
                        Some(session_state_node.points_to)
                    }
                    PointerNode::SshStructNode(ssh_struct_node) => {
                        Some(ssh_struct_node.points_to)
                    }
                }
            }
            _ => None,
        }
    }

    pub fn get_value(&self) -> Option<[u8; BLOCK_BYTE_SIZE]> {
        match self {
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(base_value_node) => {
                        Some(base_value_node.value.clone())
                    }
                    ValueNode::KeyNode(key_node) => {
                        Some(key_node.value.clone())
                    }
                }
            }
            _ => None,
        }
    }
}

    /// return a formatted string of the node, for debugging purposes
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::DataStructureNode(data_structure_node) => {
                write!(
                    f, "DTN: {} [VNs: {} PNs: {}]", 
                    data_structure_node.addr, data_structure_node.nb_value_nodes, data_structure_node.nb_pointer_nodes
                )
            }
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(base_value_node) => {
                        write!(
                            f, "VN: {} [value=\"{}\"]", 
                            base_value_node.addr, hex::encode(&base_value_node.value)
                        )
                    }
                    ValueNode::KeyNode(key_node) => {
                        write!(
                            f, "KN: {} [found_key=\"{}\" json_key=\"{}\"]", 
                            key_node.addr, hex::encode(&key_node.key), hex::encode(&key_node.key_data.key) 
                        )
                    }
                }
            }
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::BasePointerNode(base_pointer_node) => {
                        write!(
                            f, "PN: {} [label=\"{}\"]", 
                            base_pointer_node.addr, base_pointer_node.points_to
                        )
                    }
                    PointerNode::SessionStateNode(session_state_node) => {
                        write!(
                            f, "SSN: {} [label=\"{}\"]", 
                            session_state_node.addr, session_state_node.points_to
                        )
                    }
                    PointerNode::SshStructNode(ssh_struct_node) => {
                        write!(
                            f, "SSHN: {} [label=\"{}\"]", 
                            ssh_struct_node.addr, ssh_struct_node.points_to
                        )
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataStructureNode {
    pub addr: u64,
    pub byte_size: usize,
    pub nb_pointer_nodes: usize,
    pub nb_value_nodes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseValueNode {
    pub addr: u64,
    pub value: [u8; BLOCK_BYTE_SIZE],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasePointerNode {
    pub addr: u64,
    pub points_to: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStateNode {
    pub addr: u64,
    pub points_to: u64,
}

impl SessionStateNode {
    /// build a new SessionStateNode from an old one
    /// WARN: the old node must be a PointerNode
    pub fn new(old_node: &Node) -> Self {
        assert!(old_node.is_pointer());
        Self {
            addr: old_node.get_address(),
            points_to: old_node.points_to().unwrap(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshStructNode {
    pub addr: u64,
    pub points_to: u64,
}

impl SshStructNode {
    /// build a new SshStructNode from an old one
    /// WARN: the old node must be a PointerNode
    pub fn new(old_node: &Node) -> Self {
        assert!(old_node.is_pointer());
        Self {
            addr: old_node.get_address(),
            points_to: old_node.points_to().unwrap(),
        }
    }
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
    pub value: [u8; BLOCK_BYTE_SIZE], // first block of key
    pub key: Vec<u8>, // found in heap dump, full key (not just the first block)
    pub key_data: KeyData, // found in JSON file
}

pub const DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT: usize = 1;

pub struct Edge {
    pub from: u64,
    pub to: u64,
    pub edge_type: EdgeType,
    pub weight: usize, // Number of edge pointers between the two nodes, default is 1 for a DataStructure edge.
}

pub enum EdgeType {
    DataStructureEdge,
    PointerEdge,
}


impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeType::DataStructureEdge => write!(f, "dts"),
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

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::DataStructureNode(_) => {
                write!(
                    f, "    {:?} [color=blue];", 
                    self.str_addr_and_type(),
                )
            }
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(_) => {
                        write!(
                            f, "    {:?} [color=grey];", 
                            self.str_addr_and_type(),
                        )
                    }
                    ValueNode::KeyNode(_) => {
                        write!(
                            f, "    {:?} [color=green, style=filled];", 
                            self.str_addr_and_type(),
                        )
                    }
                }
            }
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::BasePointerNode(_) => {
                        write!(
                            f, "    {:?} [color=orange];",
                            self.str_addr_and_type(),
                        )
                    }
                    PointerNode::SessionStateNode(_) => {
                        write!(
                            f, "    {:?} [color=red];", 
                            self.str_addr_and_type(),
                        )
                    }
                    PointerNode::SshStructNode(_) => {
                        write!(
                            f, "    {:?} [color=purple];", 
                            self.str_addr_and_type(),
                        )
                    }
                }
            }
        }
    } 
}

