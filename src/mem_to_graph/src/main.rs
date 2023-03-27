use crate::graph_data::heap_dump_data::HeapDumpData;

// modules
mod params;
mod tests;
mod graph_data;



fn main() {
    crate::params::init();

    // heap dump data
    let heap_dump_data = HeapDumpData::new(
        params::TEST_HEAP_DUMP_FILE_PATH.clone(),
        params::BLOCK_BYTE_SIZE.try_into().unwrap()
    );
}