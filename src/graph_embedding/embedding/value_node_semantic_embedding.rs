use std::collections::HashMap;

use crate::{graph_structs::Node, graph_embedding::{GraphEmbedding, utils_embedding::{get_node_label, get_chunk_basics_informations}, neighboring::get_neighbors}};


/// generate semantic embedding of the nodes
///     - parent chn address
///     - position in the chunk
///     - nb pointer
///     - nb value
/// 
///     - ancestor (in order of depth, alternate CHN/PTR)
/// Labels [0.0, 1.0, ..., 0.0],
pub fn generate_value_node_semantic_embedding(graph_embedding : &GraphEmbedding) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut samples = Vec::new();
    let mut labels = Vec::new();

    for addr in graph_embedding.graph_annotate.graph_data.value_node_addrs.iter() {
        if graph_embedding.is_entropy_filtered_addr(addr) {
            continue;
        }

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
fn add_features_from_parent_chunk(
    graph_embedding : &GraphEmbedding, 
    addr: u64,
) -> HashMap<String, usize> {
    let node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
    let parent_chn_node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(
        &node.get_parent_chn_addr().unwrap_or_else(
            || panic!(
                "The chn addr should be set, for node at address {:#x}, for file {}", 
                addr, 
                graph_embedding.graph_annotate.graph_data.heap_dump_data.as_ref().unwrap().heap_dump_raw_file_path.to_str().unwrap()
            )
        )
    ).unwrap();

    // add features from parent chn node
    get_chunk_basics_informations(
        graph_embedding, 
        parent_chn_node.get_address(),
    )
}

/// generate the value embedding of a value node
pub fn generate_value_sample(
    graph_embedding : &GraphEmbedding, 
    addr: u64
) -> HashMap<String, usize> {
    let mut named_features_from_parent_chunk = 
        add_features_from_parent_chunk(graph_embedding, addr);
    let mut named_features_from_ancestors = get_neighbors(graph_embedding, vec![addr].into_iter().collect(), petgraph::Direction::Incoming);
    
    // combine the two features hashmaps
    named_features_from_parent_chunk.extend(
        named_features_from_ancestors
    );
    named_features_from_parent_chunk
}