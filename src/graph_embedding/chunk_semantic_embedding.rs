use super::{GraphEmbedding, utils_embedding::{get_chunk_basics_informations, get_node_label}, neighboring::generate_samples_for_neighbor_nodes_of_the_chunk};

/// generate semantic embedding of all the chunks
/// in order :
///     - chunk header addresse (not really usefull for learning, but can bu usefull to further analyse the data)
///     - chunk size
///     - nb pointer
/// 
///     - ancestor (in order of depth, alternate CHN/PTR)
///     - children (same)
///     - label (if the chunk contains a key, or is the ssh or sessionState)
pub fn generate_chunk_semantic_embedding(graph_embedding : &GraphEmbedding) -> Vec<Vec<usize>> {
    let mut samples = Vec::new();
    // get chunk :
    for chn_addr in graph_embedding.graph_annotate.graph_data.chn_addrs.iter() {
        let sample = generate_semantic_samples_of_a_chunk(graph_embedding, *chn_addr);
        samples.push(sample);
    }
    samples
}

fn generate_semantic_samples_of_a_chunk(graph_embedding : &GraphEmbedding, chn_addr: u64) -> Vec<usize> {
    let mut feature: Vec<usize> = Vec::new();

    let mut info = get_chunk_basics_informations(graph_embedding, chn_addr);
    feature.append(&mut info);

    // add ancestors
    let mut ancestors = generate_samples_for_neighbor_nodes_of_the_chunk(
        graph_embedding, chn_addr, petgraph::Direction::Incoming
    );
    feature.append(&mut ancestors);

    // add children
    let mut children = generate_samples_for_neighbor_nodes_of_the_chunk(
        graph_embedding, chn_addr, petgraph::Direction::Outgoing
    );
    feature.append(&mut children);

    // add label
    feature.push(get_node_label(graph_embedding, chn_addr));

    feature
}