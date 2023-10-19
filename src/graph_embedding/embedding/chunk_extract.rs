use std::collections::HashMap;

use crate::graph_embedding::GraphEmbedding;
use crate::graph_embedding::utils_embedding::{get_node_label, extract_chunk_data_as_bytes};
use crate::params::BLOCK_BYTE_SIZE;
use crate::utils::bytes_to_hex_string;



/// Extract chunk data user as hexa string
pub fn generate_chunk_extract(
    graph_embedding : &GraphEmbedding,
) -> (Vec<HashMap<String, String>>, Vec<usize>) {
    let mut samples = Vec::new();
    let mut labels = Vec::new();
    for chn_addr in graph_embedding.graph_annotate.graph_data.chn_addrs.iter() {
        if graph_embedding.is_filtered_addr(chn_addr) {
            continue;
        }

        let bytes = extract_chunk_data_as_bytes(graph_embedding, *chn_addr, BLOCK_BYTE_SIZE);
        let hexa_string = bytes_to_hex_string(&bytes);
        let mut sample = HashMap::new();
        sample.insert("hexa_representation".to_string(), hexa_string);
        

        samples.push(sample);
        labels.push(get_node_label(graph_embedding, *chn_addr));
    }
    (samples, labels)
}