use std::path::PathBuf;
use walkdir::WalkDir;

use crate::graph_embedding::GraphEmbedding;

/// Takes a directory, then list all files in that directory and its subdirectories
/// that are of type "-heap.raw", and their corresponding ".json" files.
/// Then do the sample and label generation for each of those files.
/// return: all samples and labels for all thoses files.
pub fn run(dir_path: PathBuf) -> (Vec<Vec<usize>>, Vec<usize>) {
    let mut samples: Vec<Vec<usize>> = Vec::new();
    let mut labels: Vec<usize> = Vec::new();

    let mut heap_dump_raw_file_paths: Vec<PathBuf> = Vec::new();

    // list all files in the directory and its subdirectories
    for entry in WalkDir::new(dir_path) {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if path.extension().map_or(false, |ext| ext == "raw") {
                heap_dump_raw_file_paths.push(path.to_path_buf());
            }
        }
    }

    // for each file, do the sample and label generation
    let nb_files = heap_dump_raw_file_paths.len();
    for i in 0..nb_files {
        let heap_dump_raw_file_path = heap_dump_raw_file_paths[i].clone();

        log::info!("[N°{} / {}] Generating samples and labels for {:?}", i, nb_files, heap_dump_raw_file_path);

        let graph_embedding = GraphEmbedding::new(
            heap_dump_raw_file_path,
            crate::params::BLOCK_BYTE_SIZE,
            *crate::params::EMBEDDING_DEPTH
        );

        let (samples_, labels_) = graph_embedding.generate_samples_and_labels();
        samples.extend(samples_);
        labels.extend(labels_);

        log::info!("[N°{} / {}]    Done.      (Nb samples: {})", i, nb_files, samples.len());
    }

    (samples, labels)
}