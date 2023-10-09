use std::hash::Hash;

use crate::graph_embedding::{GraphEmbedding, utils_embedding::{get_chunk_basics_informations, get_node_label}, neighboring::generate_samples_for_neighbor_nodes_of_the_chunk};


/// generate semantic embedding of all the chunks
/// in order :
///     - chunk header addresse (not really usefull for learning, but can bu usefull to further analyse the data)
///     - chunk size
///     - nb pointer
/// 
///     - ancestor (in order of depth, alternate CHN/PTR)
///     - children (same)
///     - label (if the chunk contains a key, or is the ssh or sessionState)
pub fn generate_chunk_semantic_embedding(
    graph_embedding : &GraphEmbedding
) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut samples = Vec::new();
    let mut labels = Vec::new();

    // get chunk :
    for chn_addr in graph_embedding.graph_annotate.graph_data.chn_addrs.iter() {
        if graph_embedding.is_entropy_filtered_addr(chn_addr) {
            continue;
        }

        let sample = generate_semantic_samples_of_a_chunk(graph_embedding, *chn_addr);
        let label = get_node_label(graph_embedding, *chn_addr);

        samples.push(sample);
        labels.push(label);
    }
    (samples, labels)
}

fn generate_semantic_samples_of_a_chunk(
    graph_embedding : &GraphEmbedding, 
    chn_addr: u64
) -> Vec<usize> {

    let mut named_features = 
        get_chunk_basics_informations(graph_embedding, chn_addr);

    // add ancestors
    let mut ancestors = generate_samples_for_neighbor_nodes_of_the_chunk(
        graph_embedding, chn_addr, petgraph::Direction::Incoming
    );
    named_features.append(&mut ancestors);

    // add children
    let mut children = generate_samples_for_neighbor_nodes_of_the_chunk(
        graph_embedding, chn_addr, petgraph::Direction::Outgoing
    );
    named_features.append(&mut children);
    

    

    // add label
    named_features.push(get_node_label(graph_embedding, chn_addr));

    named_features
}