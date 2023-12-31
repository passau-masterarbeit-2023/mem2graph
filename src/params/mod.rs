use lazy_static::lazy_static;
use std::collections::HashSet;
use std::path::PathBuf;
use dotenv::dotenv;
use std::sync::Once;
use chrono;
use std::str::FromStr;

use crate::utils::{Endianness, string_to_usize_vec};
pub mod argv;

pub const BLOCK_BYTE_SIZE: usize = 8; // 64-bit, ex: C0 03 7B 09 2A 56 00 00

/// WARN: SHOULD BE USED ONLY FOR NODE CONSTRUCTION (see utils::convert_block_to_pointer_if_possible)
pub const PTR_ENDIANNESS: Endianness = Endianness::Little;
pub const MALLOC_HEADER_ENDIANNESS: Endianness = Endianness::Little;

/// Initialize logger. 
/// WARN: Must be called before any logging is done.
fn init_logger() {
    let log_directory = "./log";
    std::fs::create_dir_all(log_directory).expect("Failed to create log directory");

    let file_out = fern::log_file(&format!("{}/output.log", log_directory)).expect("Failed to open log file");

    // Parse the log level from LOGGER_MODE
    let log_level = match log::LevelFilter::from_str(LOGGER_MODE.as_str()) {
        Ok(level) => level,
        Err(_) => {
            println!("Invalid LOGGER_MODE value. Defaulting to 'info'.");
            log::LevelFilter::Info
        },
    };

    let logger_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}][{} {}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                chrono::offset::Utc::now().format("%Z"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(file_out)
        .chain(fern::Output::call(|record| {
            println!("{}", record.args());
        }))
        .apply();

    if let Err(e) = logger_config {
        panic!("Failed to initialize logger: {}", e);
    }

    log::info!(" 🚀 starting mem to graph converter");
}

static INIT: Once = Once::new();

/// Initialize things that need to be initialized only once.
pub fn init() {
    INIT.call_once(|| {
        // initialization code here
        print!("Loading .env file... ");
        dotenv().expect("Failed to load .env file");

        print!("Initializing logger... ");
        init_logger();
    });
}

// Get the path to files for the program, using the environment variables.
lazy_static! {
    pub static ref ARGV: argv::Argv = argv::get_program_args();

    static ref LOGGER_MODE: String = {
        let logger_mode = std::env::var("LOGGER_MODE");
        match logger_mode {
            Ok(mode) => mode,
            Err(_) => {
                println!("LOGGER_MODE environment variable not set. Defaulting to 'info'.");
                return "info".to_string();
            },
        }
    };

    static ref PROJECT_BASE_DIR: PathBuf = {
        let repo_dir = std::env::var("PROJECT_BASE_DIR")
            .expect("PROJECT_BASE_DIR environment variable must be set");
        let path: PathBuf = PathBuf::from(repo_dir);
        check_path(&path);
        path
    };
    
    pub static ref TEST_HEAP_DUMP_FILE_PATH: PathBuf = {
        let test_heap_dump_raw_file_path = std::env::var("TEST_HEAP_DUMP_RAW_FILE_PATH")
            .expect("TEST_HEAP_DUMP_RAW_FILE_PATH environment variable must be set").to_string();
        let path: PathBuf = PROJECT_BASE_DIR.join(&test_heap_dump_raw_file_path);
        check_path(&path);
        path
    };

    pub static ref TEST_HEAP_JSON_FILE_PATH: PathBuf = {
        let path: PathBuf = crate::utils::heap_dump_path_to_json_path(&TEST_HEAP_DUMP_FILE_PATH);
        check_path(&path);
        path
    };

    pub static ref EMBEDDING_DEPTH: usize = {
        let base_embedding_depth = std::env::var("EMBEDDING_DEPTH");
        match base_embedding_depth {
            Ok(depth) => depth.parse::<usize>().unwrap(),
            Err(_) => {
                println!("EMBEDDING_DEPTH environment variable not set. Defaulting to '1'.");
                return 1;
            },
        }
    };

    /// WARN : This vector must be sorted in ascending order.
    pub static ref N_GRAM: Vec<usize> = {
        get_n_gram_from_env()
    };

    pub static ref CHUNK_BYTES_SIZE_TO_KEEP_FILTER : HashSet<usize> = {
        let chunk_bytes_size_to_keep_filter = std::env::var("CHUNK_BYTES_SIZE_TO_KEEP_FILTER");
        match chunk_bytes_size_to_keep_filter {
            Ok(filter) => {
                let filter: HashSet<usize> = string_to_usize_vec(filter.as_str()).into_iter().collect();
                filter
            },
            Err(_) => {
                println!("CHUNK_BYTES_SIZE_TO_KEEP_FILTER environment variable not set. Defaulting to '32'.");
                return vec![32].into_iter().collect();
            },
        }
    };

    pub static ref TEST_CSV_EMBEDDING_FILE_PATH: PathBuf = {
        let test_csv_embedding_file_path = std::env::var("TEST_CSV_EMBEDDING_FILE_PATH")
            .expect("TEST_CSV_EMBEDDING_FILE_PATH environment variable must be set").to_string();
        let path: PathBuf = PROJECT_BASE_DIR.join(&test_csv_embedding_file_path);
        // WARN: do NOT check_path(&path); This is because this file is created by the program.
        path
    };

    pub static ref DEFAULT_DATA_DIR_PATH: PathBuf = {
        let testing_data_dir_path = std::env::var("DEFAULT_DATA_DIR_PATH")
            .expect("DEFAULT_DATA_DIR_PATH environment variable must be set").to_string();
        let path: PathBuf = PROJECT_BASE_DIR.join(&testing_data_dir_path);
        check_path(&path);
        path
    };

    pub static ref DEFAULT_SAVE_SAMPLES_AND_LABELS_DIR_PATH: PathBuf = {
        let samples_and_labels_data_dir_path = std::env::var("DEFAULT_SAVE_SAMPLES_AND_LABELS_DIR_PATH")
            .expect("DEFAULT_SAVE_SAMPLES_AND_LABELS_DIR_PATH environment variable must be set").to_string();
        let path: PathBuf = PathBuf::from(samples_and_labels_data_dir_path);
        check_path(&path);
        path
    };

    pub static ref CHUNK_NB_OF_START_BYTES_FOR_CHUNK_ENTROPY: usize = {
        let val = std::env::var("CHUNK_NB_OF_START_BYTES_FOR_CHUNK_ENTROPY");
        match val {
            Ok(nb) => nb.parse::<usize>().unwrap(),
            Err(_) => {
                println!("CHUNK_NB_OF_START_BYTES_FOR_CHUNK_ENTROPY environment variable not set. Defaulting to 10.");
                return 10;
            },
        }
    };

    pub static ref CHUNK_NB_OF_START_BYTES_FOR_CHUNK_EMBEDDING: usize = {
        let val = std::env::var("CHUNK_NB_OF_START_BYTES_FOR_CHUNK_EMBEDDING");
        match val {
            Ok(nb) => nb.parse::<usize>().unwrap(),
            Err(_) => {
                println!("CHUNK_NB_OF_START_BYTES_FOR_CHUNK_EMBEDDING environment variable not set. Defaulting to 10.");
                return 10;
            },
        }
    };

    pub static ref MIN_NB_OF_CHUNKS_TO_KEEP: usize = {
        let val = std::env::var("MIN_NB_OF_CHUNKS_TO_KEEP");
        match val {
            Ok(nb) => nb.parse::<usize>().unwrap(),
            Err(_) => {
                println!("MIN_NB_OF_CHUNKS_TO_KEEP environment variable not set. Defaulting to 10.");
                return 0;
            },
        }
    };

}

/// Check if a path exists. If not, panic.
fn check_path(path: &PathBuf) {
    if !path.exists() {
        panic!("Path {:?} does not exist", path);
    }
}

/// get the N_GRAM environment variable and return it as a vector of usize (sorted in ascending order)
pub fn get_n_gram_from_env() -> Vec<usize>{
    let base_n_gram = std::env::var("N_GRAM");
    match base_n_gram {
        Ok(n_gram) => {
            let mut n_gram: Vec<usize> = string_to_usize_vec(n_gram.as_str());
            n_gram.sort();
            n_gram
        },
        Err(_) => {
            println!("N_GRAM environment variable not set. Defaulting to '1'.");
            return vec![1];
        },
    }
}