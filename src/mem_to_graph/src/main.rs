
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
    crate::exe_pipeline::run(crate::params::TESTING_DATA_DIR_PATH.clone());


}