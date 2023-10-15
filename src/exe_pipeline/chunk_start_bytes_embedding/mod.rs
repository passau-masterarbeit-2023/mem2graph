use std::path::PathBuf;
use crate::graph_embedding::GraphEmbedding;

use super::save_embedding;

/// Chunk semantic embedding, for a given file.
/// Save the embedding to a CSV file.
pub fn gen_and_save_chunk_start_bytes_embedding(
    output_file_path: PathBuf, 
    graph_embedding: &GraphEmbedding,
) -> usize {
    // generate the value embedding
    let (samples, labels) 
        = graph_embedding.generate_chunk_start_bytes_embedding();
    let samples_length = samples.len();
    
    // save the value embedding to CSV
    save_embedding(
        samples, 
        labels, 
        output_file_path
    );

    return samples_length;
}