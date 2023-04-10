use petgraph::graphmap::DiGraphMap;
use std::path::{PathBuf};
use std::collections::HashMap;

pub mod heap_dump_data;

use heap_dump_data::HeapDumpData;
use crate::graph_structs;
use crate::params::BLOCK_BYTE_SIZE;
use crate::utils;

/// macro for getting the heap_dump_data field unwrapped
macro_rules! heap_dump_data_ref {
    ($self:expr) => {{
        assert!(!$self.heap_dump_data.is_none(), "heap_dump_data is None");
        $self.heap_dump_data.as_ref().unwrap()
    }};
}

/// This struct contains the graph data
/// linked to a given heap dump file.
pub struct GraphData<'a> {
    graph: DiGraphMap<u64, graph_structs::Edge<'a>>,
    addr_to_node: HashMap<u64, graph_structs::Node>,

    heap_dump_data: Option<HeapDumpData>, // Some because it is an optional field, for testing purposes
    
}

impl<'a> GraphData<'a> {

    /// Initialize the graph data from a raw heap dump file.
    fn new(heap_dump_raw_file_path: PathBuf, pointer_byte_size: usize) -> Self {
        let mut instance = Self {
            graph: DiGraphMap::<u64, graph_structs::Edge>::new(),
            addr_to_node: HashMap::new(),
            heap_dump_data: Some(
                HeapDumpData::new(
                    heap_dump_raw_file_path,
                    pointer_byte_size,
                )
            ),
        };

        // instance.data_structure_step(pointer_byte_size);
        // instance.pointer_step();
        instance
    }

    /// Constructor for an empty GraphData
    fn new_empty() -> Self {
        Self {
            graph: DiGraphMap::<u64, graph_structs::Edge>::new(),
            addr_to_node: HashMap::new(),
            heap_dump_data: None,
        }
    }


    fn create_node_from_bytes_wrapper(
        &self, data: &[u8; BLOCK_BYTE_SIZE], addr: u64
    ) -> graph_structs::Node {
        let heap_dump_data_ref = heap_dump_data_ref!(self);
        return utils::create_node_from_bytes(
            data,
            addr,
            heap_dump_data_ref.min_addr,
            heap_dump_data_ref.max_addr,
        );
    }
    
    /// Wrapper for create_node_from_bytes_wrapper using a block index instead of an address.
    fn create_node_from_bytes_wrapper_index(
        &self, data: &[u8; BLOCK_BYTE_SIZE], block_index: usize
    ) -> graph_structs::Node {
        let heap_dump_data_ref = heap_dump_data_ref!(self);
        let addr = heap_dump_data_ref.index_to_addr_wrapper(block_index);
        return self.create_node_from_bytes_wrapper(data, addr);
    }

    /// add node to the map & to the map
    /// NOTE: the node is moved to the map
    fn add_node_wrapper(&mut self, node: graph_structs::Node) -> u64 {
        let node_addr = node.get_address();
        self.addr_to_node.insert(node_addr, node); // move the node
        self.graph.add_node(self.addr_to_node.get(&node_addr).unwrap().get_address());
        node_addr
    }

    /// Add an edge to the graph.
    /// NOTE: the edge is moved to the graph
    fn add_edge_wrapper(&mut self, edge: graph_structs::Edge<'a>) {
        self.graph.add_edge(
            edge.from.get_address(), 
            edge.to.get_address(), 
            edge
        );
    }
    

}

/// custom dot format for the graph
impl std::fmt::Display for GraphData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "digraph {{")?;
        // TODO match node and call its associated display function
        for addr in self.graph.nodes() {
            let node = self.addr_to_node.get(&addr).unwrap();
            // call the display function of the node
            write!(f, "{}", node)?;
        }
        for (_, _, edge) in self.graph.all_edges() {
            writeln!(f, "{}", edge)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}


// NOTE: tests must be in the same module as the code they are testing
// for them to have access to the private functions
#[cfg(test)]
mod tests {
    use log;
    use petgraph::dot::Dot;

    use super::*;
    use crate::params::{self, TEST_HEAP_DUMP_FILE_PATH, PTR_ENDIANNESS};
    use crate::graph_structs::{
        Node, 
        ValueNode, 
        PointerNode, 
        BaseValueNode, 
        BasePointerNode, 
        DataStructureNode,
        Edge,
        EdgeType,
        DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT,
    };
    use crate::tests::*;
    use crate::utils::create_node_from_bytes;

    #[test]
    fn test_petgraph_digraphmap() {
        crate::tests::setup();
        
        // create empty GraphData
        let mut graph_data = GraphData::new_empty();

        // create test nodes
        let data_structure_node = Node::DataStructureNode(DataStructureNode {
            addr: 1,
            byte_size: 8,
            nb_pointer_nodes: 0,
            nb_value_nodes: 0,
        });
        let base_value_node = Node::ValueNode(
            ValueNode::BaseValueNode(
                BaseValueNode {
                    addr: 2,
                    value: [0, 1, 2, 3, 4, 5, 6, 7],
                }
            )
        );
        let base_pointer_node = Node::PointerNode(
                PointerNode::BasePointerNode(BasePointerNode {
                    addr: 3,
                    points_to: 8,
                }
            )
        );

        // add nodes as addresses
        let data_structure_node_index = graph_data.graph.add_node(
            data_structure_node.get_address()
        );
        let base_value_node_index = graph_data.graph.add_node(
            base_value_node.get_address()
        );
        let base_pointer_node_index = graph_data.graph.add_node(
            base_pointer_node.get_address()
        );

        assert_eq!(graph_data.graph.node_count(), 3);
        assert_eq!(data_structure_node_index, data_structure_node.get_address());
        assert_eq!(base_value_node_index, base_value_node.get_address());
        assert_eq!(base_pointer_node_index, base_pointer_node.get_address());

        // add nodes to dictionary
        graph_data.addr_to_node.insert(
            data_structure_node.get_address(),
            data_structure_node // move
        );
        graph_data.addr_to_node.insert(
            base_value_node.get_address(),
            base_value_node // move
        );
        graph_data.addr_to_node.insert(
            base_pointer_node.get_address(),
            base_pointer_node // move
        );

        // create test edges
        // WARN: the references to the nodes have been moved inside the dictionary
        //     so we need to get them back from the dictionary (using the address as key)
        let data_structure_edge_1 = Edge {
            from: &graph_data.addr_to_node.get(&data_structure_node_index).unwrap(),
            to: &graph_data.addr_to_node.get(&base_value_node_index).unwrap(),
            weight: DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT,
            edge_type: EdgeType::DataStructureEdge,
        };
        let pointer_edge = Edge {
            from: &graph_data.addr_to_node.get(&base_pointer_node_index).unwrap(),
            to: &graph_data.addr_to_node.get(&base_value_node_index).unwrap(),
            weight: 1,
            edge_type: EdgeType::PointerEdge,
        };
        let data_structure_edge_2 = Edge {
            from: &graph_data.addr_to_node.get(&data_structure_node_index).unwrap(),
            to: &graph_data.addr_to_node.get(&base_pointer_node_index).unwrap(),
            weight: DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT,
            edge_type: EdgeType::DataStructureEdge,
        };

        // add edges (u64 to u64, with Edge as data (weight)))
        graph_data.graph.add_edge(
            data_structure_edge_1.from.get_address(), 
            data_structure_edge_1.to.get_address(), 
            data_structure_edge_1
        );
        graph_data.graph.add_edge(
            pointer_edge.from.get_address(), 
            pointer_edge.to.get_address(), 
            pointer_edge
        );
        graph_data.graph.add_edge(
            data_structure_edge_2.from.get_address(), 
            data_structure_edge_2.to.get_address(), 
            data_structure_edge_2
        );

        // print the type of all nodes in the map
        for (addr, node) in &graph_data.addr_to_node {
            log::info!("node at address {} is of type {:?}", addr, node);
        }

        // display graph
        log::info!("first version of test graph: \n{}", Dot::new(&graph_data.graph));
        log::info!("custom formatter: \n{}", graph_data);
    }

    #[test]
    fn test_create_node_from_bytes_wrapper() {
        crate::tests::setup();
        
        let mut graph_data = GraphData::new(
            params::TEST_HEAP_DUMP_FILE_PATH.clone(), 
            params::BLOCK_BYTE_SIZE
        );
        let heap_dump_data_ref = heap_dump_data_ref!(graph_data);

        // pointer node
        let pointer_node_1 = create_node_from_bytes(
            &*TEST_PTR_1_VALUE_BYTES, 
            *TEST_PTR_1_ADDR, 
            heap_dump_data_ref.min_addr, 
            heap_dump_data_ref.max_addr,
        );

        let pointer_node_1_from_wrapper = graph_data.create_node_from_bytes_wrapper(
            &*TEST_PTR_1_VALUE_BYTES, *TEST_PTR_1_ADDR
        );

        assert_eq!(pointer_node_1.get_address(), *TEST_PTR_1_ADDR);
        assert_eq!(pointer_node_1, pointer_node_1_from_wrapper);

        // value node
        let value_node_1 = create_node_from_bytes(
            &*TEST_VAL_1_VALUE_BYTES, 
            *TEST_VAL_1_ADDR, 
            heap_dump_data_ref.min_addr, 
            heap_dump_data_ref.max_addr,
        );
        let value_node_1_from_wrapper = graph_data.create_node_from_bytes_wrapper(
            &*TEST_VAL_1_VALUE_BYTES, *TEST_VAL_1_ADDR
        );

        assert_eq!(value_node_1.get_address(), *TEST_VAL_1_ADDR);
        assert_eq!(value_node_1, value_node_1_from_wrapper);
    }

    #[test]
    fn test_create_node_from_bytes_wrapper_index() {
        crate::tests::setup();
        
        let graph_data = GraphData::new(
            params::TEST_HEAP_DUMP_FILE_PATH.clone(), 
            params::BLOCK_BYTE_SIZE
        );
        let node = graph_data.create_node_from_bytes_wrapper_index(
            &*TEST_PTR_1_VALUE_BYTES, 
            (utils::hex_str_to_addr("00000300", utils::Endianness::Big).unwrap() / BLOCK_BYTE_SIZE as u64) as usize,
        );
        assert_eq!(node.get_address(), *TEST_PTR_1_ADDR);
    }

}