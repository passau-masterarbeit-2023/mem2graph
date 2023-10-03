use std::collections::HashSet;

use crate::graph_structs::Node;

use super::GraphEmbedding;

/// get the children/ancestor (given direction) of a node
/// in order : chn_depth_1, ptr_depth_1, chn_depth_2, ptr_depth_2, ... , chn_depth_n, ptr_depth_n
pub fn get_neighbors(graph_embedding : &GraphEmbedding, addrs: HashSet<u64>, dir : petgraph::Direction) -> Vec<usize> {
    let mut result : Vec<usize> = Vec::new();
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

            // count current nodes types
            match node {
                Node::ChunkHeaderNode(_) => nb_chn += 1,
                Node::PointerNode(_) => nb_ptr += 1,
                _ => (),
            }

            // get the next ancestors
            for neighbor in graph_embedding.graph_annotate.graph_data.graph.neighbors_directed(
                *ancestor_addr, dir
            ) {
                ancestor_addrs.insert(neighbor);
            }
        }
        
        if i > 0 { // skip the first value (always the same case)
            result.push(nb_chn); // add number of chns
            result.push(nb_ptr);  // add number of ptrs
        }
    }

    result
}


/// Generate the ancestor/children (in given direction) embedding 
/// with respect to a given start chn address.
/// 
/// NOTE: Since nothing points to a CHN, we starts from the children 
/// nodes of the given chunk of the CHN.
/// 
/// (number of ptn and number of chn for each depth)
pub fn generate_samples_for_neighbor_nodes_of_the_chunk(
    graph_embedding : &GraphEmbedding, 
    chn_addr : u64, 
    dir : petgraph::Direction
) -> Vec<usize> {
    // get the children for starting the algorithm
    let mut ancestor_addrs: HashSet<u64> = HashSet::new();
    let children = 
        graph_embedding.graph_annotate.graph_data.graph.neighbors_directed(
            chn_addr, petgraph::Direction::Outgoing
        );
    for child_addr in children {
        ancestor_addrs.insert(child_addr);
    }
    get_neighbors(graph_embedding, ancestor_addrs, dir)
}