use petgraph::graphmap::DiGraphMap;
use std::path::PathBuf;
use std::collections::HashMap;
use log;
use petgraph::visit::IntoEdgeReferences;

pub mod heap_dump_data;

use heap_dump_data::HeapDumpData;
use crate::graph_structs::{self, Node, ChunkHeaderNode, Edge, EdgeType, DEFAULT_CHUNK_EDGE_WEIGHT, parse_chunk_header, HeaderFlags, FooterNode};
use crate::graph_structs::annotations::AnnotationSet;
use crate::params::BLOCK_BYTE_SIZE;
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
    /// list of all the addresses of the nodes that are CHNs
    pub chn_addrs: Vec<u64>,
    /// list of the addresses of the nodes that are values (and potential keys)
    pub value_node_addrs: Vec<u64>, 
    /// list of the addresses of the nodes that are pointers
    pub pointer_node_addrs: Vec<u64>,

    /// map from node address to annotations
    pub node_addr_to_annotations: HashMap<u64, AnnotationSet>,

    /// if the graph doesn't contain value nodes
    pub no_value_node: bool,

    pub heap_dump_data: Option<HeapDumpData>, // Some because it is an optional field, for testing purposes
}


impl GraphData {

    /// Initialize the graph data from a raw heap dump file.
    /// NOTE : If the annotation is 'None', we will not load the key in the json
    pub fn new(
        heap_dump_raw_file_path: PathBuf, 
        pointer_byte_size: usize,
        annotation : bool,
        without_pointer_node : bool
    ) -> Result<Self, crate::utils::ErrorKind> {
        let mut instance = Self {
            graph: DiGraphMap::<u64, graph_structs::Edge>::new(),
            addr_to_node: HashMap::new(),
            chn_addrs: Vec::new(),
            value_node_addrs: Vec::new(),
            pointer_node_addrs: Vec::new(),
            node_addr_to_annotations: HashMap::new(),
            no_value_node: without_pointer_node,
            heap_dump_data: Some(
                HeapDumpData::new(
                    heap_dump_raw_file_path,
                    pointer_byte_size,
                    annotation,
                )?
            ),
        };

        instance.chunk_step();
        instance.pointer_step();
        Ok(instance)
    }

    /// Constructor for an empty GraphData
    #[cfg(test)]
    fn new_empty() -> Self {
        Self {
            graph: DiGraphMap::<u64, graph_structs::Edge>::new(),
            addr_to_node: HashMap::new(),
            chn_addrs: Vec::new(),
            value_node_addrs: Vec::new(),
            pointer_node_addrs: Vec::new(),
            node_addr_to_annotations: HashMap::new(),
            no_value_node: false,
            heap_dump_data: None,
        }
    }

    /// Parse last block of a chunk
    /// If parsing fails, return a ValueNode instead of a FooterNode
    fn create_footer_node(
        &self, 
        addr: u64, 
        chunk_size: &usize,
        chunk_flags: &HeaderFlags,
        parent_chn_addr: u64, 
        block: &[u8; BLOCK_BYTE_SIZE]
    ) -> graph_structs::Node {
        let (potential_size, potential_flags) = parse_chunk_header(block);

        // check if the footer has the same size and flags as the header
        if (*chunk_size != potential_size) || (*chunk_flags != potential_flags) {
            // log::debug!("The header and footer of the chunk at address {} don't have the same size or flags", parent_chn_addr);
            // return a value node instead of a footer node when the header and footer don't match
            let value_node = self.create_node_from_bytes_wrapper(
                block, 
                addr, 
                parent_chn_addr
            );
            return value_node;
        }

        let footer_node = Node::FooterNode(FooterNode {
            addr: addr,
            byte_size: potential_size,
            flags: potential_flags,
            chn_addr: parent_chn_addr,
        });
        return footer_node;
    }

    fn create_node_from_bytes_wrapper(
        &self, data: &[u8; BLOCK_BYTE_SIZE], addr: u64, parent_chn_addr: u64
    ) -> graph_structs::Node {
        check_heap_dump!(self);
        return utils::create_node_from_bytes(
            data,
            addr,
            parent_chn_addr,
            self.heap_dump_data.as_ref().unwrap().min_addr,
            self.heap_dump_data.as_ref().unwrap().max_addr,
        );
    }
    
    /// Wrapper for create_node_from_bytes_wrapper using a block index instead of an address.
    fn create_node_from_bytes_wrapper_index(
        &self, data: &[u8; BLOCK_BYTE_SIZE], block_index: usize, parent_chn_addr: u64
    ) -> graph_structs::Node {
        check_heap_dump!(self);
        let addr = self.heap_dump_data.as_ref().unwrap().index_to_addr_wrapper(block_index);
        return self.create_node_from_bytes_wrapper(data, addr, parent_chn_addr);
    }

    /// Add a node the map.
    /// WARN : the node is moved to the map (and not copied)
    fn add_node_to_map_wrapper(&mut self, node: graph_structs::Node) {
        check_heap_dump!(self);
        if node.is_value() {
            self.value_node_addrs.push(node.get_address());
        }else if node.is_chn() {
            self.chn_addrs.push(node.get_address());
        }else if node.is_pointer() {
            self.pointer_node_addrs.push(node.get_address());
        }

        let node_addr = node.get_address();
        self.addr_to_node.insert(node_addr, node); // move the node
    }

    /// add node to the map & to the graph
    /// NOTE: the node is moved to the map
    fn add_node_wrapper(&mut self, node: graph_structs::Node) -> u64 {
        // keep addr of all the value nodes
        let node_addr = node.get_address();
        
        self.add_node_to_map_wrapper(node);
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

    //////////////////////////////////////////////////////////////////////////////
    /// ------------------------- Graph with value nodes -------------------------
    /// Step 1: chunk step
    
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

    /// Parse all chunks step. Don't follow pointers yet.
    fn chunk_step(&mut self) {
        check_heap_dump!(self);
        
        // discover chunks and iterate over them
        let mut block_index = 0;
        let mut chunk_number_in_heap = 0;
        while block_index < self.heap_dump_data.as_ref().unwrap().blocks.len() {
            block_index = self.pass_null_blocks(block_index);

            // get the chunk
            let chunk_size_in_blocks = self.parse_chunk(
                block_index, chunk_number_in_heap
            );

            // In DEBUG mode, print chunk info
            #[cfg(debug_assertions)]
            {
                if block_index + chunk_size_in_blocks >= self.heap_dump_data.as_ref().unwrap().blocks.len() {
                    log::debug!("[block_index:{block_index}] chunk at address {} has {} blocks (incomplete)", self.heap_dump_data.as_ref().unwrap().index_to_addr_wrapper(block_index), chunk_size_in_blocks);
                } else {
                    let chn = self.addr_to_node.get(
                        &self.heap_dump_data.as_ref().unwrap().index_to_addr_wrapper(block_index)
                    ).unwrap();
                    log::debug!(
                        "[block_index:{block_index}][addr:{}][size:{}] chunk has {} blocks", 
                        chn.get_address(), chunk_size_in_blocks * BLOCK_BYTE_SIZE,  chunk_size_in_blocks
                    );
                }
            }

            // update the block index by leaping over the chunk (size includes header, footer and data)
            block_index += chunk_size_in_blocks;
            chunk_number_in_heap += 1;
        }

    }

    /// Parse the chunk from a given block and populate the graph.
    /// WARN: We don't follow the pointers in the chunk step. This is done in a later step.
    /// NOTE: If skip_value_node true, we will not add the value node and the pointer to the graph
    /// 
    /// :return: The size of the chunk, in blocks. This includes the header, footer and data.
    /// 
    /// If the chunk is not valid (for instance, size=0), panic
    fn parse_chunk(&mut self, header_index: usize, chunk_number_in_heap: usize) -> usize {
        check_heap_dump!(self);
        let chunk_data_first_block_index = header_index + 1;

        // precondition: the block at header_addr is not the last block of the heap dump or after
        if header_index >= (self.heap_dump_data.as_ref().unwrap().blocks.len() - 1) {
            panic!("The block at index {} is or is aftre the last block of the heap dump", header_index);
        }

        // get the size of the chunk from malloc header
        // NOTE: The size of the chunk is the size of the data + the size of the header + the size of the footer
        let (chunk_byte_size, header_flags)  = parse_chunk_header(
            &self.heap_dump_data.as_ref().unwrap().blocks[header_index]
        );

        // check if chunk_byte_syze is an integer and block size aligned
        let tmp_chunk_size_in_blocks = chunk_byte_size / BLOCK_BYTE_SIZE;
        if tmp_chunk_size_in_blocks % 1 != 0 {
            log::debug!("tmp_nb_blocks_in_chunk: {}", tmp_chunk_size_in_blocks);
            log::debug!("The chunk size is not a multiple of the block size, at block index: {}", header_index);
            panic!("The chunk size is not a multiple of the block size, at block index: {}", header_index);
        }

        // get the number of blocks in the chunk as an integer
        let chunk_size_in_blocks = tmp_chunk_size_in_blocks;

        // check if the chunk is complete, i.e. if the chunk is still unclosed after at the end of the heap dump
        if (header_index + chunk_size_in_blocks) >= self.heap_dump_data.as_ref().unwrap().blocks.len() {
            log::debug!("The chunk is not complete, at block index: {}", header_index);
            return self.heap_dump_data.as_ref().unwrap().blocks.len() - header_index // leaping over the chunk
        }
    
        // check that the chunk is not empty, i.e. that it contains at least 2 blocks
        if chunk_size_in_blocks < 2 {
            log::debug!(
                "The chunk is too small ({} blocks), at block index: {}",
                    chunk_size_in_blocks, header_index
                );
            panic!(
                "The chunk is too small ({} blocks), at block index: {}",
                    chunk_size_in_blocks, header_index
                );
        }
        
        // add the CHN to the graph (as an address)
        let current_chn_addr = self.heap_dump_data.as_ref().unwrap().index_to_addr_wrapper(header_index);

        let mut count_pointer_nodes = 0;
        let mut count_value_nodes = 0;
        let mut children_node_addrs: Vec<u64> = Vec::new();
        for block_index in (header_index + 1)..(header_index  + chunk_size_in_blocks as usize) {
            // check last block
            let node;
            if block_index == header_index  + chunk_size_in_blocks as usize - 1 {
                // create the footer node
                node = self.create_footer_node(
                    self.heap_dump_data.as_ref().unwrap().index_to_addr_wrapper(block_index),
                    &chunk_byte_size,
                    &header_flags,
                    current_chn_addr,
                    &self.heap_dump_data.as_ref().unwrap().blocks[block_index]
                );
            } else {
                // create the node
                node = self.create_node_from_bytes_wrapper_index(
                    &self.heap_dump_data.as_ref().unwrap().blocks[block_index], 
                    block_index,
                    current_chn_addr
                );
            }
            
            children_node_addrs.push(node.get_address());
            
            // stats
            if node.is_pointer() {
                count_pointer_nodes += 1;
            } else if node.is_value() {
                count_value_nodes += 1;
            }

            // WARN: move the node to the map, do last
            if !self.no_value_node {
                self.add_node_wrapper(node); 
            }else {
                // add the node to the map, but not to the graph
                self.add_node_to_map_wrapper(node);
            }
        }

        // determine if the current chunk is free or in use
        let next_chunk_header_flags = HeaderFlags::parse_chunk_header_flags(
            &self.heap_dump_data.as_ref().unwrap()
                .blocks[header_index + chunk_size_in_blocks as usize + 1]
        );

        // create the CHN with the correct number of pointer and value nodes
        let chn = Node::ChunkHeaderNode(ChunkHeaderNode {
            addr: current_chn_addr,
            byte_size: chunk_byte_size,
            flags: header_flags,
            is_free: next_chunk_header_flags.is_preceding_chunk_free(),
            nb_pointer_nodes: count_pointer_nodes,
            nb_value_nodes: count_value_nodes,
            start_data_bytes_entropy: utils::compute_chunk_start_bytes_entropy(
                &self.heap_dump_data.as_ref().unwrap().blocks, 
                chunk_data_first_block_index
            ),
            chunk_number_in_heap: chunk_number_in_heap,
        });
        self.add_node_wrapper(chn);
        
        // add all the edges to the graph
        if !self.no_value_node {
            for child_node_addr in children_node_addrs {
                self.add_edge_wrapper(Edge {
                    from: self.addr_to_node.get(&current_chn_addr).unwrap().get_address(),
                    to: child_node_addr,
                    weight: DEFAULT_CHUNK_EDGE_WEIGHT,
                    edge_type: EdgeType::ChunkEdge,
                });
            }
        }
        
        return chunk_size_in_blocks
    }

    /// Parse a pointer node. Follow it until it point to a node that is not a pointer, and add the edge 
    /// weightened by the number of intermediate pointer nodes.
    fn parse_pointer(&mut self, node_addr: &u64) {
        let node = self.addr_to_node.get(&node_addr).unwrap();

        // check if the pointer points to a node in the graph
        let weight = 1;
        let pointed_node: Option<&Node> = self.addr_to_node.get(&node.points_to().unwrap());

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
    /// NOTE: this function is called after the chunk step.
    /// NOTE: if without_pointer_node is true, the edge will be beetween 
    ///     the CHNs (representing chunks in the graph) containing 
    ///     the pointer and the pointed node.
    fn pointer_step(&mut self) {
        // get all pointer nodes
        for i in  0..self.pointer_node_addrs.len(){ // borrow checker workaround, don't use iter here
            let pointer_addr = self.pointer_node_addrs[i];
            if self.no_value_node {
                self.parse_pointer_without_value_node(&pointer_addr);
            }else{
                self.parse_pointer(&pointer_addr);
            }
        }
    }

    //////////////////////////////////////////////////////////////////////////////
    // ------------------------- Graph without value nodes -------------------------

    /// Parse the pointers, and link chunks to each other.
    fn parse_pointer_without_value_node(&mut self, node_addr: &u64) {
        let pointer_node = self.addr_to_node.get(&node_addr).unwrap();

        // get the pointer chn addr
        let pointer_chn_addr = pointer_node.get_parent_chn_addr().unwrap();

        // get the pointed node
        let pointed_node_addr = pointer_node.points_to().unwrap();
        let pointed_node = self.addr_to_node.get(&pointed_node_addr);

        // check if the pointed node is in the memory
        if pointed_node.is_none() {
            // the pointed node isn't in the memory, we don't add the edge
            return;
        }

        let pointed_node_parent_chn_addr = pointed_node.unwrap().get_parent_chn_addr();

        let pointed_addr;
        // check if the pointed node parent chn exists
        if pointed_node_parent_chn_addr.is_none() {
            // if it doesn't exist, it means the pointed node is a chn
            // so we use the pointed node address as the pointed chn address
            pointed_addr = pointed_node_addr;
        }else{
            // if it exists, we use the pointed node chn address as the pointed chn address
            pointed_addr = pointed_node_parent_chn_addr.unwrap();
        }

        let previous_edge = self.graph.edge_weight_mut(pointer_chn_addr, pointed_addr);

        // add the edge if it is not already in the graph
        if previous_edge.is_none() {
            self.add_edge_wrapper(Edge {
                from: pointer_chn_addr,
                to: pointed_addr,
                weight: 1,
                edge_type: EdgeType::PointerEdge,
            });
        } else {
            // update the weight of the edge
            let previous_edge = previous_edge.unwrap();
            previous_edge.weight += 1;
        }
    }

    //////////////////////////////////////////////////////////////////////////////
    /// CUSTOM FORMATTER FOR SPECIFIC GRAPH DISPLAYS /////////////////////////////

    /// Generate a string following a dot format.
    /// NOTE: This function can take additional parameters to add comments
    /// to the graphs and individual nodes.
    fn generate_gv_str(
        &self,
        graph_header_comment: Option<String>,
        hashmap: Option<&HashMap<u64, String>>,
    ) -> String {
        let mut dot_gv_str = String::new();

        dot_gv_str.push_str("digraph {\n");
        if graph_header_comment.is_some() {
            dot_gv_str.push_str(
                format!("    comment=\"{}\"\n", graph_header_comment.unwrap())
                    .as_str()
            );
        }

        for addr in self.graph.nodes() {
            let node = self.addr_to_node.get(&addr).unwrap();

            // get the comment
            let node_comment = match hashmap {
                Some(hashmap) => {
                    match hashmap.get(&node.get_address()) {
                        Some(comment) => {
                            format!(" comment=\"{}\"", comment).to_string()
                        },
                        None => "".to_string(),
                    }
                },
                None => "".to_string(),
            };

            // handle special nodes
            match self.node_addr_to_annotations.get(&addr) {
                Some(annotation) => {
                    dot_gv_str.push_str(format!(
                        "    \"{}\" [{}{}]\n", 
                        node.str_addr_and_type(), 
                        annotation.annotate_dot_attributes(),
                        node_comment
                    ).to_string().as_str());
                },
                None => {
                    // anote with default annotation
                    dot_gv_str.push_str(format!(
                        "    \"{}\" [{}{}]\n",
                        node.str_addr_and_type(), 
                        AnnotationSet::get_default_dot_attributes(node),
                        node_comment,
                    ).to_string().as_str());
                }
            }
        }
        
        // since edge doesn't stores references but real addresses,
        // we cannot write the Display function of Edge directly
        // we need to do it here
        for (from_addr, to_addr, edge) in self.graph.edge_references() {
            let from = self.addr_to_node.get(&from_addr).unwrap();
            let to = self.addr_to_node.get(&to_addr).unwrap();
            dot_gv_str.push_str(
                format!(
                            "    \"{}\" -> \"{}\" [label=\"{}({})\" weight={}]\n", from.str_addr_and_type(), to.str_addr_and_type(), edge.edge_type, edge.weight, edge.weight
                ).to_string().as_str()
            );
        }

        dot_gv_str.push_str("}\n");
        dot_gv_str
    }

    pub fn stringify_with_comment_hashmap(
        &self,
        graph_header_comment: String,
        hashmap: &HashMap<u64, String>
    ) -> String {
        self.generate_gv_str(
            Some(graph_header_comment), 
            Some(hashmap)
        )
    }

}

/// custom dot format for the graph
/// NOTE: This formatter is called when saving the graph to a dot file
impl std::fmt::Display for GraphData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.generate_gv_str(None, None))?;
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
        PointerNode, 
        ValueNode, 
        ChunkHeaderNode,
        Edge,
        EdgeType,
        DEFAULT_CHUNK_EDGE_WEIGHT,
    };
    use crate::tests::*;
    use crate::utils::create_node_from_bytes;

    #[test]
    fn test_petgraph_digraphmap() {
        crate::tests::setup();
        
        // create empty GraphData
        let mut graph_data = GraphData::new_empty();

        // create test nodes
        let chn_node = Node::ChunkHeaderNode(ChunkHeaderNode {
            addr: 1,
            byte_size: 8,
            flags: HeaderFlags{p : true, m : false, a : false},
            is_free: false,
            nb_pointer_nodes: 0,
            nb_value_nodes: 0,
            start_data_bytes_entropy: 0.0,
            chunk_number_in_heap: 0,
        });
        let base_value_node = Node::ValueNode(
            ValueNode {
                addr: 2,
                value: [0, 1, 2, 3, 4, 5, 6, 7],
                chn_addr: 1,
            }
        
        );
        let base_pointer_node = Node::PointerNode(PointerNode {
                addr: 3,
                points_to: 8,
                chn_addr: 1,
            }
        
        );

        // add nodes as addresses
        let chn_index = graph_data.graph.add_node(
            chn_node.get_address()
        );
        let base_value_node_index = graph_data.graph.add_node(
            base_value_node.get_address()
        );
        let base_pointer_node_index = graph_data.graph.add_node(
            base_pointer_node.get_address()
        );

        assert_eq!(graph_data.graph.node_count(), 3);
        assert_eq!(chn_index, chn_node.get_address());
        assert_eq!(base_value_node_index, base_value_node.get_address());
        assert_eq!(base_pointer_node_index, base_pointer_node.get_address());

        // add nodes to dictionary
        graph_data.addr_to_node.insert(
            chn_node.get_address(),
            chn_node // move
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
        let chunk_edge_1 = Edge {
            from: chn_index,
            to: base_value_node_index,
            weight: DEFAULT_CHUNK_EDGE_WEIGHT,
            edge_type: EdgeType::ChunkEdge,
        };
        let pointer_edge = Edge {
            from: base_pointer_node_index,
            to: base_value_node_index,
            weight: 1,
            edge_type: EdgeType::PointerEdge,
        };
        let chunk_edge_2 = Edge {
            from: chn_index,
            to: base_pointer_node_index,
            weight: DEFAULT_CHUNK_EDGE_WEIGHT,
            edge_type: EdgeType::ChunkEdge,
        };

        // add edges (u64 to u64, with Edge as data (weight)))
        graph_data.graph.add_edge(
            chunk_edge_1.from, 
            chunk_edge_1.to, 
            chunk_edge_1
        );
        graph_data.graph.add_edge(
            pointer_edge.from, 
            pointer_edge.to, 
            pointer_edge
        );
        graph_data.graph.add_edge(
            chunk_edge_2.from, 
            chunk_edge_2.to, 
            chunk_edge_2
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
            params::BLOCK_BYTE_SIZE,
            true,
            false
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
            params::BLOCK_BYTE_SIZE,
            true,
            false
        ).unwrap();
        let node = graph_data.create_node_from_bytes_wrapper_index(
            &*TEST_PTR_1_VALUE_BYTES, 
            ((*TEST_PTR_1_ADDR - *TEST_HEAP_START_ADDR) / BLOCK_BYTE_SIZE as u64) as usize,
            *TEST_MALLOC_HEADER_1_ADDR
        );
        assert_eq!(node.get_address(), *TEST_PTR_1_ADDR);
    }
}