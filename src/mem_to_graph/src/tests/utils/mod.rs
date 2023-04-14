use crate::tests::*;
use crate::utils::*;
use crate::params::TEST_HEAP_DUMP_FILE_PATH;

use serde_json::json;

#[test]
fn test_addr_to_index() {
    crate::tests::setup();
    assert_eq!(addr_to_index(1000, 1000, 100), 0);
    assert_eq!(addr_to_index(1100, 1000, 100), 1);
    assert_eq!(addr_to_index(1200, 1000, 100), 2);
    assert_eq!(addr_to_index(1300, 1000, 50), 6);
    assert_eq!(addr_to_index(1500, 1000, 250), 2);
}

#[test]
fn test_index_to_addr() {
    crate::tests::setup();
    assert_eq!(index_to_addr(0, 1000, 100), 1000);
    assert_eq!(index_to_addr(1, 1000, 100), 1100);
    assert_eq!(index_to_addr(2, 1000, 100), 1200);
    assert_eq!(index_to_addr(6, 1000, 50), 1300);
    assert_eq!(index_to_addr(2, 1000, 250), 1500);
}

#[test]
fn test_addr_to_index_and_index_to_addr() {
    crate::tests::setup();
    let min_addr = 1000u64;
    let block_size = 100usize;

    for addr in (min_addr..(min_addr * 2)).step_by(block_size) {
        let index = addr_to_index(addr, min_addr, block_size);
        let converted_addr = index_to_addr(index, min_addr, block_size);
        assert_eq!(addr, converted_addr);
    }
}

macro_rules! unwrap_to_string {
    ($expected_value:expr) => {
        match $expected_value {
            Some(v) => v.to_string(),
            None => "None".to_string(),
        }
    };
}

#[test]
fn test_hex_str_to_addr() {
    crate::tests::setup();
    // 16 hex chars = 4 * 16 bits = 64 bits
    assert_eq!(hex_str_to_addr("0000000000000000", Endianness::Big).unwrap(), 0);
    assert_eq!(hex_str_to_addr("00000300", Endianness::Big).unwrap(), 768);
    assert_eq!(hex_str_to_addr("00030000", Endianness::Little).unwrap(), 768);

    /* The little-endian representation of 0x0000000000000400 is indeed 
    0x0004000000000000. The reason for the additional "0" is that 
    little-endian representation reverses the order of the bytes 
    and not the individual nibbles (4-bit groups) in the hexadecimal 
    representation.

    big endian of 1024:     00 00 00 00 00 00 04 00
    little endian of 1024:  00 04 00 00 00 00 00 00 */
    assert_eq!(hex_str_to_addr("0000000000000400", Endianness::Big).unwrap(), 1024);
    assert_eq!(hex_str_to_addr("0004000000000000", Endianness::Little).unwrap(), 1024);
}



#[test]
fn test_hex_str_to_block_bytes() {
    crate::tests::setup();
    // big endian
    let test_cases = vec![
        // (hex_str, expected_value)
        ("0000000000000000", [0, 0, 0, 0, 0, 0, 0, 0]),
        ("0000000000000100", [0, 0, 0, 0, 0, 0, 1, 0]),
        ("0000000000000200", [0, 0, 0, 0, 0, 0, 2, 0]),
        ("0003000000000000", [0, 3, 0, 0, 0, 0, 0, 0]),
        ("0001020304050607", [0, 1, 2, 3, 4, 5, 6, 7]),
    ];
    for (hex_str, expected_value) in test_cases {
        let bytes_to_test = hex_str_to_block_bytes(hex_str);
        assert_eq!(bytes_to_test, expected_value);
    }
}

#[test]
fn test_is_pointer() {
    crate::tests::setup();
    let min_addr: u64 = *TEST_MIN_ADDR; // HEAP_START
    let max_addr: u64 = *TEST_MAX_ADDR; // HEAP_START + HEAP_SIZE

    let test_cases = vec![
        // pointers, in little endian
        (&*TEST_PTR_1_VALUE_STR.as_str(), Some(*TEST_PTR_1_VALUE)),
        (&*TEST_PTR_2_VALUE_STR.as_str(), Some(*TEST_PTR_2_VALUE)),
        // integers, in big endian
        ("0000000000001000", None), 
        ("0000000000001FFF", None),
    ];

    for (hex_str, expected_value) in test_cases {
        // use helper function to convert hex string to big endian bytes
        let data: [u8; 8] = hex_str_to_block_bytes(hex_str);
        let result = convert_block_to_pointer_if_possible(&data, min_addr, max_addr);

        assert!(
            // check if expected value is in range when it is not None
            expected_value.is_none() || (expected_value.is_some() && expected_value.unwrap() >= min_addr && expected_value.unwrap() <= max_addr),
            "Expected value ({}) is not in range", unwrap_to_string!(expected_value)
        );
        assert_eq!(
            result.unwrap_or_default(), 
            expected_value.unwrap_or_default(), 
            "Assert error for {}: {} != {}", hex_str, unwrap_to_string!(result), unwrap_to_string!(expected_value)
        );

        // log info
        if result.is_some() {
            log::info!("0x{} is a pointer to {}", hex_str, result.unwrap());
        } else {
            log::info!("0x{} is not a pointer", hex_str);
        }
    }
}

#[test]
fn test_create_node_from_bytes() {
    crate::tests::setup();
    // pointer 1 test
    let pointer_block_of_8_bytes = hex_str_to_block_bytes(&*TEST_PTR_1_VALUE_STR.as_str());
    let mut node = create_node_from_bytes(
        &pointer_block_of_8_bytes, 
        *TEST_PTR_1_ADDR, 
        *TEST_MIN_ADDR, 
        *TEST_MAX_ADDR,
    );
    assert_eq!(node.get_address(), *TEST_PTR_1_ADDR);
    log::debug!("node1: {:?}, data: {:?}", node, pointer_block_of_8_bytes);
    assert!(node.is_pointer());
    //assert_eq!(node.get_pointer_value(), *TEST_PTR_1_VALUE);

    // value node
    let value_block_of_8_bytes = hex_str_to_block_bytes("a3341294ab2bd410");
    node = create_node_from_bytes(
        &value_block_of_8_bytes, 
        *TEST_PTR_1_ADDR, 
        *TEST_MIN_ADDR, 
        *TEST_MAX_ADDR, 
    );
    assert_eq!(node.get_address(), *TEST_PTR_1_ADDR);
    assert!(node.is_value());

    // test with a real pointer but wrong endianness
    // 
    let pointer_block_of_8_bytes = hex_str_to_block_bytes(&*TEST_PTR_1_VALUE_STR.as_str());
    node = create_node_from_bytes(
        &pointer_block_of_8_bytes, 
        *TEST_PTR_1_ADDR, 
        *TEST_MIN_ADDR, 
        *TEST_MAX_ADDR, 
    );
    log::debug!("node2: {:?}, data: {:?}", node, pointer_block_of_8_bytes);
    assert_eq!(node.get_address(), *TEST_PTR_1_ADDR);
    assert!(node.is_pointer());
}

#[test]
fn test_heap_dump_path_to_json_path() {
    crate::tests::setup();
    let test_path = heap_dump_path_to_json_path(&*TEST_HEAP_DUMP_FILE_PATH);
    assert!(test_path.exists())
}

#[test]
fn test_block_bytes_to_addr() {
    crate::tests::setup();
    let test_cases = vec![
        // (hex_str, expected_value_big_endian, expected_value_little_endian)
        // WARN: online convertors tend to make mistakes when working on huge numbers
        // use Python: int.from_bytes(bytes.fromhex("0001020304050607"), 'little')
        ("0000000000000000", 0, 0),
        ("0000000000000100", 256, 281474976710656),
        ("0000000000000200", 512, 562949953421312),
        ("0003000000000000", 844424930131968, 768),
        ("0fffffffffffff0f", 1152921504606846735, 1152921504606846735),
        ("0001020304050607", 283686952306183, 506097522914230528),
    ];
    for (hex_str, expected_value_big_endian, expected_value_little_endian) in test_cases {
        let bytes_to_test = hex_str_to_block_bytes(hex_str);
        let result = block_bytes_to_addr(&bytes_to_test, Endianness::Big);
        assert_eq!(result, expected_value_big_endian);

        let bytes_to_test = hex_str_to_block_bytes(hex_str);
        let result = block_bytes_to_addr(&bytes_to_test, Endianness::Little);
        assert_eq!(result, expected_value_little_endian);
    }
}

#[test]
fn test_json_value_to_addr() {
    crate::tests::setup();
    
    let json_value = json!("12345678");

    let expected_addr: u64 = 0x12345678;
    let actual_addr = json_value_to_addr(&json_value);

    assert_eq!(actual_addr, expected_addr, "The address should be equal to the expected value");
}