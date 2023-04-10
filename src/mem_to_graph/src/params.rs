use lazy_static::lazy_static;
use std::path::{PathBuf};
use dotenv::dotenv;
use std::sync::Once;

use crate::utils::Endianness;

pub const DEBUG: bool = false;

pub const XXD_LINE_BLOCK_BYTE_SIZE: u64 = 16;
pub const BLOCK_BYTE_SIZE: usize = 8; // 64-bit, ex: C0 03 7B 09 2A 56 00 00

/// WARN: SHOULD BE USED ONLY FOR NODE CONSTRUCTION (see utils::convert_block_to_pointer_if_possible)
pub const PTR_ENDIANNESS: Endianness = Endianness::Little;
pub const MALLOC_HEADER_ENDIANNESS: Endianness = Endianness::Little;

/// Initialize logger. 
/// WARN: Must be called before any logging is done.
fn init_logger() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(LOGGER_MODE.as_str()));

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

    static ref HOME_DIR: String = std::env::var("HOME")
        .expect("HOME environment variable must be set");

    static ref REPO_DIR: String = {
        let repo_dir = std::env::var("REPOSITORY_BASE_DIR")
            .expect("REPOSITORY_BASE_DIR environment variable must be set");
        HOME_DIR.clone() + &repo_dir
    };

    static ref DATA_DIR: String = {
        let data_dir = std::env::var("DATA_BASE_DIR")
            .expect("DATA_BASE_DIR environment variable must be set");
        HOME_DIR.clone() + &data_dir
    };
    
    pub static ref TEST_HEAP_DUMP_FILE_PATH: PathBuf = {
        let test_heap_dump_raw_file_path = std::env::var("TEST_HEAP_DUMP_RAW_FILE_PATH")
            .expect("TEST_HEAP_DUMP_RAW_FILE_PATH environment variable must be set").to_string();
        PathBuf::from(REPO_DIR.clone() + &test_heap_dump_raw_file_path)
    };

    pub static ref TEST_HEAP_JSON_FILE_PATH: PathBuf = {
        crate::utils::heap_dump_path_to_json_path(&TEST_HEAP_DUMP_FILE_PATH)
    };
}
