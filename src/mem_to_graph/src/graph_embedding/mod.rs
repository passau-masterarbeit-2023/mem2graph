#[cfg(test)]
use crate::exe_pipeline::value_embedding::save_value_embeding;
use crate::graph_structs::{
    Node,
};
use crate::graph_annotate::GraphAnnotate;

use std::path::PathBuf;
use std::collections::HashSet;

pub struct GraphEmbedding {
    graph_annotate: GraphAnnotate,
    depth: usize,
}

impl GraphEmbedding {
    pub fn new(
        heap_dump_raw_file_path: PathBuf, 
        pointer_byte_size: usize,
        depth: usize,
        annotation : bool
    ) -> Result<GraphEmbedding, crate::utils::ErrorKind> {
        let graph_annotate = GraphAnnotate::new(heap_dump_raw_file_path, pointer_byte_size, annotation)?;
        
        Ok(GraphEmbedding {
            graph_annotate,
            depth,
        })
    }

    #[cfg(test)]
    fn save_samples_and_labels_to_csv(&self, csv_path: PathBuf) {
        let (samples, labels) = self.generate_value_samples_and_labels();
        save_value_embeding(samples, labels, csv_path, self.depth);
    }

    // ----------------------------- extracting dts -----------------------------//
    /// extract the data of all the dts from the graph
    /// the couple (dts_base_info, dts_data) is returned for each dts
    pub fn extract_all_dts_data(&self, block_size : usize, no_pointer : bool) -> (Vec<Vec<usize>>, Vec<Vec<String>>) {
        let mut dts_base_info = Vec::new();
        let mut dts_data = Vec::new();

        for dtn_addr in self.graph_annotate.graph_data.dtn_addrs.iter() {
            let base_info = self.get_dts_basics_informations(*dtn_addr);
            let data = self.extract_dts_data(*dtn_addr, block_size, no_pointer);

            dts_base_info.push(base_info);
            dts_data.push(data);
        }

        (dts_base_info, dts_data)
    }
    
    /// extract the data of the dts :
    /// get the blocks block_size bytes by block_size bytes (get empty string if the block is a pointer, else get the hexa value)*
    fn extract_dts_data(&self, addr: u64, block_size : usize, no_pointer : bool) -> Vec<String> {
        let mut data = Vec::new();
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

        match node {
            Node::DataStructureNode(data_structure_node) => {
                let mut current_addr = data_structure_node.addr + block_size as u64;

                // get the data of the dts
                for _ in 1..(data_structure_node.byte_size/8) {
                    // get the node at the current address
                    let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&current_addr).unwrap();
                    let mut current_block_str = String::new();

                    // if the block is a pointer
                    if node.is_pointer() {
                        if no_pointer {
                            data.push(String::new());
                        } else {
                            data.push(format!("p:{:x}", node.points_to().unwrap()));
                        }
                    } else {
                        let current_block = node.get_value().unwrap();
                        if !no_pointer {
                            current_block_str.push_str("v:");
                        }
                        // convert the block to hexa
                        for i in 0..block_size {
                            current_block_str.push_str(&format!("{:02x}", current_block[i]));
                        }
                        // add the block to the data
                        data.push(current_block_str);
                    }
                    
                    current_addr += block_size as u64;
                }
            },
            _ => // if the node is not in a data structure, we panic
                panic!("Node is not a DTN"),
        }
        data
    }

    // ----------------------------- DTN embedding -----------------------------//
    /// generate semantic embedding of all the DTN
    /// in order :
    ///     - DTN addresse (not really usefull for learning, but can bu usefull to further analyse the data)
    ///     - DTN size
    ///     - nb pointer
    /// 
    ///     - ancestor (in order of depth, alternate DTN/PTR)
    ///     - children (same)
    ///     - label (if the struct contains a key, or is the ssh or sessionState)
    pub fn generate_semantic_dtns_samples(&self) -> Vec<Vec<usize>> {
        let mut samples = Vec::new();
        // get dtn :
        for dtn_addr in self.graph_annotate.graph_data.dtn_addrs.iter() {
            let sample = self.generate_semantic_dtn_samples(*dtn_addr);
            samples.push(sample);
        }
        samples
    }


    /// generate semantic embedding of a DTN
    fn generate_semantic_dtn_samples(&self, addr: u64) -> Vec<usize> {
        let mut feature: Vec<usize> = Vec::new();

        let mut info = self.get_dts_basics_informations(addr);
        feature.append(&mut info);

        // add ancestors
        let mut ancestors = self.generate_neighbors_dtn(addr, petgraph::Direction::Incoming);
        feature.append(&mut ancestors);

        // add children
        let mut children = self.generate_neighbors_dtn(addr, petgraph::Direction::Outgoing);
        feature.append(&mut children);

        // add label
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
        match node {
            Node::DataStructureNode(data_structure_node) => {
                let label = data_structure_node.dtn_type.clone();
                feature.push(label as usize);
            },
            _ => // if the node is not in a data structure, we panic
                panic!("Node is not a DTN"),
        }

        feature
    }

    /// extract the basics information of the dts
    fn get_dts_basics_informations(&self, addr: u64) -> Vec<usize> {
        let mut info = Vec::new();
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

        // add features from parent dtn node
        match node {
            Node::DataStructureNode(data_structure_node) => {
                info.push(data_structure_node.addr.try_into().expect("addr overflow in embedding")); // WARN : can be overflow !!!!!!
                info.push(data_structure_node.byte_size);
                info.push(data_structure_node.nb_pointer_nodes);
            },
            _ => // if the node is not in a data structure, we panic
                panic!("Node is not a DTN"),
        }
        info
    }

    /// generate the ancestor/children (given dir) embedding of the DTN (nb of ptn and nb of dtn for each deapth)
    fn generate_neighbors_dtn(&self, dtn_addr : u64, dir : petgraph::Direction) -> Vec<usize> {
        // calculate the ancestor for every node in the children of the dtn
        let mut ancestors_by_node : Vec<Vec<usize>> = Vec::new();
        let children = self.graph_annotate.graph_data.graph.neighbors_directed(dtn_addr, petgraph::Direction::Outgoing);
        for child_addr in children {
            ancestors_by_node.push(self.get_neighbors(child_addr, dir));
        }

        // add each ancestor for every vector
        let mut ancestors : Vec<usize> = Vec::new();
        // for each case in the ancestor vector
        for ancestor_i in 0..ancestors_by_node[0].len() {
            let mut nb = 0;
            // for each node
            for ancestors_by_node_i in 0..ancestors_by_node.len() {
                nb += ancestors_by_node[ancestors_by_node_i][ancestor_i];
            }
            ancestors.push(nb);
        }


        ancestors
    }

    // ----------------------------- value embedding -----------------------------//

    /// generate semantic embedding of the value nodes
    /// Samples [
    ///     [0.3233, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     ... 
    /// ]
    /// 
    /// Labels [0.0, 1.0, ..., 0.0],
    pub fn generate_value_samples_and_labels(&self) -> (Vec<Vec<usize>>, Vec<usize>) {
        let mut samples = Vec::new();
        let mut labels = Vec::new();

        for addr in self.graph_annotate.graph_data.value_node_addrs.iter() {
            let sample = self.generate_value_sample(*addr);
            let label = self.generate_value_label(*addr);

            // skip trivial samples (if param is set)
            if *crate::params::REMOVE_TRIVIAL_ZERO_SAMPLES &&
                sample.ends_with(&vec![0; ((self.depth - 1) * 2) - 1]) && 
                label == 0
            {
                continue;
            }

            samples.push(sample);
            labels.push(label);
        }
        
        (samples, labels)
    }

    /// get the semantics data from the associated dtn node (for a value node)
    fn add_features_from_associated_dtn(&self, addr: u64) -> Vec<usize> {
        let mut feature: Vec<usize> = Vec::new();

        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
        let parent_dtn_node: &Node = self.graph_annotate.graph_data.addr_to_node.get(
            &node.get_dtn_addr().unwrap()
        ).unwrap();

        // add features from parent dtn node
        match parent_dtn_node {
            Node::DataStructureNode(data_structure_node) => {
                feature.push(data_structure_node.byte_size);
                feature.push(((node.get_address() - data_structure_node.addr) / crate::params::BLOCK_BYTE_SIZE as u64) as usize);
                feature.push(data_structure_node.nb_pointer_nodes);
                feature.push(data_structure_node.nb_value_nodes);
            },
            _ => // if the node is not in a data structure, we return a vector of 0
                feature.append(&mut vec![0; 3]),
        }

        feature
    }

    /// generate the value embedding of a value node
    fn generate_value_sample(&self, addr: u64) -> Vec<usize> {
        let mut feature: Vec<usize> = self.add_features_from_associated_dtn(addr);
        let mut ancestor_features = self.get_neighbors(addr, petgraph::Direction::Incoming);
        

        feature.append(&mut ancestor_features); // ancestor_feature is left empty
        feature
    }

    /// get the children/ancestor (given direction) of a node
    /// in order : dtn_depth_1, ptr_depth_1, dtn_depth_2, ptr_depth_2, ... , dtn_depth_n, ptr_depth_n
    fn get_neighbors(&self, addr: u64, dir : petgraph::Direction) -> Vec<usize> {
        let mut result : Vec<usize> = Vec::new();
        // vectorize ancestors
        let mut current_node_addrs: HashSet<u64>;
        let mut ancestor_addrs: HashSet<u64> = HashSet::new();
        ancestor_addrs.insert(addr);

        for i in 0..self.depth {
            // swap current and next ancestors
            current_node_addrs = ancestor_addrs;
            ancestor_addrs = HashSet::new();

            let mut nb_dtn = 0;
            let mut nb_ptr = 0;

            for ancestor_addr in current_node_addrs.iter() {
                let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(ancestor_addr).unwrap();

                // count current nodes types
                match node {
                    Node::DataStructureNode(_) => nb_dtn += 1,
                    Node::PointerNode(_) => nb_ptr += 1,
                    _ => (),
                }

                // get the next ancestors
                for neighbor in self.graph_annotate.graph_data.graph.neighbors_directed(
                    *ancestor_addr, dir
                ) {
                    ancestor_addrs.insert(neighbor);
                }
            }
            
            if i > 0 { // skip the first value (always the same case)
                result.push(nb_dtn); // add number of dtns
                result.push(nb_ptr);  // add number of ptrs
            }
        }

        result
    }

    /// generate label for the value node (1 if the node is a key, 0 otherwise)
    fn generate_value_label(&self, addr: u64) -> usize {
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
        if node.is_key() {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{self};

    #[test]
    fn test_label_to_csv() {
        crate::tests::setup();

        let graph_embedding = GraphEmbedding::new(
            params::TEST_HEAP_DUMP_FILE_PATH.clone(), 
            crate::params::BLOCK_BYTE_SIZE,
            5,
            true
        ).unwrap();

        graph_embedding.save_samples_and_labels_to_csv(
            params::TEST_CSV_EMBEDDING_FILE_PATH.clone()
        );
    }
}