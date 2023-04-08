use std::sync::Once;
use lazy_static::lazy_static;

use crate::{utils::{hex_str_to_addr, Endianness, hex_str_to_block_bytes}, params::BLOCK_BYTE_SIZE};

// reference tests from tests/ directory
#[cfg(test)]
mod utils;
mod graph_structs;

static INIT: Once = Once::new();

/// WARN: Must be called after init()
/// otherwise, the logger will not be initialized
fn log_order_warning() {
    INIT.call_once(|| {
        log::info!(" 🚧 The order of the logs is not guaranteed. This is because the tests are run in parallel.");
        log::info!(" 🚧 Using 'print' or 'println' won't work because the output is captured by the test runner.");
    });
}


// setup() function is called before each test
pub fn setup() {
        // initialization code here
        crate::params::init(); // Call the init() function to load the .env file
        log_order_warning();
}

lazy_static! {
    // all data comes from: /Training/Training/scp/V_7_8_P1/16/1010-1644391327-heap.raw
    // and its associated json file
    // WARN: Beware of Endianness, not the same between addr indexes and addr values
    // xxd example:
    //      00000300:20947e968b55000040947e968b550000.~..U..@.~..U..
    // here, "00000300" is in big endian, but "20947e968b550000" is in little endian
    // NOTE: 00000300 is the index of the 8 bytes (32 bits) block containing the pointer 20947e968b550000
    // NOTE: 00000308 is the index of the 8 bytes (32 bits) block containing the pointer 40947e968b550000
    // NOTE: pointer representation is in little endian, and ends with 00 00
    
    // WARN: HEAP_START is in big endian!!!
    // test range: [620_599_085_909 ... 620_599_085_909 + 282_624 = 620_599_368_533]
    // Big endian HEAP_START range [94_058_013_691_904 ]
    // 94_058_013_692_960 not in range
    // 19291223004192 not in range
    
    pub static ref TEST_MIN_ADDR: u64 = hex_str_to_addr("558b967e9000", Endianness::Big).unwrap(); // HEAP_START
    pub static ref TEST_MAX_ADDR: u64 = *TEST_MIN_ADDR + hex_str_to_addr("00045000", Endianness::Big).unwrap(); // HEAP_START + HEAP_SIZE

    pub static ref TEST_PTR_1_VALUE_STR: String = "20947e968b550000".to_string();
    pub static ref TEST_PTR_1_VALUE: u64 = hex_str_to_addr(&*TEST_PTR_1_VALUE_STR.as_str(), Endianness::Little).unwrap();
    pub static ref TEST_PTR_1_VALUE_BYTES: [u8; BLOCK_BYTE_SIZE] = hex_str_to_block_bytes(
        TEST_PTR_1_VALUE_STR.as_str(), 
        Endianness::Big // NOTE: the bytes are in big endian, to simulate the raw memory extraction from the dump
    );
    pub static ref TEST_PTR_1_ADDR: u64 = *TEST_MIN_ADDR + hex_str_to_addr("00000300", Endianness::Big).unwrap();
    
    pub static ref TEST_PTR_2_VALUE_STR: String = "40947e968b550000".to_string();
    pub static ref TEST_PTR_2_VALUE: u64 = hex_str_to_addr(&*TEST_PTR_2_VALUE_STR.as_str(), Endianness::Little).unwrap();
    pub static ref TEST_PTR_2_ADDR: u64 = *TEST_MIN_ADDR + hex_str_to_addr("00000308", Endianness::Big).unwrap();
}