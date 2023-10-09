use std::collections::{HashSet, HashMap};

use crate::graph_structs::Node;

use super::GraphEmbedding;

/// get the children/ancestor (given direction) of a node
/// in order : chn_depth_1, ptr_depth_1, chn_depth_2, ptr_depth_2, ... , chn_depth_n, ptr_depth_n
pub fn get_neighbors(
    graph_embedding : &GraphEmbedding, 
    addrs: HashSet<u64>, 
    direction : petgraph::Direction,
) -> HashMap<String, usize> {
    let mut result = HashMap::new();
    // vectorize ancestors
    let mut current_node_addrs: HashSet<u64>;
    let mut ancestor_addrs: HashSet<u64> = addrs;

    for i in 0..graph_embedding.depth {
        // swap current and next ancestors
        current_node_addrs = ancestor_addrs;
        ancestor_addrs = HashSet::new();

        let mut nb_chn = 0;
        let mut nb_ptr = 0;

        for ancestor_addr in current_node_addrs.iter() {
            let node: &Node = graph_embedding.graph_annotate.graph_data.addr_to_node.get(ancestor_addr).unwrap();



            // count current nodes types (if we are at depth = 0, it means we are at the starting node, and must count it, no edge to acknowledge)
            // NOTE : we count at the edge if we are deeper
            if i == 0 {
                match node {
                    Node::ChunkHeaderNode(_) => nb_chn += 1,
                    Node::PointerNode(_) => nb_ptr += 1,
                    _ => (),
                }
            }

            // get the next ancestors
            for neighbor in graph_embedding.graph_annotate.graph_data.graph.edges_directed(
                *ancestor_addr, direction
            ) {
                let edge_weight = neighbor.2;
                let neighbor_addr = neighbor.1;

                // prepare next ancestors
                ancestor_addrs.insert(neighbor_addr);


                // count next nodes types (if we are at depth != 0, it means we are not at the starting node, and must count it with edge weight)
                if i != 0 {
                    match node {
                        Node::ChunkHeaderNode(_) => nb_chn += edge_weight.weight,
                        Node::PointerNode(_) => nb_ptr += edge_weight.weight,
                        _ => (),
                    }
                }
            }
        }
        
        let feature_direction_name = match direction {
            petgraph::Direction::Incoming => "ancestor",
            petgraph::Direction::Outgoing => "children",
        };

        // add number of chns for this depth
        let feature_name_chn = format!(
            "chns_{}_{}", feature_direction_name, i + 1
        );
        result.insert(feature_name_chn, nb_chn);

        // add number of ptrs for this depth
        let feature_name_ptr = format!(
            "ptrs_{}_{}", feature_direction_name, i + 1
        );
        result.insert(feature_name_ptr, nb_ptr);
    }

    result
}


/// Generate the ancestor/children (in given direction) embedding 
/// with respect to a given start chn address.
/// 
/// NOTE: If we have value node, Since nothing points to a CHN, we starts from the children 
/// nodes of the given chunk of the CHN. Else, we start from the CHN itself.
/// 
/// (number of ptn and number of chn for each depth)
pub fn generate_samples_for_neighbor_nodes_of_the_chunk(
    graph_embedding : &GraphEmbedding, 
    chn_addr : u64, 
    direction : petgraph::Direction
) -> HashMap<String, usize> {
    // get the children for starting the algorithm
    let mut ancestor_addrs: HashSet<u64> = HashSet::new();

    if graph_embedding.graph_annotate.graph_data.no_value_node {
        // if we have no value node, we begin from the chn node directly
        ancestor_addrs.insert(chn_addr);
    } else {
        // else we begin from the children of the chn node
        let children = 
        graph_embedding.graph_annotate.graph_data.graph.neighbors_directed(
            chn_addr, petgraph::Direction::Outgoing
        );
        for child_addr in children {
            ancestor_addrs.insert(child_addr);
        }
    }
    
    get_neighbors(graph_embedding, ancestor_addrs, direction)
}