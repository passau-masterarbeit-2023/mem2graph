use lazy_static::lazy_static;
use std::path::{PathBuf};
use dotenv::dotenv;
use std::sync::Once;
use chrono;
use std::str::FromStr;

use crate::utils::Endianness;
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
        dotenv().ok();
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
        PathBuf::from(repo_dir)
    };
    
    pub static ref TEST_HEAP_DUMP_FILE_PATH: PathBuf = {
        let test_heap_dump_raw_file_path = std::env::var("TEST_HEAP_DUMP_RAW_FILE_PATH")
            .expect("TEST_HEAP_DUMP_RAW_FILE_PATH environment variable must be set").to_string();
        PROJECT_BASE_DIR.join(&test_heap_dump_raw_file_path)
    };

    pub static ref TEST_HEAP_JSON_FILE_PATH: PathBuf = {
        crate::utils::heap_dump_path_to_json_path(&TEST_HEAP_DUMP_FILE_PATH)
    };

    pub static ref COMPRESS_POINTER_CHAINS: bool = {
        let compress_pointer_chains = std::env::var("COMPRESS_POINTER_CHAINS");
        match compress_pointer_chains {
            Ok(mode) => mode == "true",
            Err(_) => {
                println!("COMPRESS_POINTER_CHAINS environment variable not set. Defaulting to 'false'.");
                return false;
            },
        }
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

    pub static ref TEST_CSV_EMBEDDING_FILE_PATH: PathBuf = {
        let test_csv_embedding_file_path = std::env::var("TEST_CSV_EMBEDDING_FILE_PATH")
            .expect("TEST_CSV_EMBEDDING_FILE_PATH environment variable must be set").to_string();
        PROJECT_BASE_DIR.join(&test_csv_embedding_file_path)
    };

    pub static ref REMOVE_TRIVIAL_ZERO_SAMPLES: bool = {
        let remove_trivial_zero_samples = std::env::var("REMOVE_TRIVIAL_ZERO_SAMPLES");
        match remove_trivial_zero_samples {
            Ok(mode) => mode == "true",
            Err(_) => {
                println!("REMOVE_TRIVIAL_ZERO_SAMPLES environment variable not set. Defaulting to 'false'.");
                return false;
            },
        }
    };

    pub static ref DEFAULT_DATA_DIR_PATH: PathBuf = {
        let testing_data_dir_path = std::env::var("DEFAULT_DATA_DIR_PATH")
            .expect("DEFAULT_DATA_DIR_PATH environment variable must be set").to_string();
        PROJECT_BASE_DIR.join(&testing_data_dir_path)
    };

    pub static ref DEFAULT_SAVE_SAMPLES_AND_LABELS_DIR_PATH: PathBuf = {
        let samples_and_labels_data_dir_path = std::env::var("DEFAULT_SAVE_SAMPLES_AND_LABELS_DIR_PATH")
            .expect("DEFAULT_SAVE_SAMPLES_AND_LABELS_DIR_PATH environment variable must be set").to_string();
        PathBuf::from(samples_and_labels_data_dir_path)
    };

    pub static ref NB_FILES_PER_CHUNK: usize = {
        let nb_files_per_chunk = std::env::var("NB_FILES_PER_CHUNK");
        match nb_files_per_chunk {
            Ok(nb) => nb.parse::<usize>().unwrap(),
            Err(_) => {
                println!("NB_FILES_PER_CHUNK environment variable not set. Defaulting to '10'.");
                return 10;
            },
        }
    };

    pub static ref EXTRACT_NO_POINTER: bool = {
        let val = std::env::var("EXTRACT_NO_POINTER");
        match val {
            Ok(nb) => nb.parse::<bool>().unwrap(),
            Err(_) => {
                println!("EXTRACT_NO_POINTER environment variable not set. Defaulting to false.");
                return false;
            },
        }
    };

}
