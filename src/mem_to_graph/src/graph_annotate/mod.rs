use crate::{graph_data::GraphData, graph_structs::{PointerNode, SshStructNode, Node, SessionStateNode}};
use std::path::{PathBuf};

pub struct GraphAnnotate {
    graph_data: GraphData,
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
        self.annotate_graph_with_ptr();
    }

    /// annotate graph with ptr from json file
    fn annotate_graph_with_ptr(&mut self) {
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
        //
    }
}

#[cfg(test)]
mod tests {

    use log;
    use petgraph::dot::Dot;

    use super::*;
    use crate::params::{self, TEST_HEAP_DUMP_FILE_PATH, PTR_ENDIANNESS};
    use crate::graph_structs::{
        Node, 
        ValueNode, 
        PointerNode, 
        BaseValueNode, 
        BasePointerNode, 
        DataStructureNode,
        Edge,
        EdgeType,
        DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT,
    };
    use crate::tests::*;
    use crate::utils::create_node_from_bytes;

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

}