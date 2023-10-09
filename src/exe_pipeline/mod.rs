use std::{path::PathBuf, slice::Chunks};
use walkdir::WalkDir;


pub mod value_embedding;
pub mod graph_generation;
pub mod chunk_semantic_embedding;
pub mod chunk_statistic_embedding;
pub mod chunk_top_vn_semantic_embedding;

/// Takes a path as input.
/// This path can be a file or a directory.
/// If it is a file, return a vector containing only this file.
/// If it is a directory, return a vector containing all files in this directory.
fn get_raw_file_or_files_from_path(path: PathBuf) -> Vec<PathBuf> {
    let mut raw_file_paths: Vec<PathBuf> = Vec::new();

    if path.is_file() {
        raw_file_paths.push(path);
    } else if path.is_dir() {
        for entry in WalkDir::new(path) {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() {
                if path.extension().map_or(false, |ext| ext == "raw") {
                    raw_file_paths.push(path.to_path_buf());
                }
            }
        }
    }

    return raw_file_paths;
}


fn progress_bar(current: usize, total: usize, length: usize) -> String {
    let ratio = current as f64 / total as f64;
    let filled_len = (ratio * length as f64).round() as usize;
    let empty_len = length - filled_len;

    format!("|{}{}| {:.2?}%", "█".repeat(filled_len), " ".repeat(empty_len), (ratio * 100.0))
}

/// Wrapper. Since we are using the term "chunk" everywhere,
/// there is a real need to hide the built-in "chunk" batching
/// function. Keep programmer's sanity.
pub fn get_file_batches<'a>(
    files: &'a Vec<PathBuf>,
    batch_size: usize
) -> std::slice::Chunks<'a, PathBuf> {
    files.chunks(batch_size)
}