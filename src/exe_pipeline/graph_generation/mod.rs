use rayon::prelude::*;
use std::{time::Instant, path::PathBuf, fs::File, io::Write};

use crate::{exe_pipeline::progress_bar, graph_annotate::GraphAnnotate};

use super::get_raw_file_or_files_from_path;
/// Takes a directory or a file
/// If directory then list all files in that directory and its subdirectories
/// that are of type "-heap.raw", and their corresponding ".json" files.
/// Then do the graph generation for all these file
pub fn run_graph_generation(path: PathBuf, output_folder: PathBuf, annotation : bool, no_value_node: bool) {
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

   // run the dot file generation for each file by chunks
   for chunk in heap_dump_raw_file_paths.chunks(chunk_size) {
       // chunk time
       let chunk_start_time = Instant::now();

       // Create a thread pool with named threads
       let pool = rayon::ThreadPoolBuilder::new()
           .thread_name(|index| format!("worker-{}", index))
           .build()
           .unwrap();

        // generate samples and labels
        let _: Vec<_> = pool.install(|| {
            chunk.par_iter().enumerate().map(|(i, heap_dump_raw_file_path)| {
                let current_thread = std::thread::current();
                let thread_name = current_thread.name().unwrap_or("<unnamed>");
                let global_idx = i + chunk_size*chunck_index;
                // check save
                let heap_dump_path_copy = heap_dump_raw_file_path.clone();
                let heap_dump_name = heap_dump_path_copy.file_name().unwrap().to_os_string().into_string().unwrap();
                let dot_file_name = format!("{}_{}_dot.gv", dir_path_end_str.replace("/", "_"), heap_dump_name.replace("/", "_"));
                //println!("{}", dot_file_name);
                let dot_path = output_folder.clone().join(dot_file_name.clone());
                if dot_path.exists() {
                    log::info!(" 🔵 [N°{}-{} / {} files] [id: {}] already saved (csv: {}).", 
                        chunck_index*chunk_size,
                        chunck_index*chunk_size + chunk_size - 1,
                        nb_files, 
                        chunck_index,
                        dot_file_name.as_str()
                    );
                    return ();
                }
                
               let graph_annotate = GraphAnnotate::new(
                   heap_dump_raw_file_path.clone(),
                   crate::params::BLOCK_BYTE_SIZE,
                   annotation,
                   no_value_node
               );

               

               match graph_annotate {
                   Ok(graph_annotate) => {
                       let file_name_id = heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap().replace("-heap.raw", "");
                       log::info!(" 🟢 [t: {}] [N°{} / {} files] [fid: {}] made", thread_name, global_idx, nb_files, file_name_id);

                        let mut dot_file = File::create(dot_path).unwrap();
                        dot_file.write_all(format!("{}", graph_annotate.graph_data).as_bytes()).unwrap(); // using the custom formatter

                   },
                   Err(err) => match err {
                       crate::utils::ErrorKind::MissingJsonKeyError(key) => {
                           log::warn!(" 🔴 [t: {}] [N°{} / {} files] [fid: {}]    Missing JSON key: {}", thread_name, global_idx, nb_files, heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap(), key);
                           return;
                       },
                       crate::utils::ErrorKind::JsonFileNotFound(json_file_path) => {
                           log::warn!(" 🟣 [t: {}] [N°{} / {} files] [fid: {}]    JSON file not found: {:?}", thread_name, global_idx, nb_files, heap_dump_raw_file_path.file_name().unwrap().to_str().unwrap(), json_file_path);
                           return;
                       },
                       _ => {
                           panic!("Other unexpected graph embedding error: {}", err);
                       }
                   }
               }
               
           }).collect()
       });

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