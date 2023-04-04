use env_logger::fmt;
use petgraph::graphmap::DiGraphMap;
use petgraph::stable_graph::DefaultIx;
use std::path::{PathBuf};
use std::collections::HashMap;

pub mod heap_dump_data;

use heap_dump_data::HeapDumpData;
use crate::graph_structs;
use crate::params::BLOCK_BYTE_SIZE;
use crate::utils::*;

/// This struct contains the graph data
/// linked to a given heap dump file.
pub struct GraphData<'a> {
    graph: DiGraphMap<u64, graph_structs::Edge<'a>>,
    addr_to_node: HashMap<u64, graph_structs::Node>,

    heap_dump_data: Option<HeapDumpData>, // Some because it is an optional field, for testing purposes
}

impl<'a> GraphData<'a> {

    // /// Initialize the graph data from a raw heap dump file.
    // fn new(&self, heap_dump_raw_file_path: PathBuf, pointer_byte_size: usize) -> GraphData {
    //     // Get the heap dump data
    //     self.heap_dump_data = Some(
    //         HeapDumpData::new(
    //             heap_dump_raw_file_path,
    //             pointer_byte_size,
    //         )
    //     );

    //     self.data_structure_step(pointer_byte_size);
    //     self.pointer_step();
    //     return *self;
    // }

    /// Constructor for an empty GraphData
    fn new_empty() -> Self {
        Self {
            graph: DiGraphMap::<u64, graph_structs::Edge>::new(),
            addr_to_node: HashMap::new(),
            heap_dump_data: None,
        }
    }



    // /// constructor for testing purposes
    // fn new_test(&self, nodes: Vec<graph_structs::Node>, edges: Vec<graph_structs::Edge>) -> GraphData {
    //     self.heap_dump_data = None;

    //     self.graph = DiGraphMap::new();
    //     for node in nodes {
    //         self.add_node_wrapper(&node);
    //     }
    //     for edge in edges {
    //         self.add_edge_wrapper(&edge);
    //     }
    //     return *self;
    // }

    // fn create_node_from_bytes_wrapper(
    //     &self, data: &[u8; BLOCK_BYTE_SIZE], addr: u64
    // ) -> graph_structs::Node {
    //     if self.heap_dump_data.is_none() {
    //         panic!("heap_dump_data is None");
    //     }
    //     return create_node_from_bytes(
    //         data,
    //         addr,
    //         self.heap_dump_data.unwrap().min_addr,
    //         self.heap_dump_data.unwrap().max_addr
    //     );
    // }
    
    // /// Wrapper for create_node_from_bytes_wrapper using a block index instead of an address.
    // fn create_node_from_bytes_wrapper_index(
    //     &self, data: &[u8; BLOCK_BYTE_SIZE], block_index: usize
    // ) -> graph_structs::Node {
    //     let addr = self.heap_dump_data.unwrap().index_to_addr_wrapper(block_index);
    //     return self.create_node_from_bytes_wrapper(data, addr);
    // }

    // // def add_node_wrapper(self, node: Node):
    // //     """
    // //     Wrapper for add_node. Add a node with its color to the graph.
    // //     """
    // //     if isinstance(node, Filled):
    // //         self.graph.add_node(node.addr, node=node, style="filled", color=node.color)
    // //     else:
    // //         self.graph.add_node(node.addr, node=node, color=node.color)

    // fn add_node_wrapper(&self, node: &graph_structs::Node) -> u64 {
    //     self.graph.add_node(node.get_address())
    //     // TODO: add node to the map
    // }

    // // def add_edge_wrapper(self, node_start: Node, node_end: Node, weight: int = 1):
    // //     """
    // //     Wrapper for add_edge. Add an edge to the graph.
    // //     """
    // //     # get the type of the edge
    // //     edge_type: Edge
    // //     if isinstance(node_start, PointerNode):
    // //         edge_type = Edge.POINTER
    // //     elif isinstance(node_start, DataStructureNode):
    // //         edge_type = Edge.DATA_STRUCTURE
    // //     else:
    // //         raise ValueError("Unknown node type: %s" % node_start)

    // fn add_edge_wrapper(&self, edge: &graph_structs::Edge) {
    //     self.graph.add_edge(edge.from, edge.to, *edge);
    //     // TODO: add node to the map
    // }

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
    use petgraph::dot::Dot;

    use super::*;
    use crate::params::{
        BLOCK_BYTE_SIZE, 
        TEST_HEAP_DUMP_FILE_PATH
    };
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


    // #[test]
    // fn test_graphdata_display() {
    //     crate::tests::setup();

    //     use std::collections::HashMap;

    //     let mut custom_labels = HashMap::new();
    //     custom_labels.insert(1, "DataStructureNode");
    //     custom_labels.insert(2, "BaseValueNode");
    //     custom_labels.insert(3, "BasePointerNode");



    // }

}