#[cfg(test)]
use std::sync::Once;
use lazy_static::lazy_static;

use crate::{utils::{hex_str_to_addr, Endianness, hex_str_to_block_bytes}, params::BLOCK_BYTE_SIZE};
use crate::params;

// reference tests from tests/ directory
#[cfg(test)]
mod utils;
mod graph_structs;

#[cfg(test)]
static INIT: Once = Once::new();

/// WARN: Must be called after init()
/// otherwise, the logger will not be initialized
#[cfg(test)]
fn log_order_warning() {
    INIT.call_once(|| {
        log::info!(" 🚧 The order of the logs is not guaranteed. This is because the tests are run in parallel.");
        log::info!(" 🚧 Using 'print' or 'println' won't work because the output is captured by the test runner.");
    });
}

// setup() function is called before each test
#[cfg(test)]
pub fn setup() {
        // initialization code here
        crate::params::init(); // Call the init() function to load the .env file
        log_order_warning();
}

lazy_static! {
    // all data comes from: ~~data/302-1644391327-heap.raw~~ -> data/17016-1643962152-heap.raw (cleaned)
    // and its associated json file
    // WARN: Beware of Endianness, not the same between addr indexes and addr values
    // xxd example:
    //      00000300:20947e968b55000040947e968b550000.~..U..@.~..U..
    // here, "00000300" is in big endian, but "20947e968b550000" is in little endian
    // NOTE: 00000300 is the index of the 8 bytes (32 bits) block containing the pointer 20947e968b550000
    // NOTE: 00000308 is the index of the 8 bytes (32 bits) block containing the pointer 40947e968b550000
    // NOTE: pointer representation is in little endian, and ends with 00 00
    
    // WARN: HEAP_START is in big endian!!!
    
    pub static ref TEST_HEAP_START_ADDR: u64 = hex_str_to_addr("558343d1a000", Endianness::Big).unwrap(); // HEAP_START
    static ref TEST_HEAP_SIZE: u64 = 135168; // heap size obtained with command: stat -c %s test/17016-1643962152-heap.raw
    pub static ref TEST_HEAP_END_ADDR: u64 = *TEST_HEAP_START_ADDR + *TEST_HEAP_SIZE; // HEAP_START + HEAP_SIZE

    pub static ref TEST_PTR_1_VALUE_STR: String = "d061d24383550000".to_string();
    pub static ref TEST_PTR_1_VALUE: u64 = hex_str_to_addr(&*TEST_PTR_1_VALUE_STR.as_str(), Endianness::Little).unwrap();
    pub static ref TEST_PTR_1_VALUE_BYTES: [u8; BLOCK_BYTE_SIZE] = hex_str_to_block_bytes(TEST_PTR_1_VALUE_STR.as_str());
    pub static ref TEST_PTR_1_ADDR: u64 = *TEST_HEAP_START_ADDR + hex_str_to_addr("00000050", Endianness::Big).unwrap();
    
    pub static ref TEST_PTR_2_VALUE_STR: String = "306bd24383550000".to_string();
    pub static ref TEST_PTR_2_VALUE: u64 = hex_str_to_addr(&*TEST_PTR_2_VALUE_STR.as_str(), Endianness::Little).unwrap();
    pub static ref TEST_PTR_2_ADDR: u64 = *TEST_HEAP_START_ADDR + hex_str_to_addr("00000060", Endianness::Big).unwrap();
    pub static ref TEST_PTR_2_VALUE_BYTES: [u8; BLOCK_BYTE_SIZE] = hex_str_to_block_bytes(TEST_PTR_2_VALUE_STR.as_str());

    pub static ref TEST_VAL_1_VALUE_STR: String = "2f746d702f737368".to_string();
    pub static ref TEST_VAL_1_VALUE: u64 = hex_str_to_addr(&*TEST_VAL_1_VALUE_STR.as_str(), Endianness::Little).unwrap();
    pub static ref TEST_VAL_1_ADDR: u64 = *TEST_HEAP_START_ADDR + hex_str_to_addr("000002a0", Endianness::Big).unwrap();
    pub static ref TEST_VAL_1_VALUE_BYTES: [u8; BLOCK_BYTE_SIZE] = hex_str_to_block_bytes(TEST_VAL_1_VALUE_STR.as_str());

    // data structure
    // 00000290:00000000000000002100000000000000........!.......
    // 000002a0:2f746d702f7373686400000000000000/tmp/sshd.......
    pub static ref TEST_MALLOC_HEADER_1_CHUNK_STR: String = "5102000000000000".to_string();
    pub static ref TEST_MALLOC_HEADER_1_CHUNK_SIZE: usize = hex_str_to_addr(&*TEST_MALLOC_HEADER_1_CHUNK_STR.as_str(), params::MALLOC_HEADER_ENDIANNESS).unwrap() as usize;
    pub static ref TEST_MALLOC_HEADER_1_ADDR: u64 = *TEST_HEAP_START_ADDR + hex_str_to_addr("00000008", Endianness::Big).unwrap();

    pub static ref TEST_GRAPH_DOT_DIR_PATH: String = "test/graphs/".to_string();
    pub static ref TEST_HEAP_DUMP_FILE_NUMBER: String = "17016-1643962152".to_string(); // 17016-1643962152-heap.raw

    // key F 
    pub static ref TEST_KEY_F_ADDR_STR: String = "558343d20e90".to_string();
    pub static ref TEST_KEY_F_ADDR: u64 = hex_str_to_addr(&*TEST_KEY_F_ADDR_STR.as_str(), Endianness::Big).unwrap();
    pub static ref TEST_KEY_F_BYTES: Vec<u8> = hex::decode("60a2915bc3bedc7b58b763f2ea0c8b85").unwrap();
    pub static ref TEST_KEY_F_NAME: String = "KEY_F".to_string();
    pub static ref TEST_KEY_F_LEN: usize = TEST_KEY_F_BYTES.len();

    // special nodes
    pub static ref TEST_SSH_STRUCT_ADDR_STR: String = "558343d2aec0".to_string();
    pub static ref TEST_SSH_STRUCT_ADDR: u64 = hex_str_to_addr(&*TEST_SSH_STRUCT_ADDR_STR.as_str(), Endianness::Big).unwrap();
    pub static ref TEST_SESSION_STATE_ADDR_STR: String = "558343d2a620".to_string();
    pub static ref TEST_SESSION_STATE_ADDR: u64 = hex_str_to_addr(&*TEST_SESSION_STATE_ADDR_STR.as_str(), Endianness::Big).unwrap();

}
