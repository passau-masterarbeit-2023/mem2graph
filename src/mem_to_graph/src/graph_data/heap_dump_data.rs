use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use crate::utils;

pub struct HeapDumpData {
    pub block_size: usize,
    pub blocks: Vec<Vec<u8>>,
    pub heap_dump_raw_file_path: PathBuf,
    pub min_addr: u64,
    pub max_addr: u64,
    pub json_data: Value,
}

impl HeapDumpData {

    /// Constructor for HeapDumpData
    /// It contains data for a given heap dump file, 
    /// some of them are obtained from the associated json file
    pub fn new(
        heap_dump_raw_file_path: PathBuf,
        block_size: usize,
    ) -> HeapDumpData {
        // check if file exists
        if !heap_dump_raw_file_path.exists() {
            log::error!("File doesn't exist: {:?}", heap_dump_raw_file_path);
        } else {
            log::info!(" 📋 heap dump raw file path: {:?}", heap_dump_raw_file_path);
        }

        let json_path = utils::heap_dump_path_to_json_path(&heap_dump_raw_file_path);
        let blocks = HeapDumpData::generate_blocks_from_heap_dump(&heap_dump_raw_file_path, block_size);
        let json_data = HeapDumpData::get_json_data(&json_path);
        let (min_addr, max_addr) = HeapDumpData::get_min_max_addr(&json_data, blocks.len(), block_size);

        HeapDumpData {
            block_size,
            blocks,
            heap_dump_raw_file_path: heap_dump_raw_file_path,
            min_addr,
            max_addr,
            json_data,
        }
    }

    pub fn addr_to_index_wrapper(&self, addr: u64) -> usize {
        crate::utils::addr_to_index(addr, self.min_addr, self.block_size)
    }

    pub fn index_to_addr_wrapper(&self, index: usize) -> u64 {
        crate::utils::index_to_addr(index, self.min_addr, self.block_size)
    }

    /// load json file
    fn get_json_data(json_file_path: &PathBuf) -> Value {
        let file = File::open(json_file_path).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    }

    /// load heap dump file and split it into blocks
    fn generate_blocks_from_heap_dump(heap_dump_raw_file_path: &PathBuf, block_size: usize) -> Vec<Vec<u8>> {
        let mut file = File::open(heap_dump_raw_file_path).unwrap();
        let mut heap_dump = Vec::new();
        file.read_to_end(&mut heap_dump).unwrap();

        heap_dump
            .chunks(block_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    /// get min and max address from json file to a given heap dump
    fn get_min_max_addr(json_data: &Value, nb_blocks: usize, block_size: usize) -> (u64, u64) {
        let min_addr_str = json_data["HEAP_START"].as_str().unwrap();
        let min_addr = u64::from_str_radix(min_addr_str.trim_start_matches("0x"), 16).unwrap();
        let max_addr = min_addr + (nb_blocks as u64) * (block_size as u64);
        (min_addr, max_addr)
    }
}



// NOTE: tests must be in the same module as the code they are testing
// for them to have access to the private functions
#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{
        BLOCK_BYTE_SIZE, 
        TEST_HEAP_DUMP_FILE_PATH
    };

    #[test]
    fn test_object_creation() {
        crate::tests::setup();
        let heap_dump_data = HeapDumpData::new(
            TEST_HEAP_DUMP_FILE_PATH.clone(), 
            BLOCK_BYTE_SIZE
        );

        assert_eq!(heap_dump_data.block_size, BLOCK_BYTE_SIZE);
        assert_eq!(heap_dump_data.heap_dump_raw_file_path.to_str(), TEST_HEAP_DUMP_FILE_PATH.to_str());
    }

    #[test]
    fn test_get_json_data() {
        crate::tests::setup();

        let json_data = HeapDumpData::get_json_data(
            &crate::params::TEST_HEAP_JSON_FILE_PATH,
        );

        assert!(json_data.is_object());
        assert!(json_data["HEAP_START"].is_string());
        let heap_start = u64::from_str_radix(json_data["HEAP_START"].as_str().unwrap(), 16).unwrap();
        let test_heap_addr = "55a6d2356000";
        let test_heap_addr_converted = u64::from_str_radix(test_heap_addr, 16).unwrap();
        assert!(heap_start == test_heap_addr_converted);
    }

    #[test]
    fn test_generate_blocks_from_heap_dump() {
        crate::tests::setup();
        let blocks = HeapDumpData::generate_blocks_from_heap_dump(&*TEST_HEAP_DUMP_FILE_PATH, BLOCK_BYTE_SIZE);

        assert!(!blocks.is_empty());
        assert_eq!(blocks[0].len(), BLOCK_BYTE_SIZE);
    }

    #[test]
    fn test_get_min_max_addr() {
        crate::tests::setup();

        let blocks = HeapDumpData::generate_blocks_from_heap_dump(
            &crate::params::TEST_HEAP_DUMP_FILE_PATH, BLOCK_BYTE_SIZE
        );
        let json_data = HeapDumpData::get_json_data(
            &crate::params::TEST_HEAP_JSON_FILE_PATH
        );
        let (min_addr, max_addr) = HeapDumpData::get_min_max_addr(
            &json_data, blocks.len(), BLOCK_BYTE_SIZE);

        assert!(min_addr < max_addr);
    }

    #[test]
    fn test_addr_to_index_wrapper_and_index_to_addr_wrapper() {
        crate::tests::setup();
        let heap_dump_data = HeapDumpData::new(TEST_HEAP_DUMP_FILE_PATH.clone(), BLOCK_BYTE_SIZE);
        let addr = heap_dump_data.min_addr + 2 * BLOCK_BYTE_SIZE as u64;

        let index = heap_dump_data.addr_to_index_wrapper(addr);
        let addr_back = heap_dump_data.index_to_addr_wrapper(index);

        assert_eq!(addr, addr_back);
    }
}
