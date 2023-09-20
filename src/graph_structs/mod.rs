use serde_derive::{Serialize, Deserialize};

use crate::params::BLOCK_BYTE_SIZE;


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerNode{
    BasePointerNode(BasePointerNode),
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

/// Anotations for special nodes used in graph_data.
/// Allow labelling for embedding, and attribute and coloring for graph generation.
/// NOTE : Take the address of the node as parameter, to permit to display it in the label.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialNodeAnnotation {
    SessionStateNodeAnnotation(u64),
    SshStructNodeAnnotation(u64),
    KeyNodeAnnotation(u64),
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
            SpecialNodeAnnotation::KeyNodeAnnotation(_) => {
                "KN".to_string()
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
            SpecialNodeAnnotation::KeyNodeAnnotation(_) => {
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
            Node::DataStructureNode(_) => {
                "[label=\"DTN\" color=\"black\"];".to_string()
            }
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(_) => {
                        "[label=\"VN\" color=\"grey\"];".to_string()
                    }
                    ValueNode::KeyNode(_) => {
                        panic!("KeyNode should not be annotated with default attributes")
                    }
                }
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
        }
    }
}


impl Node {
    /// Check whether a node is important or not.
    #[cfg(test)]
    pub fn is_important(&self) -> bool {
        match self {
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::KeyNode(_) => true,
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

    /// Check if a node is a data structure node
    pub fn is_dtn(&self) -> bool {
        match self {
            Node::DataStructureNode(_) => true,
            _ => false,
        }
    }

    pub fn is_key(&self) -> bool {
        match self {
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::KeyNode(_) => true,
                    _ => false,
                }
            }
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

    /// returns the dtn address of the node
    /// if the node is not a value node, a key node or a pointer node, returns None
    pub fn get_dtn_addr(&self) -> Option<u64> {
        match self {
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(base_value_node) => {
                        Some(base_value_node.dtn_addr)
                    }
                    ValueNode::KeyNode(key_node) => {
                        Some(key_node.dtn_addr)
                    }
                }
            },
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::BasePointerNode(base_pointer_node) => {
                        Some(base_pointer_node.dtn_addr)
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
    pub dtn_addr: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasePointerNode {
    pub addr: u64,
    pub points_to: u64,
    pub dtn_addr: u64,
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
    pub dtn_addr: u64,
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

/// display the name of the node (without the annotation)
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::DataStructureNode(_) => {
                write!(
                    f, "    {:?}", 
                    self.str_addr_and_type(),
                )
            }
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(_) => {
                        write!(
                            f, "    {:?}", 
                            self.str_addr_and_type(),
                        )
                    }
                    ValueNode::KeyNode(_) => {
                        write!(
                            f, "    {:?}", 
                            self.str_addr_and_type(),
                        )
                    }
                }
            }
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::BasePointerNode(_) => {
                        write!(
                            f, "    {:?}",
                            self.str_addr_and_type(),
                        )
                    }
                }
            }
        }
    } 
}
