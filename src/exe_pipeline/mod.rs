use std::{path::PathBuf, collections::HashMap};
use csv::Writer;
use walkdir::WalkDir;

pub mod pipeline;
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

/// Save the samples and labels to a CSV file.
pub fn save_embedding(samples: Vec<HashMap<String, usize>>, labels: Vec<usize>, csv_path: PathBuf) {
    assert!(!samples.is_empty(), "Samples cannot be empty for CSV header extraction.");

    let csv_error_message = format!("Cannot create csv file: {:?}, no such file.", csv_path);
    let mut csv_writer = Writer::from_path(&csv_path).unwrap_or_else(
        |_| panic!("{}", csv_error_message)
    );

    // Assuming all HashMaps have the same set of keys
    let headers: Vec<String> = samples[0].keys().cloned().collect();
    
    let mut all_headers = headers.clone();
    all_headers.push("label".to_string());

    csv_writer.write_record(&all_headers).unwrap();

    // save samples and labels to CSV
    for (sample, &label) in samples.iter().zip(&labels) {
        // Check if all headers are present in the current sample
        if headers.iter().any(|h| !sample.contains_key(h)) {
            panic!("Headers mismatch between samples!");
        }

        let mut row: Vec<String> = headers.iter()
            .map(|header| sample.get(header).unwrap().to_string()) // unwrap is safe here since we've checked keys
            .collect();

        row.push(label.to_string());

        csv_writer.write_record(&row).unwrap();
    }

    csv_writer.flush().unwrap();
}


/// Save the samples and labels to a CSV file.
pub fn save_embedding_with_f64(
    samples: Vec<(HashMap<String, usize>, HashMap<String, f64>)>, 
    labels: Vec<usize>, 
    csv_path: PathBuf
) {
    assert!(!samples.is_empty(), "Samples cannot be empty for CSV header extraction.");

    let csv_error_message = format!("Cannot create csv file: {:?}, no such file.", csv_path);
    let mut csv_writer = Writer::from_path(&csv_path).unwrap_or_else(
        |_| panic!("{}", csv_error_message)
    );

    // Assuming all HashMaps have the same set of keys
    let usize_headers: Vec<String> = samples[0].0.keys().cloned().collect();
    let f64_headers: Vec<String> = samples[0].1.keys().cloned().collect();
    
    let mut all_headers = usize_headers.clone();
    all_headers.extend(f64_headers.iter().cloned());
    all_headers.push("label".to_string());

    csv_writer.write_record(&all_headers).unwrap();

    // Save samples and labels to CSV
    for ((usize_sample, f64_sample), &label) in samples.iter().zip(&labels) {
        // Check if all headers are present in the current sample
        if usize_headers.iter().any(|h| !usize_sample.contains_key(h)) || f64_headers.iter().any(|h| !f64_sample.contains_key(h)) {
            panic!("Headers mismatch between samples!");
        }

        let mut row: Vec<String> = usize_headers.iter()
            .map(|header| usize_sample.get(header).unwrap().to_string()) // unwrap is safe here since we've checked keys
            .chain(f64_headers.iter().map(|header| f64_sample.get(header).unwrap().to_string())) // similarly safe unwrap
            .collect();

        row.push(label.to_string());

        csv_writer.write_record(&row).unwrap();
    }

    csv_writer.flush().unwrap();
}
