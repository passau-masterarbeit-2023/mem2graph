use petgraph::graph::{DiGraph};
use std::path::{PathBuf};

pub mod heap_dump_data;

use heap_dump_data::HeapDumpData;
use crate::graph_structs;
use crate::params::BLOCK_BYTE_SIZE;
use crate::utils::*;

/// This struct contains the graph data
/// linked to a given heap dump file.
pub struct GraphData {
    graph: DiGraph<graph_structs::Node, graph_structs::Edge>,
    heap_dump_data: Option<HeapDumpData>, // Some because it is an optional field, for testing purposes
}

impl GraphData {

    /// Initialize the graph data from a raw heap dump file.
    fn new(&self, heap_dump_raw_file_path: PathBuf, pointer_byte_size: usize) -> GraphData {
        // Get the heap dump data
        self.heap_dump_data = Some(
            HeapDumpData::new(
                heap_dump_raw_file_path,
                pointer_byte_size,
            )
        );

        self.data_structure_step(pointer_byte_size);
        self.pointer_step();
        return *self;
    }

    // constructor for testing purposes
    fn new_test(&self, nodes: Vec<graph_structs::Node>, edges: Vec<graph_structs::Edge>) -> GraphData {
        self.heap_dump_data = None;

        self.graph = DiGraph::new();
        for node in nodes {
            self.add_node_wrapper(&node);
        }
        for edge in edges {
            self.add_edge_wrapper(&edge);
        }
        return *self;
    }

    // wrappers
    // def __create_node_from_bytes_wrapper(self, data: bytes, addr: int):
    // if self.heap_dump_data is None:
    //     raise ValueError("heap_dump_data is None")
    // return create_node_from_bytes(data, addr, self.heap_dump_data.min_addr, self.heap_dump_data.max_addr, self.params.PTR_ENDIANNESS)

    // def __create_node_from_bytes_wrapper_index(self, data: bytes, block_index: int):
    //     if self.heap_dump_data is None:
    //         raise ValueError("heap_dump_data is None")
    //     addr = self.heap_dump_data.index_to_addr_wrapper(block_index)
    //     return self.__create_node_from_bytes_wrapper(data, addr)

    fn create_node_from_bytes_wrapper(
        &self, data: &[u8; BLOCK_BYTE_SIZE], addr: u64
    ) -> graph_structs::Node {
        if self.heap_dump_data.is_none() {
            panic!("heap_dump_data is None");
        }
        return create_node_from_bytes(
            data,
            addr,
            self.heap_dump_data.unwrap().min_addr,
            self.heap_dump_data.unwrap().max_addr
        );
    }
    
    /// Wrapper for create_node_from_bytes_wrapper using a block index instead of an address.
    fn create_node_from_bytes_wrapper_index(
        &self, data: &[u8; BLOCK_BYTE_SIZE], block_index: usize
    ) -> graph_structs::Node {
        let addr = self.heap_dump_data.unwrap().index_to_addr_wrapper(block_index);
        return self.create_node_from_bytes_wrapper(data, addr);
    }

}

// impl GraphData {
//     // Init

//     fn file_init(&mut self, heap_dump_raw_file_path: &str, pointer_byte_size: usize) {
//         // Get the heap dump data
//         self.heap_dump_data = Some(HeapDumpData::new(
//             heap_dump_raw_file_path,
//             pointer_byte_size,
//             &self.params,
//         ));

//         self.data_structure_step(pointer_byte_size);
//         self.pointer_step();
//     }

//     fn test_graph_init(&mut self, nodes: Vec<Node>, edges: Vec<(Node, Node, usize)>) {
//         self.graph = DiGraph::new();
//         for node in nodes {
//             self.add_node_wrapper(&node);
//         }
//         for edge in edges {
//             self.add_edge_wrapper(&edge.0, &edge.1, edge.2);
//         }
//     }

//     pub fn new(
//         params: ProgramParams,
//         heap_dump_raw_file_path_or_nodes: Either<&str, Vec<Node>>,
//         pointer_byte_size_or_edges: Either<usize, Vec<(Node, Node, usize)>>,
//     ) -> GraphData {
//         let mut graph_data = GraphData {
//             params,
//             graph: DiGraph::new(),
//             heap_dump_data: None,
//         };

//         match (heap_dump_raw_file_path_or_nodes, pointer_byte_size_or_edges) {
//             (Either::Left(heap_dump_raw_file_path), Either::Left(pointer_byte_size)) => {
//                 graph_data.file_init(heap_dump_raw_file_path, pointer_byte_size);
//             }
//             (Either::Right(nodes), Either::Right(edges)) => {
//                 graph_data.test_graph_init(nodes, edges);
//             }
//             _ => panic!("Invalid arguments for graph generation"),
//         }

//         graph_data
//     }

//     // Wrapper functions

//     // ...

//     // Identification

//     // ...

//     // Graph data

//     // ...

//     // Graph manipulation

//     // ...

//     // Logic

//     // ...
// }
