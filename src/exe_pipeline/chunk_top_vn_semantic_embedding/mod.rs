use std::path::PathBuf;

use crate::graph_embedding::GraphEmbedding;

use super::value_embedding::save_value_embeding;

/// Value node semantic embedding, for value nodes that are first 
/// blocks of the user data section of a chunk. 
/// 
/// NOTE: This makes sense since this embedding is intended to be used
/// for the purpose of ML classification for encryption key
/// detection. It has been observed that the first block of those
/// keys are always value nodes located at the beginning of a 
/// chunk user data section.
/// 
/// This function generate the embedding for a given file, and save it.
pub fn gen_and_save_chunk_top_vn_semantic_embedding(
    output_file_path: PathBuf, 
    graph_embedding: &GraphEmbedding
) -> usize {
    // generate the value embedding
    let (samples, labels) = 
        graph_embedding.generate_chunk_top_vn_semantic_embedding();
    let samples_length = samples.len();
    
    // save the value embedding to CSV
    save_value_embeding(
        samples, 
        labels, 
        output_file_path, 
        *crate::params::EMBEDDING_DEPTH
    );

    return samples_length;
}