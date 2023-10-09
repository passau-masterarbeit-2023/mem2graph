use crate::graph_embedding::GraphEmbedding;
use crate::graph_embedding::utils_embedding::get_node_label;
use crate::graph_structs::Node;
use crate::params::BLOCK_BYTE_SIZE;

use super::value_node_semantic_embedding::generate_value_sample;

/// generate chunk top value node semantic embedding
/// NOTE: A Value Node is a 8 byte block in the context of the memory graph.
///    - parent chn address
///    - position in the chunk              WARN: In that case, should always be the same
///    - nb pointer nodes in the chunk
///    - nb value nodes in the chunk
/// 
///   - ancestor (in order of depth, alternate CHN/PTR)
/// Labels [0.0, 1.0, ..., 0.0], for key prediction (1.0 indicates that the block contains a key)
/// 
/// Process:
///     1. Iterate over the chunks (CHN) of the heap dump
///     2. For each chunk, check first user data block is VN
///     3. If yes, generate the embedding
pub fn generate_chunk_top_vn_semantic_embedding(
    graph_embedding : &GraphEmbedding
) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut samples = Vec::new();
    let mut labels = Vec::new();

    for chn_addr in graph_embedding.graph_annotate.graph_data.chn_addrs.iter() {

        // entropy filter
        if graph_embedding.is_entropy_filtered_addr(chn_addr) {
            continue;
        }

        // check if the first block of the user data section is a value node
        let first_user_block_addr = chn_addr + BLOCK_BYTE_SIZE as u64;
        let obtained_first_user_block = graph_embedding.graph_annotate.graph_data.addr_to_node.get(&first_user_block_addr);
        if obtained_first_user_block.is_none() {
            panic!("The first user data block of the chunk is not in the graph, at address {:#x}", first_user_block_addr);
        }
        let first_user_block_node: &Node = obtained_first_user_block.unwrap();
        match first_user_block_node {
            Node::ValueNode(_) => {},
            _ => continue, // skip this chunk as its first block is not a value node
        }

        // perform embedding on this value node
        let sample = generate_value_sample(graph_embedding, first_user_block_addr);
        let label = get_node_label(graph_embedding, first_user_block_addr);

        samples.push(sample);
        labels.push(label);
    }
    
    (samples, labels)
}