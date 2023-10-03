use std::{fmt::Debug, collections::HashSet};
use serde_derive::{Serialize, Deserialize};
use crate::graph_structs::Node;

/// Anotations for special nodes, that comes from JSON annotation file.
/// Allow labelling for embedding, and attribute and coloring for graph generation.
/// NOTE : Take the address of the node as parameter, to permit to display it in the label.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeAnnotation {
    SessionStateNodeAnnotation(u64),
    SshStructNodeAnnotation(u64),
    KeyAnnotation(KeyAnnotation),
}

impl NodeAnnotation {
    /// Get address of the annotated node
    pub fn get_address(&self) -> u64 {
        match self {
            NodeAnnotation::SessionStateNodeAnnotation(addr) => {
                *addr
            }
            NodeAnnotation::SshStructNodeAnnotation(addr) => {
                *addr
            }
            NodeAnnotation::KeyAnnotation(annotation_data) => {
                annotation_data.addr
            }
        }
    }
}

impl Debug for NodeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeAnnotation::SessionStateNodeAnnotation(addr) => {
                write!(
                    f, "SSN({:#x})", 
                    addr,
                )
            }
            NodeAnnotation::SshStructNodeAnnotation(addr) => {
                write!(
                    f, "SSHN({:#x})", 
                    addr,
                )
            }
            NodeAnnotation::KeyAnnotation(annotation_data) => {
                write!(
                    f, "KN({:#x})", 
                    annotation_data.addr,
                )
            }
        }
    }
}

/// Binary embedding of annotations
/// correspond to the coin trick (using binary)
enum AnnotationSubclass {
    Key = 0x1,
    SshStruct = 0x2,
    SessionState = 0x4,
}

impl AnnotationSubclass {
    pub fn is_key_subclass(class: u8) -> bool {
        class & AnnotationSubclass::Key as u8 != 0
    }

    pub fn is_ssh_struct_subclass(class: u8) -> bool {
        class & AnnotationSubclass::SshStruct as u8 != 0
    }

    pub fn is_session_state_subclass(class: u8) -> bool {
        class & AnnotationSubclass::SessionState as u8 != 0
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationSet {
    annotations: HashSet<NodeAnnotation>,
}

impl AnnotationSet {
    /// create a new embedding value from a set of annotations
    /// The embedding technique uses the coin trick (using binary)
    ///     class 0: no annotation
    ///     class 1: key annotation
    ///     class 2: ssh struct annotation
    /// Subsequent classes contain the sum of the previous classes
    /// which means the their respective bits are set to 1 when needed.
    pub fn annotation_set_embedding(&self) -> u8 {
        let mut embedding_class = 0;
        for annotation in self.annotations.iter() {
            match annotation {
                // tricks of coins (using binary)
                NodeAnnotation::SessionStateNodeAnnotation(_) => {
                    embedding_class += AnnotationSubclass::SessionState as u8;
                }
                NodeAnnotation::SshStructNodeAnnotation(_) => {
                    embedding_class += AnnotationSubclass::SshStruct as u8;
                }
                NodeAnnotation::KeyAnnotation(_) => {
                    embedding_class += AnnotationSubclass::Key as u8;
                }
            }
        }
        embedding_class
    }

    pub fn new(annotation: NodeAnnotation) -> AnnotationSet {
        let mut set = HashSet::new();
        set.insert(annotation);
        AnnotationSet {
            annotations: set,
        }
    }

    /// Add annotation to the set
    /// If the annotation is already in the set, it is not added
    /// If the annotation is not the first one, check that
    /// its address is the same as the first one
    pub fn add_annotation(&mut self, annotation: NodeAnnotation) {
        if self.annotations.is_empty() {
            self.annotations.insert(annotation);
        } else {
            let first_annotation = self.annotations.iter().next().unwrap();
            assert!(first_annotation.get_address() == annotation.get_address());
            self.annotations.insert(annotation);
        }
    }

    // wrapper for subclass functions
    #[allow(dead_code)]
    pub fn is_key_subclass(&self) -> bool {
        AnnotationSubclass::is_key_subclass(self.annotation_set_embedding())
    }

    #[allow(dead_code)]
    pub fn is_ssh_struct_subclass(&self) -> bool {
        AnnotationSubclass::is_ssh_struct_subclass(self.annotation_set_embedding())
    }

    #[allow(dead_code)]
    pub fn is_session_state_subclass(&self) -> bool {
        AnnotationSubclass::is_session_state_subclass(self.annotation_set_embedding())
    }

    fn get_name(&self) -> String {
        let class = self.annotation_set_embedding();
        match (
            AnnotationSubclass::is_key_subclass(class),
            AnnotationSubclass::is_ssh_struct_subclass(class),
            AnnotationSubclass::is_session_state_subclass(class),
        ) {
            (true, false, false) => {
                "Key".to_string()
            }
            (false, true, false) => {
                "Ssh".to_string()
            }
            (false, false, true) => {
                "SST".to_string()
            }
            (false, true, true) => {
                "Ssh_SST".to_string()
            }
            (true, false, true) => {
                "Key_SST".to_string()
            }
            _ => {
                panic!("Unhandled annotation class combination of subclasses!")
            }
        } 

        
    }

    fn get_color(&self) -> String {
        let class = self.annotation_set_embedding();
        match (
            AnnotationSubclass::is_key_subclass(class),
            AnnotationSubclass::is_ssh_struct_subclass(class),
            AnnotationSubclass::is_session_state_subclass(class),
        ) {
            (true, false, false) => {
                "green".to_string()
            }
            (false, true, false) => {
                "red".to_string()
            }
            (false, false, true) => {
                "blue".to_string()
            }
            (false, true, true) => {
                "purple".to_string() // red + blue => purple
            }
            (true, false, true) => {
                "cyan".to_string() // green + blue => cyan
            }
            _ => {
                panic!("Unhandled annotation class combination of subclasses!")
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
            Node::PointerNode(_) => {
                "[label=\"PN\" color=\"orange\"];".to_string()
            }
            Node::FooterNode(_) => {
                "[label=\"FN\" color=\"purple\"];".to_string()
            }
        }
    }

    /// get the address of the node
    #[allow(dead_code)]
    pub fn get_address(&self) -> u64 {
        let first_annotation = self.annotations.iter().next().unwrap();
        match first_annotation {
            NodeAnnotation::SessionStateNodeAnnotation(addr) => {
                *addr
            }
            NodeAnnotation::SshStructNodeAnnotation(addr) => {
                *addr
            }
            NodeAnnotation::KeyAnnotation(annotation_data) => {
                annotation_data.addr
            }
        }
    }
}

// Key data from JSON file
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyDataJSON {
    pub name: String,
    pub key: Vec<u8>,
    pub addr: u64,
    pub len: usize,
    pub real_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyAnnotation {
    pub addr: u64, // address of annotated node
    pub key: Vec<u8>, // found in heap dump, full key (not just the first block)
    pub key_data: KeyDataJSON, // found in JSON file
}