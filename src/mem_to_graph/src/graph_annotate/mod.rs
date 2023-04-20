use crate::{graph_data::GraphData, graph_structs::{PointerNode, SshStructNode, Node, SessionStateNode, ValueNode, KeyNode}};
use std::path::{PathBuf};

pub struct GraphAnnotate {
    pub graph_data: GraphData,
}

impl GraphAnnotate {
    pub fn new(heap_dump_raw_file_path: PathBuf, pointer_byte_size: usize) -> GraphAnnotate {
        let graph_data = GraphData::new(heap_dump_raw_file_path, pointer_byte_size);
        
        let mut graph_annotate = GraphAnnotate {
            graph_data,
        };
        graph_annotate.annotate();
        graph_annotate
    }

    /// Annotate the graph with data from the JSON file
    /// stored in heap_dump_data
    fn annotate(&mut self) {

        self.annotate_graph_with_key_data();
        //self.annotate_graph_with_special_ptr(); // TODO: fix this
    }

    /// annotate graph with ptr from json file
    fn annotate_graph_with_special_ptr(&mut self) {
        {
            // SSH_STRUCT_ADDR
            let ssh_struct_addr = self.graph_data.heap_dump_data.as_ref().unwrap().addr_ssh_struct;
            let old_node: Option<&Node> = self.graph_data.addr_to_node.get(&ssh_struct_addr);
            if old_node.is_some() {
                let new_node = Node::PointerNode(PointerNode::SshStructNode(
                    SshStructNode::new(old_node.unwrap())
                ));
    
                // replace old node with new node in the map
                self.graph_data.addr_to_node.insert(ssh_struct_addr, new_node);
                log::info!("ssh_struct_addr ({}) found.", ssh_struct_addr)
            } else {
                log::info!("ssh_struct_addr ({}) not found.", ssh_struct_addr)
            }
        }
        {
            // SESSION_STATE_ADDR
            let session_state_addr = self.graph_data.heap_dump_data.as_ref().unwrap().addr_session_state;
            let old_node: Option<&Node> = self.graph_data.addr_to_node.get(&session_state_addr);
            if old_node.is_some() {
                let new_node = Node::PointerNode(PointerNode::SessionStateNode(
                    SessionStateNode::new(old_node.unwrap())
                ));
    
                // replace old node with new node in the map
                self.graph_data.addr_to_node.insert(session_state_addr, new_node);
                log::info!("session_state_addr ({}) found.", session_state_addr)
            } else {
                log::info!("session_state_addr ({}) not found.", session_state_addr)
            }
        }
    }

    fn annotate_graph_with_key_data(&mut self) {
        // iterate over all the key data
        for (addr, key_data) in &self.graph_data.heap_dump_data.as_ref().unwrap().addr_to_key_data {
            // get the node at the key_data's address
            let node: Option<&Node> = self.graph_data.addr_to_node.get(addr);
            if node.is_some() && node.unwrap().is_value() {
                // if the node is a ValueNode, then we can annotate it
                // i.e. we create a KeyNode from the Node and its key_data
                let mut aggregated_key: Vec<u8> = Vec::new();

                // get all the ValueNodes that are part of the key
                let block_size = self.graph_data.heap_dump_data.as_ref().unwrap().block_size;
                for i in 0..(key_data.len / block_size) {
                    let current_key_block_addr = addr + (i * block_size) as u64;
                    let current_key_block_node: Option<&Node> = self.graph_data.addr_to_node.get(&current_key_block_addr);
                    if current_key_block_node.is_some() {
                        aggregated_key.extend_from_slice(&current_key_block_node.unwrap().get_value().unwrap());
                    } else {
                        // log warning
                        log::warn!(
                            "current_key_block_node not found for addr: {}, for key {}", 
                            current_key_block_addr, key_data.name
                        );
                        break;
                    }
                    
                }
                
                // annotate if the key found in the heap dump is the same as the key found in the json file
                if aggregated_key == key_data.key {
                    // replace the ValueNode with a KeyNode
                    let key_node = Node::ValueNode(ValueNode::KeyNode(KeyNode {
                        addr: *addr, // addr of first block of key
                        dtn_addr: node.unwrap().get_dtn_addr().unwrap(), // dtn_addr of first block of key
                        value: node.unwrap().get_value().unwrap(), // first block value of key
                        key: aggregated_key, // found in heap dump, full key (not just the first block)
                        key_data: key_data.clone(), // found in heap dump, key data
                    }));
                    self.graph_data.addr_to_node.insert(*addr, key_node);
                } else {
                    log::warn!(
                        "key ({}) found in heap dump is not the same as the key found in the json file.  
                        found aggregated_key: {:?}, 
                        expected key_data.key: {:?}", 
                        key_data.name, key_data.key, aggregated_key
                    );
                }

            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{self};
    use crate::graph_structs::{
        Node, 
        ValueNode, 
        PointerNode, 
    };

    #[test]
    fn test_annotation() {
        crate::tests::setup();

        let graph_annotate = GraphAnnotate::new(
            params::TEST_HEAP_DUMP_FILE_PATH.clone(), 
            params::BLOCK_BYTE_SIZE
        );

        // check that there is at least one SSH_STRUCT node
        assert!(graph_annotate.graph_data.addr_to_node.values().any(|node| {
            if let Node::PointerNode(PointerNode::SshStructNode(_)) = node {
                true
            } else {
                false
            }
        }));

        // NOTE: We have no SESSION_STATE node in the test heap dump file
        // we don't really know why.
        // TODO: Find out why there is no SESSION_STATE node in the test heap dump file
    }

    #[test]
    fn test_key_annotation() {
        crate::tests::setup();

        let graph_annotate = GraphAnnotate::new(
            params::TEST_HEAP_DUMP_FILE_PATH.clone(), 
            params::BLOCK_BYTE_SIZE
        );

        // check that there is at least one KeyNode
        assert!(graph_annotate.graph_data.addr_to_node.values().any(|node| {
            if let Node::ValueNode(ValueNode::KeyNode(_)) = node {
                true
            } else {
                false
            }
        }));

        // check the last key
        let test_key_node = graph_annotate.graph_data.addr_to_node.get(&*crate::tests::TEST_KEY_F_ADDR).unwrap();
        match test_key_node {
            Node::ValueNode(ValueNode::KeyNode(key_node)) => {
                assert_eq!(key_node.key, *crate::tests::TEST_KEY_F_BYTES);
                assert_eq!(key_node.key_data.name, *crate::tests::TEST_KEY_F_NAME);
                assert_eq!(key_node.key_data.len, *crate::tests::TEST_KEY_F_LEN);
            },
            _ => panic!("Node is not a KeyNode"),
        }
    }

}