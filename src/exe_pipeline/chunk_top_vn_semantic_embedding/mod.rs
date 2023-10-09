use rayon::prelude::*;
use std::{time::Instant, path::PathBuf};

use crate::{graph_embedding::GraphEmbedding, exe_pipeline::{progress_bar, value_embedding::save_value_embeding}, params::argv::{SelectAnnotationLocation, EntropyFilter}, utils::truncate_path_to_last_n_dirs};


use super::{get_raw_file_or_files_from_path, get_file_batches};

/// Value node semantic embedding, for value nodes that are first 
/// blocks of the user data section of a chunk. 
/// 
/// NOTE: This makes sense since this embedding is intended to be used
/// for the purpose of ML classification for encryption key
/// detection. It has been observed that the first block of those
/// keys are always value nodes located at the beginning of a 
/// chunk user data section.
/// 
/// |> Prepare the data:
///     1. Takes a directory or a file
///     If directory then list all files in that directory 
///     and its subdirectories that are of type "-heap.raw",
///     with their corresponding ".json" files.
/// 
/// |> Batch processing:
///     2. Then, perform the value embedding including labelling
///     for each of those files.
///     NOTE: The optional entropy filter is applied direclty
///     in the embedding processing.
/// 
///     3. Save the resulting samples and labels in a csv file.
///     For this pipeline, each input RAW file has its own csv file.
pub fn run_chunk_top_vn_semantic_embedding(
    path: PathBuf, 
    output_folder: PathBuf, 
    annotation : SelectAnnotationLocation, 
    entropy_filter : EntropyFilter
) {
    // |> Prepare the data:
    // --> Step 1: Getting the files
    let heap_dump_raw_file_paths: Vec<PathBuf> = get_raw_file_or_files_from_path(path.clone());

    // start timer
    let start_time = Instant::now();

    // cut the path
    let dir_path_ = path.clone();
    let dir_path_end = truncate_path_to_last_n_dirs(&dir_path_, 5);
    let dir_path_end_str = dir_path_end.to_str().unwrap(); 

    // prepare file batch processing
    let nb_files = heap_dump_raw_file_paths.len();
    let batch_size = crate::params::NB_FILES_PER_FILE_BATCH.clone();
    let mut batch_index = 0;

    // test if there is at least one file
    if nb_files == 0 {
        panic!("The file doesn't exist or the directory doesn't contain any .raw file: {}", path.to_str().unwrap());
    }

    // |> Batch processing:
    let file_batches = get_file_batches(&heap_dump_raw_file_paths, batch_size);
    for batch in file_batches {
        // chunk time
        let chunk_start_time = Instant::now();

        // Create a thread pool with named threads
        let pool = rayon::ThreadPoolBuilder::new()
            .thread_name(|index| format!("worker-{}", index))
            .build()
            .unwrap();

        // generate samples and labels, file per file
        let _ : Vec<_> = pool.install(|| {
            batch.par_iter().enumerate().map(|(i, heap_dump_raw_file_path)| {
                let current_thread = std::thread::current();
                let thread_name = current_thread.name().unwrap_or("<unnamed>");
                let global_idx = i + batch_size*batch_index;

                // check if CSV file already exists, in that case skip
                let file_name = heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap();
                let csv_pipeline_prefix = "chunk_top_vn_semantic";
                let csv_file_name = format!(
                    "{}_{}_{}_samples.csv", 
                    csv_pipeline_prefix, 
                    dir_path_end_str.replace("/", "_"),
                    file_name,
                );
                let output_file_path = output_folder.clone().join(csv_file_name.clone());
                if output_file_path.exists() {
                    log::info!(" 🔵 [N°{}-{} / {} files] [id: {}] already saved (csv: {}).", 
                        batch_index*batch_size,
                        batch_index*batch_index + batch_size - 1,
                        nb_files, 
                        batch_index,
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
                    annotation,
                    false
                );
                match graph_embedding {
                    Ok(_) => {},
                    Err(err) => match err {
                        crate::utils::ErrorKind::MissingJsonKeyError(key) => {
                            log::warn!(" 🔴 [t: {}] [N°{} / {} files] [fid: {}]    Missing JSON key: {}", thread_name, global_idx, nb_files, heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap(), key);
                            return (); // skip
                        },
                        crate::utils::ErrorKind::JsonFileNotFound(json_file_path) => {
                            log::warn!(" 🟣 [t: {}] [N°{} / {} files] [fid: {}]    JSON file not found: {:?}", thread_name, global_idx, nb_files, heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap(), json_file_path);
                            return (); // skip
                        },
                        _ => {
                            panic!("Other unexpected graph embedding error: {}", err);
                        }
                    }
                }
                let graph_embedding = graph_embedding.unwrap();

                // generate the value embedding
                let (samples, labels) = graph_embedding.generate_chunk_top_vn_semantic_embedding();
                let samples_length = samples.len();
                
                // save the value embedding to CSV
                save_value_embeding(samples, labels, output_file_path, *crate::params::EMBEDDING_DEPTH);
                let file_name_id = heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap().replace("-heap.raw", "");
                log::info!(" 🟢 [t: {}] [N°{} / {} files] [fid: {}]    (Nb samples: {})", thread_name, global_idx, nb_files, file_name_id, samples_length);

            }).collect()
        });

        // log time
        let chunk_duration = chunk_start_time.elapsed();
        let total_duration = start_time.elapsed();
        let progress = progress_bar(batch_index * batch_size, nb_files, 20);
        log::info!(
            " ⏱️  [chunk: {:.2?} / total: {:.2?}] {}",
            chunk_duration,
            total_duration,
            progress
        );

        batch_index += 1;
    }

}