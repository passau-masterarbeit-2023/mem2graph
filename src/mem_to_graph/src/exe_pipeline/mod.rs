use std::path::PathBuf;
use walkdir::WalkDir;
use rayon::prelude::*;
use std::time::Instant;

use crate::graph_embedding::GraphEmbedding;

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

/// Takes a directory or a file
/// If directory then list all files in that directory and its subdirectories
/// that are of type "-heap.raw", and their corresponding ".json" files.
/// Then do the sample and label generation for each of those files.
/// return: all samples and labels for all thoses files.
pub fn run(path: PathBuf) {
    // start timer
    let start_time = Instant::now();

    // cut the path to just after "phdtrack_data"
    let dir_path_ = path.clone();
    let dir_path_split = dir_path_.to_str().unwrap().split("phdtrack_data/").collect::<Vec<&str>>();
    
    if dir_path_split.len() != 2 {
        panic!("The path must contains \"phdtrack_data/\" and the name of the directory or file.");
    }
    let dir_path_end_str = dir_path_split[1];

    let heap_dump_raw_file_paths: Vec<PathBuf> = get_raw_file_or_files_from_path(path.clone());

    let nb_files = heap_dump_raw_file_paths.len();
    let chunk_size = crate::params::NB_FILES_PER_CHUNK.clone();
    let mut chunck_index = 0;

    // test if there is at least one file
    if nb_files == 0 {
        panic!("The file doesn't exist or the directory doesn't contain any .raw file: {}", path.to_str().unwrap());
    }

    // run the sample and label generation for each file by chunks
    for chunk in heap_dump_raw_file_paths.chunks(chunk_size) {
        // chunk time
        let chunk_start_time = Instant::now();

        // check save
        let csv_file_name = format!("{}_chunck_idx-{}_samples.csv", dir_path_end_str.replace("/", "_"), chunck_index);
        let csv_path = crate::params::SAMPLES_AND_LABELS_DATA_DIR_PATH.clone().join(csv_file_name.clone());
        if csv_path.exists() {
            log::info!(" 🔵 [N°{}-{} / {} files] [id: {}] already saved (csv: {}).", 
                chunck_index*chunk_size,
                chunck_index*chunk_size + chunk_size - 1,
                nb_files, 
                chunck_index,
                csv_file_name.as_str()
            );
            chunck_index += 1;
            continue;
        }

        // generate samples and labels
        let results: Vec<_> = chunk.par_iter().enumerate().map(|(i, heap_dump_raw_file_path)| {
            let global_idx = i + chunk_size*chunck_index;

            let graph_embedding = GraphEmbedding::new(
                heap_dump_raw_file_path.clone(),
                crate::params::BLOCK_BYTE_SIZE,
                *crate::params::EMBEDDING_DEPTH
            );

            match graph_embedding {
                Ok(graph_embedding) => {
                    // generate samples and labels
                    let (samples_, labels_) = graph_embedding.generate_samples_and_labels();

                    let file_name_id = heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap().replace("-heap.raw", "");
                    log::info!(" 🟢 [N°{} / {} files] [id: {}]    (Nb samples: {})", global_idx, nb_files, file_name_id, samples_.len());

                    (samples_, labels_)
                },
                Err(err) => match err {
                    crate::utils::ErrorKind::MissingJsonKeyError(key) => {
                        log::warn!(" 🔴 [N°{} / {} files] [id: {}]    Missing JSON key: {}", global_idx, nb_files, heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap(), key);
                        (Vec::new(), Vec::new())
                    },
                    _ => {
                        panic!("Other unexpected graph embedding error: {}", err);
                    }
                }
            }

            
        }).collect();

        // save to csv
        let mut samples = Vec::new();
        let mut labels = Vec::new();
        for (samples_, labels_) in results {
            samples.extend(samples_);
            labels.extend(labels_);
            
        }
        save(samples, labels, csv_path);

        // log time
        let chunk_duration = chunk_start_time.elapsed();
        let total_duration = start_time.elapsed();
        let progress = progress_bar(chunck_index * chunk_size, nb_files, 20);
        log::info!(
            " ⏱️  [chunk: {:.2?} / total: {:.2?}] {}",
            chunk_duration,
            total_duration,
            progress
        );

        chunck_index += 1;
    }

}

/// NOTE: saving empty files allow so that we don't have to recompute the samples and labels
/// for broken files (missing JSON key, etc.)
pub fn save(samples: Vec<Vec<usize>>, labels: Vec<usize>, csv_path: PathBuf) {
    let mut csv_writer = csv::Writer::from_path(csv_path).unwrap();

    // header of CSV
    let mut header = Vec::new();
    header.push("f_dtn_byte_size".to_string());
    header.push("f_position_in_dtn".to_string());
    header.push("f_dtn_ptrs".to_string());
    header.push("f_dtn_vns".to_string());
    // start at 1 since 0 is a ValueNode (so always [0, 0])
    for i in 1..*crate::params::EMBEDDING_DEPTH {
        header.push(format!("f_dtns_ancestor_{}", i));
        header.push(format!("f_ptrs_ancestor_{}", i));
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

fn progress_bar(current: usize, total: usize, length: usize) -> String {
    let ratio = current as f64 / total as f64;
    let filled_len = (ratio * length as f64).round() as usize;
    let empty_len = length - filled_len;

    format!("|{}{}| {:.2?}%", "█".repeat(filled_len), " ".repeat(empty_len), (ratio * 100.0))
}