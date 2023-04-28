use std::path::PathBuf;

// link modules
mod params;
mod tests;
mod graph_data;
mod graph_structs;
mod utils;
mod graph_annotate;
mod graph_embedding;
mod exe_pipeline;

fn main() {
    crate::params::init();

    // call pipeline
    let input_path: std::path::PathBuf;
    if params::ARGV.file.is_some() {
        input_path = PathBuf::from(params::ARGV.file.as_ref().unwrap());
    } else if params::ARGV.directory.is_some() {
        input_path = PathBuf::from(params::ARGV.directory.as_ref().unwrap());
    } else {
        // default
        input_path = params::TESTING_DATA_DIR_PATH.clone();
    }
    crate::exe_pipeline::run(input_path);
}