use std::path::PathBuf;

use exe_pipeline::statistic_dtn_embedding::run_statistics_dtn_embedding;
use exe_pipeline::{value_embedding::run_value_embedding, graph_generation::run_graph_generation, semantic_dtn_embedding::run_semantic_dtn_embedding, extract_dts_data::run_extract_dtn_data};
use params::argv::Pipeline;

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
    let mut input_path: Vec<std::path::PathBuf> = Vec::new();
    if params::ARGV.files.is_some() {
        let files = params::ARGV.files.as_ref().unwrap();
        for (_, file) in files.iter().enumerate() {
            input_path.push(PathBuf::from(file));
        }
    } else if params::ARGV.directories.is_some() {
        for path in params::ARGV.directories.as_ref().unwrap() {
            input_path.push(PathBuf::from(path));
        }
    } else {
        // default
        input_path.push(params::DEFAULT_DATA_DIR_PATH.clone());
    }

    // output folder
    let output_folder;
    if params::ARGV.output.is_some() {
        output_folder = PathBuf::from(params::ARGV.output.as_ref().unwrap());
    } else {
        output_folder = params::DEFAULT_SAVE_SAMPLES_AND_LABELS_DIR_PATH.clone();
    }

    // test all provided paths
    for path in input_path.clone() {
        if !path.exists() {
            panic!("🚩 The path doesn't exist: {}", path.to_str().unwrap());
        }
    }
    // annotation of the graph
    let annotation = !params::ARGV.no_annotation;
    let no_value_node = params::ARGV.no_value_node;

    // launch computations
    for path in input_path {
        match params::ARGV.pipeline {
            params::argv::Pipeline::ValueEmbedding => {
                no_arg_no_value_and_pointer_node(params::ARGV.pipeline);
                run_value_embedding(path, output_folder.clone(), annotation)
            },
            params::argv::Pipeline::Graph => {
                run_graph_generation(path, output_folder.clone(), annotation, no_value_node)
            },
            params::argv::Pipeline::SemanticEmbeddingDTN => {
                run_semantic_dtn_embedding(path, output_folder.clone(), annotation, no_value_node)
            },
            params::argv::Pipeline::DtsExtraction => {
                run_extract_dtn_data(path, output_folder.clone(), *params::EXTRACT_NO_POINTER, annotation, no_value_node)
            },
            params::argv::Pipeline::StatisticEmbeddingDTN => {
                no_arg_no_value_and_pointer_node(params::ARGV.pipeline);
                run_statistics_dtn_embedding(path, output_folder.clone(), annotation)
            },
        }
    }
}

/// This function is call if the flag '-v' is used on a pipeline that doesn't use it
fn no_arg_no_value_and_pointer_node(pipeline : Pipeline) {
    if params::ARGV.no_value_node {
        panic!("🚩 The flag '-v' is used with the wrong pipeline : {:?}", pipeline);
    }
}