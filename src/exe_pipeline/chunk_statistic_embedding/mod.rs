use std::path::PathBuf;
use crate::graph_embedding::GraphEmbedding;
use crate::params::{N_GRAM, BLOCK_BYTE_SIZE};
use super::save_embedding_with_f64;

/// Chunk statistic embedding, for a given file.
/// Save the embedding to a CSV file.
pub fn gen_and_save_chunk_statistic_embedding(
    output_file_path: PathBuf, 
    graph_embedding: &GraphEmbedding,
) -> usize {
    // generate the value embedding
    let (samples, labels) 
        = graph_embedding.generate_chunk_statistic_embedding(
            &(*N_GRAM), 
            BLOCK_BYTE_SIZE
        );
    let samples_length = samples.len();
    
    // save the value embedding to CSV
    save_embedding_with_f64(
        samples, 
        labels, 
        output_file_path
    );

    return samples_length;
}