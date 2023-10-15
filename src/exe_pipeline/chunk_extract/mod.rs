use std::path::PathBuf;
use crate::graph_embedding::GraphEmbedding;

use super::save_embedding_with_string;

/// Chunk semantic embedding, for a given file.
/// Save the embedding to a CSV file.
pub fn gen_and_save_chunk_extract(
    output_file_path: PathBuf, 
    graph_embedding: &GraphEmbedding,
) -> usize {
    // generate the value embedding
    let (samples, labels) 
        = graph_embedding.generate_chunk_extract();
    let samples_length = samples.len();
    
    // save the value embedding to CSV
    save_embedding_with_string(
        samples, 
        labels, 
        output_file_path
    );

    return samples_length;
}