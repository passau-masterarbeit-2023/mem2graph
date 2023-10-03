use crate::{graph_structs::Node, params::BLOCK_BYTE_SIZE, utils::{to_n_bits_binary, u64_to_bytes}};

use super::GraphEmbedding;

 /// extract the data of the chunk :
/// get all the bit of the chunk as u8
pub fn extract_chunk_data_as_bytes(graph_embedding : &GraphEmbedding, addr: u64, block_size : usize) -> Vec<u8> {
    let mut data = Vec::new();
    let node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

    match node {
        Node::ChunkHeaderNode(chunk_header_node) => {
            let mut current_addr = chunk_header_node.addr + block_size as u64;

            // get the data of the chunk
            for _ in 1..(chunk_header_node.byte_size/8) {
                // get the node at the current address
                let node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(&current_addr).unwrap();
                // if the block is a pointer
                if node.is_pointer() {
                    let bits = u64_to_bytes(node.points_to().unwrap());
                    data.extend_from_slice(&bits);
                    
                } else {
                    let current_block = node.get_value().unwrap();
                    data.extend_from_slice(&current_block);
                }
                
                current_addr += block_size as u64;
            }
        },
        _ => panic!("Node is not a chunk"),
    }
    data
}


/// extract the data of the chunk :
/// get all the bit of the chunk as char ('1' or 'O')
pub fn extract_chunk_data_as_bits(graph_embedding : &GraphEmbedding, addr: u64) -> Vec<char> {
    let mut data = Vec::new();
    let node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

    match node {
        Node::ChunkHeaderNode(chunk_header_node) => {
            let mut current_addr = chunk_header_node.addr + BLOCK_BYTE_SIZE as u64;
            let block_size_bit = (BLOCK_BYTE_SIZE * 8) as usize;

            // get the data of the chunk
            for _ in 1..(chunk_header_node.byte_size/8) {
                // get the node at the current address
                let node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(&current_addr).unwrap();
                // if the block is a pointer
                if node.is_pointer() {
                    let mut bits = to_n_bits_binary(node.points_to().unwrap(), block_size_bit).chars().collect();
                    data.append(&mut bits);
                    
                } else {
                    let current_block = node.get_value().unwrap();
                    
                    // convert the block to binary
                    for i in 0..BLOCK_BYTE_SIZE {
                        // each value of the array are bytes, so 8 bit long
                        let mut bits = to_n_bits_binary(current_block[i] as u64, 8).chars().collect();
                        data.append(&mut bits);
                    }
                }
                
                current_addr += BLOCK_BYTE_SIZE as u64;
            }
        },
        _ => panic!("Node is not a chunk"),
    }
    data
}

/// get the label of a node
pub fn get_node_label(graph_embedding : &GraphEmbedding, addr : u64) -> usize {
    let annotation = graph_embedding.graph_annotate.graph_data.node_addr_to_annotations.get(&addr);
    match annotation {
        Some(annotation) => {
            annotation.annotation_set_embedding() as usize
        },
        None => 0,
    }
}

/// extract the basics information of the chunk
/// Return [addr, size, nb_pointer]
pub fn get_chunk_basics_informations(graph_embedding : &GraphEmbedding, addr: u64) -> Vec<usize> {
    let mut info = Vec::new();
    let node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

    // add features from parent chn node
    match node {
        Node::ChunkHeaderNode(chunk_header_node) => {
            info.push(chunk_header_node.addr.try_into().expect("addr overflow in embedding")); // WARN : can be overflow !!!!!!
            info.push(chunk_header_node.byte_size);
            info.push(chunk_header_node.nb_pointer_nodes);
        },
        _ => panic!("Node is not a chunk"),
    }
    info
}