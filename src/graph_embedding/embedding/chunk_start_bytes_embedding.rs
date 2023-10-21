use std::collections::HashMap;

use crate::graph_embedding::GraphEmbedding;
use crate::graph_embedding::utils_embedding::{extract_chunk_data_as_bytes, get_node_label, get_chunk_basics_informations};
use crate::params::{BLOCK_BYTE_SIZE, CHUNK_NB_OF_START_BYTES_FOR_CHUNK_EMBEDDING};






/// generate an embedding of only the start bytes of the chunks (one value per byte)
pub fn generate_chunk_start_bytes_embedding(graph_embedding : &GraphEmbedding) -> (Vec<HashMap<String, usize>>, Vec<usize>) {
    let mut samples = Vec::new();
    let mut labels = Vec::new();

    for addr in graph_embedding.graph_annotate.graph_data.chn_addrs.iter() {
        if graph_embedding.is_filtered_addr(addr) {
            continue;
        }
        
        let sample = generate_chunk_start_bytes_sample(graph_embedding, *addr);
        let label = get_node_label(graph_embedding, *addr);

        samples.push(sample);
        labels.push(label);
    }
    
    (samples, labels)
}

pub fn generate_chunk_start_bytes_sample(graph_embedding : &GraphEmbedding, addr: u64) -> HashMap<String, usize> {
    let mut sample = get_chunk_basics_informations(graph_embedding, addr);

    let bytes = extract_chunk_data_as_bytes(graph_embedding, addr, BLOCK_BYTE_SIZE);

    for (index, &byte) in bytes.iter().take(*CHUNK_NB_OF_START_BYTES_FOR_CHUNK_EMBEDDING).enumerate() {
        sample.insert(format!("byte_{}", index), byte as usize);
    }
    // insert missing 0
    for index in bytes.len()..*CHUNK_NB_OF_START_BYTES_FOR_CHUNK_EMBEDDING {
        sample.insert(format!("byte_{}", index), 0);
    }

    sample
}