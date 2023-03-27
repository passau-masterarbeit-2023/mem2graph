use petgraph::Graph;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use crate::graph_data::heap_dump_data::HeapDumpData;

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_HEAP_DUMP_FILE_PATH: &str = "path/to/test/heap_dump.raw";
    const TEST_BLOCK_SIZE: usize = 8;

    #[test]
    fn test_object_creation() {
        let heap_dump_data = HeapDumpData::new(TEST_HEAP_DUMP_FILE_PATH, TEST_BLOCK_SIZE);

        assert_eq!(heap_dump_data.block_size, TEST_BLOCK_SIZE);
        assert_eq!(heap_dump_data.heap_dump_raw_file_path, Path::new(TEST_HEAP_DUMP_FILE_PATH));
    }

    #[test]
    fn test_get_json_data() {
        let json_data = HeapDumpData::get_json_data(
            TEST_HEAP_DUMP_FILE_PATH.replace("-heap.raw", ".json").as_str(),
        );

        assert!(json_data.is_object());
        assert!(json_data["HEAP_START"].is_string());
        assert!(json_data["HEAP_SIZE"].is_u64());
    }

    #[test]
    fn test_generate_blocks_from_heap_dump() {
        let blocks = HeapDumpData::generate_blocks_from_heap_dump(TEST_HEAP_DUMP_FILE_PATH, TEST_BLOCK_SIZE);

        assert!(!blocks.is_empty());
        assert_eq!(blocks[0].len(), TEST_BLOCK_SIZE);
    }

    #[test]
    fn test_get_min_max_addr() {
        let json_data = HeapDumpData::get_json_data(
            TEST_HEAP_DUMP_FILE_PATH.replace("-heap.raw", ".json").as_str(),
        );
        let (min_addr, max_addr) = HeapDumpData::get_min_max_addr(&json_data);

        assert!(min_addr < max_addr);
    }

    #[test]
    fn test_addr_to_index_wrapper_and_index_to_addr_wrapper() {
        let heap_dump_data = HeapDumpData::new(TEST_HEAP_DUMP_FILE_PATH, TEST_BLOCK_SIZE);
        let addr = heap_dump_data.min_addr + 2 * TEST_BLOCK_SIZE as u64;

        let index = heap_dump_data.addr_to_index_wrapper(addr);
        let addr_back = heap_dump_data.index_to_addr_wrapper(index);

        assert_eq!(addr, addr_back);
    }
}
