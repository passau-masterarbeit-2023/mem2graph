use std::path::PathBuf;
use crate::graph_embedding::GraphEmbedding;

/// Chunk semantic embedding, for a given file.
/// Save the embedding to a CSV file.
pub fn gen_and_save_chunk_semantic_embedding(
    output_file_path: PathBuf, 
    graph_embedding: &GraphEmbedding,
) -> usize {
    // generate the value embedding
    let (samples, labels) 
        = graph_embedding.generate_chunk_semantic_embedding();
    let samples_length = samples.len();
    
    // save the value embedding to CSV
    save_chunk_semantic_embeding(
        samples, 
        labels, 
        output_file_path, 
        *crate::params::EMBEDDING_DEPTH
    );

    return samples_length;
}

/// NOTE: saving empty files allow so that we don't have to recompute the samples and labels
/// for broken files (missing JSON key, etc.)
fn save_chunk_semantic_embeding(
    samples: Vec<Vec<usize>>, 
    labels: Vec<usize>, 
    csv_path: PathBuf, 
    embedding_depth: usize,
) {
    assert!(samples.len() == labels.len());
    let csv_error_message = format!("Cannot create csv file: {:?}, no such file.", csv_path);
    let mut csv_writer = csv::Writer::from_path(csv_path).unwrap_or_else(
        |_| panic!("{}", csv_error_message)
    );
    // header of CSV
    let mut header = Vec::new();
    header.push("f_chn_addr".to_string());
    header.push("f_chunk_byte_size".to_string());
    header.push("f_chunk_ptrs".to_string());
    for i in 0..embedding_depth {
        header.push(format!("f_chns_ancestor_{}", i + 1));
        header.push(format!("f_ptrs_ancestor_{}", i + 1));
    }

    for i in 0..embedding_depth {
        header.push(format!("f_chns_children_{}", i + 1));
        header.push(format!("f_ptrs_children_{}", i + 1));
    }
    header.push("label".to_string());


    csv_writer.write_record(header).unwrap();

    // save samples and labels to CSV
    for (sample, label) in samples.iter().zip(labels.iter()) {
        let mut row: Vec<String> = Vec::new();
        row.extend(sample.iter().map(|value| value.to_string()));
        row.push(label.to_string());

        csv_writer.write_record(&row).unwrap();
    }

    csv_writer.flush().unwrap();
}