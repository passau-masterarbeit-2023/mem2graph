use hex::decode;

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
fn test_is_pointer() {
    // all data comes from: /Training/Training/scp/V_7_8_P1/16/1010-1644391327-heap.raw
    // and its associated json file
    let min_addr: u64 = hex_str_to_addr("558b967e9000").unwrap(); // HEAP_START
    let max_addr: u64 = min_addr + hex_str_to_addr("00045000").unwrap();

    let test_cases = vec![
        ("20947e968b550000", Some(hex_str_to_addr("20947e968b550000").unwrap())),
        ("40947e968b550000", Some(hex_str_to_addr("40947e968b550000").unwrap())),
        ("0000000000001000", None),
        ("0000000000001FFF", None),
        ("0000000000000FFF", None),
        ("0000000000002000", None),
        ("0000000000002001", None),
    ];

    for (hex_str, expected_value) in test_cases {
        let data = decode(hex_str).unwrap();
        let result = is_pointer(&data, min_addr, max_addr, Endianness::Big);

        // check if expected value is in range
        let checked_expected_value;
        if expected_value.is_some() &&  expected_value.unwrap() >= min_addr && expected_value.unwrap() <= max_addr {
            checked_expected_value = expected_value;
        } else {
            checked_expected_value = None;
        }

        assert_eq!(
            result, checked_expected_value, "Assert error for {}: {} != {}", 
            hex_str, unwrap_to_string!(result), 
            unwrap_to_string!(expected_value)
        );
    }
}

