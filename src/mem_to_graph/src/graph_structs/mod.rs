use serde_derive::{Serialize, Deserialize};

# [derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerNode{
    BasePointerNode(BasePointerNode),
    SessionStateNode(SessionStateNode),
    SshStructNode(SshStructNode),
}

# [derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueNode{
    BaseValueNode(BaseValueNode),
    KeyNode(KeyNode),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Node {
    DataStructureNode(DataStructureNode),
    ValueNode(ValueNode),
    PointerNode(PointerNode),
}

impl Node {
    pub fn is_important(&self) -> bool {
        is_important_node(&self)
    }
}

pub fn is_important_node(node: &Node) -> bool {
    match node {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataStructureNode {
    pub addr: u64,
    pub byte_size: usize,
    pub nb_pointer_nodes: usize,
    pub nb_value_nodes: usize,
    pub color: String,
    pub style: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseValueNode {
    pub addr: u64,
    pub value: Vec<u8>,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasePointerNode {
    pub addr: u64,
    pub points_to: u64,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStateNode {
    pub addr: u64,
    pub points_to: u64,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshStructNode {
    pub addr: u64,
    pub points_to: u64,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyData {
    pub name: String,
    pub key: Vec<u8>,
    pub addr: Vec<u8>,
    pub len: usize,
    pub real_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyNode {
    pub addr: u64,
    pub value: Vec<u8>,
    pub key: Vec<u8>,
    pub key_data: KeyData,
    pub color: String,
}

pub enum Edge {
    DataStructure,
    Pointer,
}


impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Edge::DataStructure => write!(f, "dts"),
            Edge::Pointer => write!(f, "ptr"),
        }
    }
}
