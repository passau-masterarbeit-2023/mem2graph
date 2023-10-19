use rayon::prelude::*;
use std::{time::Instant, path::PathBuf};

use crate::{graph_embedding::GraphEmbedding, params::{argv::{SelectAnnotationLocation, EntropyFilter, ChunkByteSizeFilter}, ARGV}, utils::truncate_path_to_last_n_dirs};
use super::get_raw_file_or_files_from_path;

/// Wrapper for the embedding pipeline, with the CSV saving.
pub fn embedding_pipeline_to_csv(
    path: PathBuf, 
    output_folder: PathBuf, 
    annotation : SelectAnnotationLocation, 
    entropy_filter : EntropyFilter,
    chunk_byte_size_filter : ChunkByteSizeFilter,
    no_value_node : bool,
    gen_and_save_embedding: fn(PathBuf, &GraphEmbedding) -> usize,
) {
    embedding_pipeline(
        path, 
        output_folder, 
        annotation, 
        entropy_filter, 
        chunk_byte_size_filter,
        no_value_node,
        gen_and_save_embedding,
        ".csv"
    );
}

/// Generic pipeline function for embedding generation.
/// 
/// > Prepare the data:
///     Takes a directory or a file
///     If directory then list all files in that directory 
///     and its subdirectories that are of type "-heap.raw",
///     with their corresponding ".json" files.
/// 
/// > File per file processing:
///     Then, perform the provided embedding including labelling
///     for each of those files.
///     NOTE: The optional entropy filter is applied direclty
///     in the embedding processing.
///     NOTE: The saving of the embedding is done by the provided
///     function.
/// 
/// :gen_and_save_embedding: This function is responsible for 
/// doing the embedding on a given file, and saving it.
/// + It takes as input the output folder and the graph embedding.
/// + It returns the number of samples generated.
pub fn embedding_pipeline(
    path: PathBuf, 
    output_folder: PathBuf, 
    annotation : SelectAnnotationLocation, 
    entropy_filter : EntropyFilter,
    chunk_byte_size_filter : ChunkByteSizeFilter,
    no_value_node : bool,
    gen_and_save_embedding: fn(PathBuf, &GraphEmbedding) -> usize,
    save_file_extension: &str,
) {
    // start timer
    let start_time = Instant::now();

    // |> Prepare the data:
    // --> Step 1: Getting the files
    let heap_dump_raw_file_paths: Vec<PathBuf> = get_raw_file_or_files_from_path(path.clone());
    let nb_files = heap_dump_raw_file_paths.len();
    if nb_files == 0 {
        panic!(
            "The file doesn't exist or the directory doesn't contain any .raw file: {}", 
            path.to_str().unwrap()
        );
    } 

    let csv_pipeline_prefix = format!("{:?}", ARGV.pipeline);

    // |> File per file processing:
    // Create a thread pool with named threads
    let pool = rayon::ThreadPoolBuilder::new()
        .thread_name(|index| format!("worker-{}", index))
        .build()
        .unwrap();

    // generate samples and labels, file per file (parallelized)
    let _ : Vec<_> = pool.install(|| {
        heap_dump_raw_file_paths
            .par_iter()
            .enumerate()
            .map(|(i, heap_dump_raw_file_path)| 
        {
            let current_thread = std::thread::current();
            let thread_name = current_thread.name().unwrap_or("<unnamed>");

            // cut the path
            let dir_path_ = heap_dump_raw_file_path.clone();
            let dir_path_end = truncate_path_to_last_n_dirs(&dir_path_, 5);
            let dir_path_end_str = dir_path_end.to_str().unwrap();

            // check if CSV file already exists, in that case skip
            let file_name = heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap();
            let csv_file_name = format!(
                "{}_{}_{}_{}", 
                csv_pipeline_prefix, 
                dir_path_end_str.replace("/", "_"),
                file_name,
                save_file_extension
            );
            let output_file_path = output_folder.clone().join(csv_file_name.clone());
            if output_file_path.exists() {
                log::info!(" 🔵 [N°{} / {} files] already saved (csv: {}).", 
                    i,
                    nb_files,
                    output_file_path.to_str().unwrap()
                );
                return (); // skip
            }

            // make and check the memory graph
            let graph_embedding = GraphEmbedding::new(
                heap_dump_raw_file_path.clone(),
                crate::params::BLOCK_BYTE_SIZE,
                *crate::params::EMBEDDING_DEPTH,
                entropy_filter,
                chunk_byte_size_filter,
                annotation,
                no_value_node,
            );
            match graph_embedding {
                Ok(_) => {},
                Err(err) => match err {
                    crate::utils::ErrorKind::MissingJsonKeyError(key) => {
                        log::warn!(
                            " 🔴 [t: {}] [N°{} / {} files] [fid: {}]    Missing JSON key: {}", 
                            thread_name, 
                            i, 
                            nb_files, 
                            heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap(),
                            key
                        );
                        return (); // skip
                    },
                    crate::utils::ErrorKind::JsonFileNotFound(json_file_path) => {
                        log::warn!(" 🟣 [t: {}] [N°{} / {} files] [fid: {}]    JSON file not found: {:?}", 
                            thread_name, 
                            i, 
                            nb_files, 
                            heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap(), 
                            json_file_path
                        );
                        return (); // skip
                    },
                    _ => {
                        panic!("Other unexpected graph embedding error: {}", err);
                    }
                }
            }
            let graph_embedding = graph_embedding.unwrap();

            // generate the value embedding and save it
            let nb_of_samples = gen_and_save_embedding(output_file_path, &graph_embedding);

            let file_name_id = heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap().replace("-heap.raw", "");
            log::info!(
                " 🟢 [t: {}] [N°{} / {} files] [fid: {}]    (Nb samples: {})", 
                thread_name, 
                i, 
                nb_files, 
                file_name_id, 
                nb_of_samples
            );

        }).collect()
    });

    // log time
    let total_duration = start_time.elapsed();
    log::info!(
        " ⏱️  total pipeline time: {:.2?}]",
        total_duration
    );
}