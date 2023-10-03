use crate::graph_structs::Node;

use super::{GraphEmbedding, utils_embedding::get_node_label, neighboring::get_neighbors};

/// generate semantic embedding of the nodes
/// Samples [
///     [0.3233, ..., 0.1234],
///     [0.1234, ..., 0.1234],
///     [0.1234, ..., 0.1234],
///     ... 
/// ]
/// 
/// Labels [0.0, 1.0, ..., 0.0],
pub fn generate_value_node_semantic_embedding(graph_embedding : &GraphEmbedding) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut samples = Vec::new();
    let mut labels = Vec::new();

    for addr in graph_embedding.graph_annotate.graph_data.value_node_addrs.iter() {
        let sample = generate_value_sample(graph_embedding, *addr);
        let label = get_node_label(graph_embedding, *addr);

        // skip trivial samples (if param is set)
        if *crate::params::REMOVE_TRIVIAL_ZERO_SAMPLES &&
            sample.ends_with(&vec![0; ((graph_embedding.depth - 1) * 2) - 1]) && 
            label == 0
        {
            continue;
        }

        samples.push(sample);
        labels.push(label);
    }
    
    (samples, labels)
}

/// get the semantics data from the parent chunk of a node
fn add_features_from_parent_chunk(graph_embedding : &GraphEmbedding, addr: u64) -> Vec<usize> {
    let mut feature: Vec<usize> = Vec::new();

    let node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
    let parent_chn_node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(
        &node.get_parent_chn_addr().unwrap()
    ).unwrap();

    // add features from parent chn node
    match parent_chn_node {
        Node::ChunkHeaderNode(chunk_header_node) => {
            feature.push(chunk_header_node.byte_size);
            feature.push(((node.get_address() - chunk_header_node.addr) / crate::params::BLOCK_BYTE_SIZE as u64) as usize);
            feature.push(chunk_header_node.nb_pointer_nodes);
            feature.push(chunk_header_node.nb_value_nodes);
        },
        _ => // if the node is not in a chunk, we return a vector of 0
            feature.append(&mut vec![0; 3]),
    }

    feature
}

/// generate the value embedding of a value node
fn generate_value_sample(graph_embedding : &GraphEmbedding, addr: u64) -> Vec<usize> {
    let mut feature: Vec<usize> = add_features_from_parent_chunk(graph_embedding, addr);
    let mut ancestor_features = get_neighbors(graph_embedding, vec![addr].into_iter().collect(), petgraph::Direction::Incoming);
    

    feature.append(&mut ancestor_features); // ancestor_feature is left empty
    feature
}