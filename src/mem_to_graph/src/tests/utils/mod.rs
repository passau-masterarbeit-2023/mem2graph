use hex::decode;
use lazy_static::lazy_static;

use crate::utils::*;

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
    
    static ref TEST_MIN_ADDR: u64 = hex_str_to_addr("558b967e9000", Endianness::Big).unwrap(); // HEAP_START
    static ref TEST_MAX_ADDR: u64 = *TEST_MIN_ADDR + hex_str_to_addr("00045000", Endianness::Big).unwrap(); // HEAP_START + HEAP_SIZE
    
    static ref TEST_PTR_1_VALUE_STR: String = "20947e968b550000".to_string();
    static ref TEST_PTR_1_VALUE: u64 = hex_str_to_addr(&*TEST_PTR_1_VALUE_STR.as_str(), Endianness::Little).unwrap();
    static ref TEST_PTR_1_ADDR: u64 = *TEST_MIN_ADDR + hex_str_to_addr("00000300", Endianness::Big).unwrap();
    
    static ref TEST_PTR_2_VALUE_STR: String = "40947e968b550000".to_string();
    static ref TEST_PTR_2_VALUE: u64 = hex_str_to_addr(&*TEST_PTR_2_VALUE_STR.as_str(), Endianness::Little).unwrap();
    static ref TEST_PTR_2_ADDR: u64 = *TEST_MIN_ADDR + hex_str_to_addr("00000308", Endianness::Big).unwrap();
}

#[test]
fn test_hex_str_to_block_bytes() {
    // big endian
    let test_cases = vec![
        // (hex_str, expected_value)
        ("0000000000000000", [0, 0, 0, 0, 0, 0, 0, 0], 0),
        ("00000300", [0, 0, 0, 0, 0, 0, 3, 0], 768),
        ("0000000000000400", [0, 0, 0, 0, 0, 0, 4, 0], 1024),
        ("0004000000000000", [0, 4, 0, 0, 0, 0, 0, 0], 1125899906842624u64),
    ];
    for (hex_str, expected_value, decimal) in test_cases {
        let bytes = hex_str_to_block_bytes(hex_str, Endianness::Big);
        assert_eq!(bytes, expected_value);

        let bytes_as_u64 = u64::from_be_bytes(bytes);
        assert_eq!(bytes_as_u64, decimal);

        let expected_value_as_u64 = u64::from_be_bytes(expected_value);
        assert_eq!(expected_value_as_u64, decimal);
    }

    // little endian
    let test_cases = vec![
        // (hex_str, expected_value)
        ("0000000000000000", [0, 0, 0, 0, 0, 0, 0, 0], 0),
        ("00030000", [0, 0, 0, 0, 0, 0, 3, 0], 768),
        ("0004000000000000", [0, 0, 0, 0, 0, 0, 4, 0], 1024),
        ("0000000000000400", [0, 4, 0, 0, 0, 0, 0, 0], 1125899906842624u64),
    ];
    for (hex_str, expected_value, decimal) in test_cases {
        let bytes = hex_str_to_block_bytes(hex_str, Endianness::Little);
        assert_eq!(bytes, expected_value);

        // note: the result of hex_str_to_block_bytes should always be in big endian 
        let bytes_as_u64 = u64::from_be_bytes(bytes);
        assert_eq!(bytes_as_u64, decimal);

        let expected_value_as_u64 = u64::from_be_bytes(expected_value);
        assert_eq!(expected_value_as_u64, decimal);
    }
}

#[test]
fn test_is_pointer() {
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
        let data: [u8; 8] = hex_str_to_block_bytes(hex_str, Endianness::Little);
        let result = is_pointer(&data, min_addr, max_addr, Endianness::Big);

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

// #[test]
// fn test_create_node_from_bytes() {
//     // vector
//     let vector_block_of_8_bytes = decode(&*TEST_PTR_1_VALUE_STR.as_str()).unwrap();
//     let node = create_node_from_bytes(
//         &vector_block_of_8_bytes, 0, 0, 1, Endianness::Little
//     );
//     match node {
//         Node::ValueNode(value_node) => {
//             match value_node {
//                 ValueNode::BaseValueNode(base_value_node) => {
//                     assert_eq!(base_value_node.value, bytes);
//                 }
//                 _ => panic!("Expected a BaseValueNode"),
//             }
//         }
//         _ => panic!("Expected a ValueNode"),
//     }
// }