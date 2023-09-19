#[cfg(test)]
use crate::exe_pipeline::value_embedding::save_value_embeding;
use crate::graph_structs::{Node, SpecialNodeAnnotation};
use crate::graph_annotate::GraphAnnotate;
use crate::utils::{generate_bit_combinations, to_n_bits_binary, u64_to_bytes, compute_statistics, shannon_entropy};

use std::path::PathBuf;
use std::collections::{HashSet, HashMap};

pub struct GraphEmbedding {
    graph_annotate: GraphAnnotate,
    depth: usize,
}

impl GraphEmbedding {
    pub fn new(
        heap_dump_raw_file_path: PathBuf, 
        pointer_byte_size: usize,
        depth: usize,
        annotation : bool,
        without_value_node : bool,
    ) -> Result<GraphEmbedding, crate::utils::ErrorKind> {
        let graph_annotate = GraphAnnotate::new(heap_dump_raw_file_path, pointer_byte_size, annotation, without_value_node)?;
        
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
            let mut base_info = self.get_dts_basics_informations(*dtn_addr);
            base_info.push(self.get_node_label(*dtn_addr));
            let data = self.extract_dts_data_as_hex_blocks(*dtn_addr, block_size, no_pointer);

            dts_base_info.push(base_info);
            dts_data.push(data);

        }

        (dts_base_info, dts_data)
    }
    
    /// extract the data of the dts :
    /// get the blocks block_size bytes by block_size bytes (get empty string if the block is a pointer, else get the hexa value)*
    fn extract_dts_data_as_hex_blocks(&self, addr: u64, block_size : usize, no_pointer : bool) -> Vec<String> {
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

    // ----------------------------- statistic DTN embedding -----------------------------//
    /// generate statistic embedding of all the DTN
    /// in order :
    ///    - DTN addresse (not really usefull for learning, but can bu usefull to further analyse the data)
    ///    - N-gram of the DTN (in number of bit, ascending order, bitwise order)
    /// Common statistic (f64)
    ///    - Mean Byte Value
    ///    - Mean Absolute Deviation (MAD)
    ///    - Standard Deviation
    ///    - Skewness
    ///    - Kurtosis
    ///    - Shannon entropy
    pub fn generate_statistic_dtns_samples(&self, n_gram : usize, block_size : usize) -> Vec<(Vec<usize>, Vec<f64>)> {
        let mut samples = Vec::new();
        // get dtn :
        for dtn_addr in self.graph_annotate.graph_data.dtn_addrs.iter() {
            let sample = self.generate_statistic_dtn_samples(*dtn_addr, n_gram, block_size);
            samples.push(sample);
        }
        samples
    }

    /// generate statistic embedding of a DTN
    fn generate_statistic_dtn_samples(&self, addr: u64, n_gram : usize, block_size : usize) -> 
        (Vec<usize>, Vec<f64>) {
        let mut feature_usize: Vec<usize> = Vec::new();
        let mut feature_f64: Vec<f64> = Vec::new();
        
        // -------- usize
        
        // common information
        feature_usize.push(addr.try_into().expect("addr overflow in embedding"));

        // add n-gram
        let mut n_gram_vec = self.generate_n_gram_dtns(addr, n_gram, block_size);
        feature_usize.append(&mut n_gram_vec);

        // add label
        feature_usize.push(self.get_node_label(addr));

        // -------- f64

        let mut common_statistics = self.generate_common_statistic_dtns(addr, block_size);
        feature_f64.append(&mut common_statistics);
        

        (feature_usize, feature_f64)
    }

    /// generate common statistic
    fn generate_common_statistic_dtns(&self, addr: u64, block_size : usize) -> Vec<f64> {
        let mut statistics = Vec::new();


        let bytes = self.extract_dts_data_as_bytes(addr, block_size);

        let result = compute_statistics(&bytes);

        statistics.push(result.0);
        statistics.push(result.1);
        statistics.push(result.2);
        statistics.push(result.3);
        statistics.push(result.4);


        statistics.push(shannon_entropy(&bytes));
        
        statistics
    }

    /// generate all the n-gram of the DTN until n (include)
    fn generate_n_gram_dtns(&self, addr: u64, n: usize, block_size : usize) -> Vec<usize> {
        let mut n_gram = Vec::new();

        let mut n_gram_counter = HashMap::<String, usize>::new();
        // keep the ordonned key to reconstruct the vector
        let mut ordonned_key = Vec::<String>::new();
        // reserve this much of space because addition of all power of 2 until n is 2^(n + 1) - 1
        ordonned_key.reserve(1 << (n + 1));

        // initialise the hashmap
        for i in 1..(n + 1){
            let mut bit_combi = generate_bit_combinations(i);

            
            for combi in bit_combi.iter() {
                n_gram_counter.insert(combi.clone(), 0);
            }

            ordonned_key.append(&mut bit_combi);
        }

        n_gram.reserve(ordonned_key.len());

        // get th bits of the dtn
        let dtn_bits = self.extract_dts_data_as_bits(addr, block_size);

        // for each bit
        for char_i in 0..dtn_bits.len() {
            // get the window
            let mut window = String::new();
            for window_size in 1..(n + 1) {
                if char_i + window_size > dtn_bits.len() {
                    break;
                }
                window.push(dtn_bits[char_i + window_size - 1]);
                let window_count = n_gram_counter.get_mut(&window).unwrap();
                *window_count += 1;
            }
        }

        // construct the final vecteur in order
        for key in ordonned_key.iter() {
            n_gram.push(*n_gram_counter.get(key).unwrap());
        }


        n_gram
    }


    /// extract the data of the dts :
    /// get all the bit of the dts as u8
    fn extract_dts_data_as_bytes(&self, addr: u64, block_size : usize) -> Vec<u8> {
        let mut data = Vec::new();
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

        match node {
            Node::DataStructureNode(data_structure_node) => {
                let mut current_addr = data_structure_node.addr + block_size as u64;

                // get the data of the dts
                for _ in 1..(data_structure_node.byte_size/8) {
                    // get the node at the current address
                    let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&current_addr).unwrap();
                    // if the block is a pointer
                    if node.is_pointer() {
                        let bits = u64_to_bytes(node.points_to().unwrap());
                        data.extend_from_slice(&bits);
                        
                    } else {
                        let current_block = node.get_value().unwrap();
                        data.extend_from_slice(&current_block);
                    }
                    
                    current_addr += block_size as u64;
                }
            },
            _ => // if the node is not in a data structure, we panic
                panic!("Node is not a DTN"),
        }
        data
    }


    /// extract the data of the dts :
    /// get all the bit of the dts as char ('1' or 'O')
    fn extract_dts_data_as_bits(&self, addr: u64, block_size : usize) -> Vec<char> {
        let mut data = Vec::new();
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

        match node {
            Node::DataStructureNode(data_structure_node) => {
                let mut current_addr = data_structure_node.addr + block_size as u64;
                let block_size_bit = (1 << block_size) as usize;

                // get the data of the dts
                for _ in 1..(data_structure_node.byte_size/8) {
                    // get the node at the current address
                    let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&current_addr).unwrap();
                    // if the block is a pointer
                    if node.is_pointer() {
                        let mut bits = to_n_bits_binary(node.points_to().unwrap(), block_size_bit).chars().collect();
                        data.append(&mut bits);
                        
                    } else {
                        let current_block = node.get_value().unwrap();
                        
                        // convert the block to binary
                        for i in 0..block_size {
                            // each value of the array are bytes, so 8 bit long
                            let mut bits = to_n_bits_binary(current_block[i] as u64, 8).chars().collect();
                            data.append(&mut bits);
                        }
                    }
                    
                    current_addr += block_size as u64;
                }
            },
            _ => // if the node is not in a data structure, we panic
                panic!("Node is not a DTN"),
        }
        data
    }

    // ----------------------------- semantic DTN embedding -----------------------------//
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
        feature.push(self.get_node_label(addr));

        feature
    }


    /// get the label of a dtn (graph_structure, DtnType)
    /// Basestruct = 0,
    /// Keystruct = 1,
    /// SshStruct = 2,
    /// SessionStateStruct = 3
    fn get_node_label(&self, addr : u64) -> usize {
        let annotation = self.graph_annotate.graph_data.special_node_to_annotation.get(&addr);
        match annotation {
            Some(annotation) => {
                match annotation {
                    SpecialNodeAnnotation::KeyNodeAnnotation(_) => 1,
                    SpecialNodeAnnotation::SshStructNodeAnnotation(_) => 2,
                    SpecialNodeAnnotation::SessionStateNodeAnnotation(_) => 3,
                }
            },
            None => 0,
        }
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
        // so extract all the children of the dtn
        let mut ancestor_addrs: HashSet<u64> = HashSet::new();
        let children = self.graph_annotate.graph_data.graph.neighbors_directed(dtn_addr, petgraph::Direction::Outgoing);
        for child_addr in children {
            ancestor_addrs.insert(child_addr);
        }

        let mut result : Vec<usize> = Vec::new();
        // vectorize ancestors
        let mut current_node_addrs: HashSet<u64>;

        // for each depth
        for _ in 0..self.depth {
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
            
            result.push(nb_dtn); // add number of dtns
            result.push(nb_ptr);  // add number of ptrs
        }

        result
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
            let label = self.get_node_label(*addr);

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
            true,
            false,
        ).unwrap();

        graph_embedding.save_samples_and_labels_to_csv(
            params::TEST_CSV_EMBEDDING_FILE_PATH.clone()
        );
    }
}