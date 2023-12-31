use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::collections::HashMap;


use crate::graph_structs::annotations::KeyDataJSON;
use crate::utils::{self, json_value_to_addr, json_value_to_usize, json_value_for_key, ErrorKind};
use crate::params::BLOCK_BYTE_SIZE;

pub struct HeapDumpData {
    pub block_size: usize,
    pub blocks: Vec<[u8; BLOCK_BYTE_SIZE]>,
    pub heap_dump_raw_file_path: PathBuf,
    pub min_addr: u64,
    pub max_addr: u64,
    pub json_data: Value,
    pub addr_to_key_data: HashMap<u64, KeyDataJSON>,

    // special addresses
    pub addr_ssh_struct: Option<u64>,
    pub addr_session_state: Option<u64>,
}

impl HeapDumpData {

    /// Constructor for HeapDumpData
    /// It contains data for a given heap dump file, 
    /// some of them are obtained from the associated json file
    pub fn new(
        heap_dump_raw_file_path: PathBuf,
        block_size: usize,
        annotation : bool,
    ) -> Result<HeapDumpData, crate::utils::ErrorKind>  {
        // check if file exists
        if !heap_dump_raw_file_path.exists() {
            log::error!("File doesn't exist: {:?}", heap_dump_raw_file_path);
        } else {
            log::info!(" 📋 heap dump raw file path: {:?}", heap_dump_raw_file_path);
        }

        let json_path = utils::heap_dump_path_to_json_path(&heap_dump_raw_file_path);
        let blocks = HeapDumpData::generate_blocks_from_heap_dump(&heap_dump_raw_file_path, block_size);
        
        
        let potential_json_data = HeapDumpData::get_json_data(&json_path);
        let json_data;
        match potential_json_data {
            Ok(data) => {
                json_data = data;
                //log::info!(" 📋 json file path: {:?}", json_path);
            },
            Err(_) => {
                log::error!("File doesn't exist: {:?}", json_path);
                return Err(ErrorKind::JsonFileNotFound(json_path));
            }
        }
        
        
        let (min_addr, max_addr) = HeapDumpData::get_min_max_addr(&json_data, blocks.len(), block_size)?;
        let addr_to_key_data;
        if annotation {
            addr_to_key_data = generate_key_data_from_json(&json_data)?;
        } else {
            addr_to_key_data = HashMap::new();
        }

        // special addresses
        let addr_ssh_struct;
        let addr_session_state;
        if annotation {
            addr_ssh_struct = Some(json_value_to_addr(json_value_for_key(&json_data, "SSH_STRUCT_ADDR".to_string())?));
            addr_session_state = Some(json_value_to_addr(json_value_for_key(&json_data, "SESSION_STATE_ADDR".to_string())?));
        } else {
            addr_ssh_struct = None;
            addr_session_state = None;
        }

        Ok(HeapDumpData {
            block_size,
            blocks,
            heap_dump_raw_file_path: heap_dump_raw_file_path,
            min_addr,
            max_addr,
            json_data,
            addr_to_key_data,
            addr_ssh_struct,
            addr_session_state,
        })
    }

    #[cfg(test)]
    pub fn addr_to_index_wrapper(&self, addr: u64) -> usize {
        crate::utils::addr_to_index(addr, self.min_addr, self.block_size)
    }

    pub fn index_to_addr_wrapper(&self, index: usize) -> u64 {
        crate::utils::index_to_addr(index, self.min_addr, self.block_size)
    }

    /// load json file
    fn get_json_data(json_file_path: &PathBuf) -> Result<Value, std::io::Error> {
        let file = File::open(json_file_path).unwrap();
        let reader = BufReader::new(file);
        let res = serde_json::from_reader(reader)?;
        Ok(res)
    }

    /// load heap dump file and split it into blocks
    fn generate_blocks_from_heap_dump(heap_dump_raw_file_path: &PathBuf, block_size: usize) -> Vec<[u8; BLOCK_BYTE_SIZE]> {
        let mut file = File::open(heap_dump_raw_file_path).unwrap();
        let mut heap_dump = Vec::new();
        file.read_to_end(&mut heap_dump).unwrap();

        let mut blocks = Vec::new();
        for chunk in heap_dump.chunks(block_size) {
            let mut block = [0u8; BLOCK_BYTE_SIZE];
            block[..chunk.len()].copy_from_slice(chunk);
            blocks.push(block);
        }
        blocks
    }

    /// get min and max address from json file to a given heap dump
    fn get_min_max_addr(json_data: &Value, nb_blocks: usize, block_size: usize) -> Result<(u64, u64), ErrorKind> {
        let min_addr = json_value_to_addr(json_value_for_key(&json_data, "HEAP_START".to_string())?);
        let max_addr = min_addr + (nb_blocks as u64) * (block_size as u64);
        Ok((min_addr, max_addr))
    }




}

/// Generate a dictionary of key data from the JSON file.
/// dict keys are addresses of the keys (first block of the key)
fn generate_key_data_from_json( 
    json_data: &Value,
) -> Result<HashMap<u64, KeyDataJSON>, ErrorKind> {
    let mut addr_key_pairs: HashMap<u64, KeyDataJSON> = HashMap::new();

    for (json_key, _) in json_data.as_object().unwrap().iter() {
        if json_key.starts_with("KEY_") && json_key.len() == 5 {
            let key_data: KeyDataJSON = generate_key_data_for_a_key(&json_data, json_key)?;

            addr_key_pairs.insert(key_data.addr, key_data);
        }
    }

    log::debug!("Number of keys in JSON: {}", addr_key_pairs.len());

    Ok(addr_key_pairs)
}

fn generate_key_data_for_a_key(
    json_data: &Value,
    key_name: &str,
) -> Result<KeyDataJSON, ErrorKind> {
    let key_value = json_value_for_key(&json_data, key_name.to_string())?;
    let key_hex: &str = key_value.as_str().unwrap();

    let real_key_addr = json_value_to_addr(json_value_for_key(&json_data, (key_name.to_owned() + "_ADDR").to_string())?);
    let key_bytes: Vec<u8> = hex::decode(key_hex).unwrap();

    let key_size = json_value_to_usize(json_value_for_key(&json_data, (key_name.to_owned() + "_LEN").to_string())?);
    let real_key_len = json_value_to_usize(json_value_for_key(&json_data, (key_name.to_owned() + "_REAL_LEN").to_string())?);

    Ok(KeyDataJSON {
        name: key_name.to_string(),
        key: key_bytes,
        addr: real_key_addr,
        len: key_size,
        real_len: real_key_len,
    })
}

// NOTE: tests must be in the same module as the code they are testing
// for them to have access to the private functions
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{params::{
        BLOCK_BYTE_SIZE, 
        TEST_HEAP_DUMP_FILE_PATH
    }, tests::TEST_HEAP_START_ADDR};

    #[test]
    fn test_object_creation() {
        crate::tests::setup();
        let heap_dump_data: HeapDumpData = HeapDumpData::new(
            TEST_HEAP_DUMP_FILE_PATH.clone(), 
            BLOCK_BYTE_SIZE,
            true
        ).unwrap();

        assert_eq!(heap_dump_data.block_size, BLOCK_BYTE_SIZE);
        assert_eq!(heap_dump_data.heap_dump_raw_file_path.to_str(), TEST_HEAP_DUMP_FILE_PATH.to_str());
    }

    #[test]
    fn test_get_json_data() {
        crate::tests::setup();

        let json_data = HeapDumpData::get_json_data(
            &crate::params::TEST_HEAP_JSON_FILE_PATH,
        ).unwrap();

        assert!(json_data.is_object());
        assert!(json_data["HEAP_START"].is_string());
        let heap_start = json_value_to_addr(&json_data["HEAP_START"]);
        let test_heap_addr = *TEST_HEAP_START_ADDR;
        assert!(heap_start == test_heap_addr);
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
        ).unwrap();
        let (min_addr, max_addr) = HeapDumpData::get_min_max_addr(
            &json_data, blocks.len(), BLOCK_BYTE_SIZE).unwrap();

        assert!(min_addr < max_addr);
    }

    #[test]
    fn test_addr_to_index_wrapper_and_index_to_addr_wrapper() {
        crate::tests::setup();
        let heap_dump_data = HeapDumpData::new(
            TEST_HEAP_DUMP_FILE_PATH.clone(), 
            BLOCK_BYTE_SIZE,
            true
        ).unwrap();
        let addr = heap_dump_data.min_addr + 2 * BLOCK_BYTE_SIZE as u64;

        let index = heap_dump_data.addr_to_index_wrapper(addr);
        let addr_back = heap_dump_data.index_to_addr_wrapper(index);

        assert_eq!(addr, addr_back);
    }

    #[test]
    fn test_generate_key_data_from_json() {
        crate::tests::setup();
        
        let json_data = HeapDumpData::get_json_data(
            &*crate::params::TEST_HEAP_JSON_FILE_PATH
        ).unwrap();
        let addr_to_key_data = generate_key_data_from_json(&json_data).unwrap();

        assert_eq!(addr_to_key_data.len(), 6); // 6 keys, from A to F

        // test key F
        assert!(addr_to_key_data.get(&*crate::tests::TEST_KEY_F_ADDR).is_some());
        assert!(addr_to_key_data.get(&*crate::tests::TEST_KEY_F_ADDR).unwrap().key == *crate::tests::TEST_KEY_F_BYTES);
    }
}
