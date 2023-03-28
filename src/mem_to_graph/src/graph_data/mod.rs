use petgraph::graph::{DiGraph, Node, Edge};

pub mod heap_dump_data;

use super::graph_data::heap_dump_data::HeapDumpData;
use crate::graph_structs;
use crate::utils::*;

pub struct GraphData {
    graph: DiGraph<graph_structs::Node, graph_structs::Edge>,
    heap_dump_data: Option<HeapDumpData>,
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
