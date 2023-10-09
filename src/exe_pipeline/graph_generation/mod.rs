use std::{path::PathBuf, fs::File, io::Write};
use crate::graph_embedding::GraphEmbedding;

/// Generate a graph to dot file for the given file.
pub fn gen_and_save_memory_graph(
    output_file_path: PathBuf, 
    graph_embedding: &GraphEmbedding,
) -> usize {
    // annotate the graph
    let mut dot_file = File::create(output_file_path).unwrap();
    dot_file.write_all(
        format!("{}", 
        graph_embedding.graph_annotate.graph_data
    ).as_bytes()).unwrap(); // using the custom formatter
    return 0; // no samples, only the graph
}
