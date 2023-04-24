use petgraph::graphmap::DiGraphMap;
use std::path::{PathBuf};
use std::collections::HashMap;
use log;
use petgraph::visit::IntoEdgeReferences;

pub mod heap_dump_data;

use heap_dump_data::HeapDumpData;
use crate::graph_structs::{self, Node, DataStructureNode, Edge, EdgeType, DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT};
use crate::params::{BLOCK_BYTE_SIZE, MALLOC_HEADER_ENDIANNESS, COMPRESS_POINTER_CHAINS};
use crate::utils;

/// macro for getting the heap_dump_data field unwrapped
macro_rules! check_heap_dump {
    ($self:expr) => {{
        assert!(!$self.heap_dump_data.is_none(), "heap_dump_data is None");
    }};
}

/// This struct contains the graph data
/// linked to a given heap dump file.
pub struct GraphData {
    pub graph: DiGraphMap<u64, graph_structs::Edge>,
    pub addr_to_node: HashMap<u64, graph_structs::Node>,
    pub unannotated_value_node_addrs: Vec<u64>, // list of the addresses of the nodes that are values (and potental keys)

    pub heap_dump_data: Option<HeapDumpData>, // Some because it is an optional field, for testing purposes
}


impl GraphData {

    /// Initialize the graph data from a raw heap dump file.
    pub fn new(heap_dump_raw_file_path: PathBuf, pointer_byte_size: usize) -> Result<Self, crate::utils::ErrorKind> {
        let mut instance = Self {
            graph: DiGraphMap::<u64, graph_structs::Edge>::new(),
            addr_to_node: HashMap::new(),
            unannotated_value_node_addrs: Vec::new(),
            heap_dump_data: Some(
                HeapDumpData::new(
                    heap_dump_raw_file_path,
                    pointer_byte_size,
                )?
            ),
        };

        instance.data_structure_step();
        instance.pointer_step();
        Ok(instance)
    }

    /// Constructor for an empty GraphData
    fn new_empty() -> Self {
        Self {
            graph: DiGraphMap::<u64, graph_structs::Edge>::new(),
            addr_to_node: HashMap::new(),
            unannotated_value_node_addrs: Vec::new(),
            heap_dump_data: None,
        }
    }


    fn create_node_from_bytes_wrapper(
        &self, data: &[u8; BLOCK_BYTE_SIZE], addr: u64, dtn_addr: u64
    ) -> graph_structs::Node {
        check_heap_dump!(self);
        return utils::create_node_from_bytes(
            data,
            addr,
            dtn_addr,
            self.heap_dump_data.as_ref().unwrap().min_addr,
            self.heap_dump_data.as_ref().unwrap().max_addr,
        );
    }
    
    /// Wrapper for create_node_from_bytes_wrapper using a block index instead of an address.
    fn create_node_from_bytes_wrapper_index(
        &self, data: &[u8; BLOCK_BYTE_SIZE], block_index: usize, dtn_addr: u64
    ) -> graph_structs::Node {
        check_heap_dump!(self);
        let addr = self.heap_dump_data.as_ref().unwrap().index_to_addr_wrapper(block_index);
        return self.create_node_from_bytes_wrapper(data, addr, dtn_addr);
    }

    /// add node to the map & to the map
    /// NOTE: the node is moved to the map
    fn add_node_wrapper(&mut self, node: graph_structs::Node) -> u64 {
        // keep addr of all the value nodes
        if node.is_value() {
            self.unannotated_value_node_addrs.push(node.get_address());
        }

        let node_addr = node.get_address();
        self.addr_to_node.insert(node_addr, node); // move the node
        self.graph.add_node(self.addr_to_node.get(&node_addr).unwrap().get_address());
        node_addr
    }

    /// Add an edge to the graph.
    /// NOTE: the edge is moved to the graph
    /// WARN: the nodes must already be in the addr_to_node map
    fn add_edge_wrapper(&mut self, edge: graph_structs::Edge) {
        self.graph.add_edge(
            self.addr_to_node.get(&edge.from).unwrap().get_address(),
            self.addr_to_node.get(&edge.to).unwrap().get_address(), 
            edge
        );
    }

    /// get the malloc header (number of byte allocated + 1)
    fn get_memalloc_header(&self, data: &[u8; BLOCK_BYTE_SIZE]) -> usize {
        utils::block_bytes_to_addr(data, MALLOC_HEADER_ENDIANNESS) as usize
    }

    /////////////////////////////////////////////////////////////
    /// Step 1: data structure step
    
    /// Pass null blocks.
    fn pass_null_blocks(&self, index: usize) -> usize {
        check_heap_dump!(self);
        let mut tmp_index = index;
        while 
            (tmp_index < self.heap_dump_data.as_ref().unwrap().blocks.len()) && // check if index is in bounds
            (self.heap_dump_data.as_ref().unwrap().blocks[tmp_index] == [0u8; BLOCK_BYTE_SIZE])
        {
            tmp_index += 1;
        }
        return tmp_index
    }

    /// Parse all data structures step. Don't follow pointers yet.
    fn data_structure_step(&mut self) {
        check_heap_dump!(self);
        
        // generate data structures and iterate over them
        let mut block_index = 0;
        while block_index < self.heap_dump_data.as_ref().unwrap().blocks.len() {
            block_index = self.pass_null_blocks(block_index);

            // get the data structure
            let data_structure_block_size = self.parse_datastructure(block_index);

            // update the block index by leaping over the data structure
            block_index += data_structure_block_size + 1;
        }

    }

    /// Parse the data structure from a given block and populate the graph.
    /// WARN: We don't follow the pointers in the data structure. This is done in a second step.
    /// :return: The number of blocks in the data structure.
    /// If the data structure is not valid, return 0, since there no data structure to leap over.
    fn parse_datastructure(&mut self, start_block_index: usize) -> usize {
        check_heap_dump!(self);

        // precondition: the block at startBlockIndex is not the last block of the heap dump or after
        if start_block_index >= (self.heap_dump_data.as_ref().unwrap().blocks.len() - 1) {
            return 0; // this is not a data structure, no need to leap over it
        }

        // get the size of the data structure from malloc header
        // NOTE: the size given by malloc header is the size of the data structure + 1
        let datastructure_size = self.get_memalloc_header(&self.heap_dump_data.as_ref().unwrap().blocks[start_block_index]) - 1;

        // check if nb_blocks_in_datastructure is an integer
        let tmp_nb_blocks_in_datastructure = datastructure_size / BLOCK_BYTE_SIZE;
        if tmp_nb_blocks_in_datastructure % 1 != 0 {
            log::debug!("tmp_nb_blocks_in_datastructure: {}", tmp_nb_blocks_in_datastructure);
            log::debug!("The data structure size is not a multiple of the block size, at block index: {}", start_block_index);
            return 0 // this is not a data structure, no need to leap over it
        }

        // get the number of blocks in the data structure as an integer
        let nb_blocks_in_datastructure = tmp_nb_blocks_in_datastructure;

        // check if the data structure is complete, i.e. if the data structure is still unclosed after at the end of the heap dump
        if (start_block_index + nb_blocks_in_datastructure) >= self.heap_dump_data.as_ref().unwrap().blocks.len() {
            log::debug!("The data structure is not complete, at block index: {}", start_block_index);
            return 0
        }
    
        // check that the data structure is not empty, i.e. that it contains at least one block
        // It cannot also be composed of only one block, since the first block is the malloc header,
        // and a data structure cannot be only the malloc header.
        if nb_blocks_in_datastructure < 2 {
            log::debug!(
                "The data structure is too small ({} blocks), at block index: {}",
                    nb_blocks_in_datastructure, start_block_index
                );
            return 0
        }
        
        // add the data structure node to the graph (as an address)
        let current_datastructure_addr = self.heap_dump_data.as_ref().unwrap().index_to_addr_wrapper(start_block_index);

        let mut count_pointer_nodes = 0;
        let mut count_value_nodes = 0;
        let mut children_node_addrs: Vec<u64> = Vec::new();
        for block_index in (start_block_index + 1)..(start_block_index  + nb_blocks_in_datastructure as usize) {
            let node = self.create_node_from_bytes_wrapper_index(
                &self.heap_dump_data.as_ref().unwrap().blocks[block_index], 
                block_index,
                current_datastructure_addr
            );
            children_node_addrs.push(node.get_address());
            
            // stats
            if node.is_pointer() {
                count_pointer_nodes += 1;
            } else {
                count_value_nodes += 1;
            }

            self.add_node_wrapper(node); // WARN: move the node to the graph, do last
        }
                
        
        // create the data structure node with the correct number of pointer and value nodes
        let datastructure_node = Node::DataStructureNode(DataStructureNode {
            addr: current_datastructure_addr,
            byte_size: datastructure_size,
            nb_pointer_nodes: count_pointer_nodes,
            nb_value_nodes: count_value_nodes,
        });
        self.add_node_wrapper(datastructure_node);
        
        // add all the edges to the graph
        for child_node_addr in children_node_addrs {
            self.add_edge_wrapper(Edge {
                from: self.addr_to_node.get(&current_datastructure_addr).unwrap().get_address(),
                to: child_node_addr,
                weight: DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT,
                edge_type: EdgeType::DataStructureEdge,
            });
        }
        
        return nb_blocks_in_datastructure
    }

    /// Parse a pointer node. Follow it until it point to a node that is not a pointer, and add the edge 
    /// weightened by the number of intermediate pointer nodes.
    /// TODO: testing needed, correction needed
    fn parse_pointer(&mut self, node_addr: u64) {
        let node = self.addr_to_node.get(&node_addr).unwrap();

        // check if the pointer points to a node in the graph
        let mut current_pointer_node: &Node = node;
        let mut weight = 1;
        let mut pointed_node: Option<&Node> = self.addr_to_node.get(&current_pointer_node.points_to().unwrap());

        // only for pointer to pointer regrouping
        if *COMPRESS_POINTER_CHAINS {
            while current_pointer_node.is_pointer() && pointed_node.is_some() && pointed_node.unwrap().is_pointer() {
            
                weight += 1;
    
                // next iteration
                current_pointer_node = pointed_node.unwrap();
                
                pointed_node = self.addr_to_node.get(&current_pointer_node.points_to().unwrap());
            }
        }

        // add the edge if the node is valid
        if pointed_node.is_some() {
            self.add_edge_wrapper(Edge {
                from: node.get_address(),
                to: pointed_node.unwrap().get_address(),
                weight: weight,
                edge_type: EdgeType::PointerEdge,
            });
        }
    }

    /// Parse all pointers step.
    fn pointer_step(&mut self) {
        // get all pointer nodes
        let all_pointer_addr: Vec<u64> = self.get_all_ptr_node_addrs();

        for pointer_addr in all_pointer_addr {
            self.parse_pointer(pointer_addr);
        }
    }

    fn get_all_ptr_node_addrs(&self) -> Vec<u64> {
        let mut all_addr: Vec<u64> = Vec::new();
        for (addr, node) in self.addr_to_node.iter() {
            match node {
                Node::PointerNode(_) => {
                    all_addr.push(*addr);
                },
                _ => {}
            }
        }
        return all_addr
    }

}

/// custom dot format for the graph
impl std::fmt::Display for GraphData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "digraph {{")?;
        // TODO match node and call its associated display function
        for addr in self.graph.nodes() {
            let node = self.addr_to_node.get(&addr).unwrap();
            // call the display function of the node
            writeln!(f, "{}", node)?;
        }
        
        // since edge doesn't stores references but real addresses,
        // we cannot write the Display function of Edge directly
        // we need to do it here
        for (from_addr, to_addr, edge) in self.graph.edge_references() {
            let from = self.addr_to_node.get(&from_addr).unwrap();
            let to = self.addr_to_node.get(&to_addr).unwrap();
            writeln!(f, "    \"{}\" -> \"{}\" [label=\"{}\" weight={}]", from.str_addr_and_type(), to.str_addr_and_type(), edge.edge_type, edge.weight)?;
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
    use crate::params::{self};
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
                    dtn_addr: 1,
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
            from: data_structure_node_index,
            to: base_value_node_index,
            weight: DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT,
            edge_type: EdgeType::DataStructureEdge,
        };
        let pointer_edge = Edge {
            from: base_pointer_node_index,
            to: base_value_node_index,
            weight: 1,
            edge_type: EdgeType::PointerEdge,
        };
        let data_structure_edge_2 = Edge {
            from: data_structure_node_index,
            to: base_pointer_node_index,
            weight: DEFAULT_DATA_STRUCTURE_EDGE_WEIGHT,
            edge_type: EdgeType::DataStructureEdge,
        };

        // add edges (u64 to u64, with Edge as data (weight)))
        graph_data.graph.add_edge(
            data_structure_edge_1.from, 
            data_structure_edge_1.to, 
            data_structure_edge_1
        );
        graph_data.graph.add_edge(
            pointer_edge.from, 
            pointer_edge.to, 
            pointer_edge
        );
        graph_data.graph.add_edge(
            data_structure_edge_2.from, 
            data_structure_edge_2.to, 
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
        
        let graph_data = GraphData::new(
            params::TEST_HEAP_DUMP_FILE_PATH.clone(), 
            params::BLOCK_BYTE_SIZE
        ).unwrap();
        check_heap_dump!(graph_data);

        // pointer node
        let pointer_node_1 = create_node_from_bytes(
            &*TEST_PTR_1_VALUE_BYTES, 
            *TEST_PTR_1_ADDR, 
            *TEST_MALLOC_HEADER_1_ADDR,
            graph_data.heap_dump_data.as_ref().unwrap().min_addr, 
            graph_data.heap_dump_data.as_ref().unwrap().max_addr,
        );

        let pointer_node_1_from_wrapper = graph_data.create_node_from_bytes_wrapper(
            &*TEST_PTR_1_VALUE_BYTES, *TEST_PTR_1_ADDR, *TEST_MALLOC_HEADER_1_ADDR
        );

        assert_eq!(pointer_node_1.get_address(), *TEST_PTR_1_ADDR);
        assert_eq!(pointer_node_1, pointer_node_1_from_wrapper);

        // value node
        let value_node_1 = create_node_from_bytes(
            &*TEST_VAL_1_VALUE_BYTES, 
            *TEST_VAL_1_ADDR, 
            *TEST_MALLOC_HEADER_1_ADDR,
            graph_data.heap_dump_data.as_ref().unwrap().min_addr, 
            graph_data.heap_dump_data.as_ref().unwrap().max_addr,
        );
        let value_node_1_from_wrapper = graph_data.create_node_from_bytes_wrapper(
            &*TEST_VAL_1_VALUE_BYTES, *TEST_VAL_1_ADDR, *TEST_MALLOC_HEADER_1_ADDR
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
        ).unwrap();
        let node = graph_data.create_node_from_bytes_wrapper_index(
            &*TEST_PTR_1_VALUE_BYTES, 
            (utils::hex_str_to_addr("00000300", utils::Endianness::Big).unwrap() / BLOCK_BYTE_SIZE as u64) as usize,
            *TEST_MALLOC_HEADER_1_ADDR
        );
        assert_eq!(node.get_address(), *TEST_PTR_1_ADDR);
    }

    #[test]
    fn test_graph_from_test_file() {
        crate::tests::setup();

        use std::path::Path;
        use std::fs::File;
        use std::io::Write;
        
        let graph_data = GraphData::new(
            params::TEST_HEAP_DUMP_FILE_PATH.clone(), 
            params::BLOCK_BYTE_SIZE
        ).unwrap();
        check_heap_dump!(graph_data);

        // save the graph to a file as a dot file (graphviz)
        let dot_file_name: String = format!("{}test_graph_from_{}.gv", &*TEST_GRAPH_DOT_DIR_PATH, &*TEST_HEAP_DUMP_FILE_NUMBER);
        let dot_file_path = Path::new(dot_file_name.as_str());
        let mut dot_file = File::create(dot_file_path).unwrap();
        dot_file.write_all(format!("{}", graph_data).as_bytes()).unwrap(); // using the custom formatter

        // check that the value node addresses are kept
        assert!(graph_data.unannotated_value_node_addrs.len() > 0);
    }

}