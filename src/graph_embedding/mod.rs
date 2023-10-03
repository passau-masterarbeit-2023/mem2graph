#[cfg(test)]
use crate::exe_pipeline::value_embedding::save_value_embeding;
use crate::graph_structs::Node;
use crate::graph_annotate::GraphAnnotate;
use crate::params::BLOCK_BYTE_SIZE;
use crate::params::argv::SelectAnnotationLocation;
use crate::utils::{to_n_bits_binary, u64_to_bytes, compute_statistics, shannon_entropy, get_bin_to_index, get_bin_to_index_size};

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
        annotation : SelectAnnotationLocation,
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
        let (samples, labels) = self.generate_semantic_block_embedding();
        save_value_embeding(samples, labels, csv_path, self.depth);
    }

    // ----------------------------- extracting chunks -----------------------------//
    /// extract the data of all the chunks from the graph
    /// the couple (chunk_base_info, chunks_data) is returned for each chunk
    pub fn extract_all_chunks_data(&self, block_size : usize, no_pointer : bool) -> (Vec<Vec<usize>>, Vec<Vec<String>>) {
        let mut chunks_base_info = Vec::new();
        let mut chunks_data = Vec::new();

        for chn_addr in self.graph_annotate.graph_data.chn_addrs.iter() {
            let mut base_info = self.get_chunk_basics_informations(*chn_addr);
            base_info.push(self.get_node_label(*chn_addr));
            let data = self.extract_chunk_data_as_hex_blocks(*chn_addr, block_size, no_pointer);

            chunks_base_info.push(base_info);
            chunks_data.push(data);

        }

        (chunks_base_info, chunks_data)
    }
    
    /// extract the data of the chunk :
    /// get the blocks block_size bytes by block_size bytes (get empty string if the block is a pointer, else get the hexa value)*
    fn extract_chunk_data_as_hex_blocks(&self, addr: u64, block_size : usize, no_pointer : bool) -> Vec<String> {
        let mut data = Vec::new();
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

        match node {
            Node::ChunkHeaderNode(chunk_header_node) => {
                let mut current_addr = chunk_header_node.addr + block_size as u64;

                // get the data of the chunk
                for _ in 1..(chunk_header_node.byte_size/8) {
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
            _ => panic!("Node is not a chunk"),
        }
        data
    }

    // ----------------------------- statistic chunk embedding -----------------------------//
    /// generate statistic embedding of all chunks
    /// in order :
    ///    - CHN addresse (not really usefull for learning, but can bu usefull to further analyse the data)
    ///    - N-gram of the chunk data (in number of bit, ascending order, bitwise order)
    /// Common statistic (f64)
    ///    - Mean Byte Value
    ///    - Mean Absolute Deviation (MAD)
    ///    - Standard Deviation
    ///    - Skewness
    ///    - Kurtosis
    ///    - Shannon entropy
    pub fn generate_statistic_samples_for_all_chunks(&self, n_gram : &Vec<usize>, block_size : usize) -> Vec<(Vec<usize>, Vec<f64>)> {
        let mut samples = Vec::new();
        for chn_addr in self.graph_annotate.graph_data.chn_addrs.iter() {
            let sample = self.generate_chunk_statistic_samples(*chn_addr, n_gram, block_size);
            samples.push(sample);
        }
        samples
    }

    /// generate statistic embedding of a chunk
    fn generate_chunk_statistic_samples(&self, chn_addr: u64, n_gram : &Vec<usize>, block_size : usize) -> 
        (Vec<usize>, Vec<f64>) {
        let mut feature_usize: Vec<usize> = Vec::new();
        let mut feature_f64: Vec<f64> = Vec::new();
        
        // -------- usize
        
        // common information
        feature_usize.push(chn_addr.try_into().expect("addr overflow in embedding"));

        // add n-gram
        let mut n_gram_vec = self.generate_n_gram_for_chunk(chn_addr, n_gram);
        feature_usize.append(&mut n_gram_vec);

        // add label
        feature_usize.push(self.get_node_label(chn_addr));

        // -------- f64

        let mut common_statistics = self.generate_common_statistic_for_chunk(chn_addr, block_size);
        feature_f64.append(&mut common_statistics);
        

        (feature_usize, feature_f64)
    }

    /// generate common statistic
    fn generate_common_statistic_for_chunk(&self, addr: u64, block_size : usize) -> Vec<f64> {
        let mut statistics = Vec::new();


        let bytes = self.extract_chunk_data_as_bytes(addr, block_size);

        let result = compute_statistics(&bytes);

        statistics.push(result.0);
        statistics.push(result.1);
        statistics.push(result.2);
        statistics.push(result.3);
        statistics.push(result.4);


        statistics.push(shannon_entropy(&bytes));
        
        statistics
    }

    /// generate all the n-gram of the chunk
    fn generate_n_gram_for_chunk(
        &self, 
        chn_addr: u64, 
        n_grams : &Vec<usize>,
    ) -> Vec<usize> {
        let mut n_gram_result = vec![0; get_bin_to_index_size()];

        // get bits of the chunk
        let chunk_bits = self.extract_chunk_data_as_bits(chn_addr);

        // for each bit
        for char_i in 0..chunk_bits.len() {
            let mut window = String::new();
            // get the window
            for window_size in n_grams {
                // if the window is too big, we stop
                if char_i + window_size > chunk_bits.len() {
                    break;
                }
                
                // Extend the window to match the current window_size
                while window.len() < *window_size {
                    window.push(chunk_bits[char_i + window.len()]);
                }

                // get the index of the window
                let index = get_bin_to_index(&window);
                // increment the index
                n_gram_result[index] += 1;
            }
        }

        n_gram_result
    }


    /// extract the data of the chunk :
    /// get all the bit of the chunk as u8
    fn extract_chunk_data_as_bytes(&self, addr: u64, block_size : usize) -> Vec<u8> {
        let mut data = Vec::new();
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

        match node {
            Node::ChunkHeaderNode(chunk_header_node) => {
                let mut current_addr = chunk_header_node.addr + block_size as u64;

                // get the data of the chunk
                for _ in 1..(chunk_header_node.byte_size/8) {
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
            _ => panic!("Node is not a chunk"),
        }
        data
    }


    /// extract the data of the chunk :
    /// get all the bit of the chunk as char ('1' or 'O')
    fn extract_chunk_data_as_bits(&self, addr: u64) -> Vec<char> {
        let mut data = Vec::new();
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

        match node {
            Node::ChunkHeaderNode(chunk_header_node) => {
                let mut current_addr = chunk_header_node.addr + BLOCK_BYTE_SIZE as u64;
                let block_size_bit = (BLOCK_BYTE_SIZE * 8) as usize;

                // get the data of the chunk
                for _ in 1..(chunk_header_node.byte_size/8) {
                    // get the node at the current address
                    let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&current_addr).unwrap();
                    // if the block is a pointer
                    if node.is_pointer() {
                        let mut bits = to_n_bits_binary(node.points_to().unwrap(), block_size_bit).chars().collect();
                        data.append(&mut bits);
                        
                    } else {
                        let current_block = node.get_value().unwrap();
                        
                        // convert the block to binary
                        for i in 0..BLOCK_BYTE_SIZE {
                            // each value of the array are bytes, so 8 bit long
                            let mut bits = to_n_bits_binary(current_block[i] as u64, 8).chars().collect();
                            data.append(&mut bits);
                        }
                    }
                    
                    current_addr += BLOCK_BYTE_SIZE as u64;
                }
            },
            _ => panic!("Node is not a chunk"),
        }
        data
    }

    // ----------------------------- semantic chunk embedding -----------------------------//
    /// generate semantic embedding of all the chunks
    /// in order :
    ///     - chunk header addresse (not really usefull for learning, but can bu usefull to further analyse the data)
    ///     - chunk size
    ///     - nb pointer
    /// 
    ///     - ancestor (in order of depth, alternate CHN/PTR)
    ///     - children (same)
    ///     - label (if the chunk contains a key, or is the ssh or sessionState)
    pub fn generate_semantic_samples_for_all_chunks(&self) -> Vec<Vec<usize>> {
        let mut samples = Vec::new();
        // get chunk :
        for chn_addr in self.graph_annotate.graph_data.chn_addrs.iter() {
            let sample = self.generate_semantic_samples_of_a_chunk(*chn_addr);
            samples.push(sample);
        }
        samples
    }

    fn generate_semantic_samples_of_a_chunk(&self, chn_addr: u64) -> Vec<usize> {
        let mut feature: Vec<usize> = Vec::new();

        let mut info = self.get_chunk_basics_informations(chn_addr);
        feature.append(&mut info);

        // add ancestors
        let mut ancestors = self.generate_samples_for_neighbor_nodes_of_the_chunk(
            chn_addr, petgraph::Direction::Incoming
        );
        feature.append(&mut ancestors);

        // add children
        let mut children = self.generate_samples_for_neighbor_nodes_of_the_chunk(
            chn_addr, petgraph::Direction::Outgoing
        );
        feature.append(&mut children);

        // add label
        feature.push(self.get_node_label(chn_addr));

        feature
    }


    /// get the label of a node
    /// Basestruct = 0,
    /// Keystruct = 1,
    /// SshStruct = 2,
    /// SessionStateStruct = 3
    fn get_node_label(&self, addr : u64) -> usize {
        let annotation = self.graph_annotate.graph_data.node_addr_to_annotations.get(&addr);
        match annotation {
            Some(annotation) => {
                annotation.annotation_set_embedding() as usize
            },
            None => 0,
        }
    }

    /// extract the basics information of the chunk
    /// Return [addr, size, nb_pointer]
    fn get_chunk_basics_informations(&self, addr: u64) -> Vec<usize> {
        let mut info = Vec::new();
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();

        // add features from parent chn node
        match node {
            Node::ChunkHeaderNode(chunk_header_node) => {
                info.push(chunk_header_node.addr.try_into().expect("addr overflow in embedding")); // WARN : can be overflow !!!!!!
                info.push(chunk_header_node.byte_size);
                info.push(chunk_header_node.nb_pointer_nodes);
            },
            _ => panic!("Node is not a chunk"),
        }
        info
    }

    /// Generate the ancestor/children (in given direction) embedding 
    /// with respect to a given start chn address.
    /// 
    /// NOTE: Since nothing points to a CHN, we starts from the children 
    /// nodes of the given chunk of the CHN.
    /// 
    /// (number of ptn and number of chn for each depth)
    fn generate_samples_for_neighbor_nodes_of_the_chunk(
        &self, 
        addr : u64, 
        dir : petgraph::Direction
    ) -> Vec<usize> {
        // get the children for starting the algorithm
        let mut ancestor_addrs: HashSet<u64> = HashSet::new();
        let children = 
            self.graph_annotate.graph_data.graph.neighbors_directed(
                addr, petgraph::Direction::Outgoing
            );
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

            let mut nb_chn = 0;
            let mut nb_ptr = 0;

            for ancestor_addr in current_node_addrs.iter() {
                let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(ancestor_addr).unwrap();

                // count current nodes types
                match node {
                    Node::ChunkHeaderNode(_) => nb_chn += 1,
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
            
            result.push(nb_chn); // add number of chns
            result.push(nb_ptr);  // add number of ptrs
        }

        result
    }

    // ----------------------------- value embedding -----------------------------//

    /// generate semantic embedding of the nodes
    /// Samples [
    ///     [0.3233, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     ... 
    /// ]
    /// 
    /// Labels [0.0, 1.0, ..., 0.0],
    pub fn generate_semantic_block_embedding(&self) -> (Vec<Vec<usize>>, Vec<usize>) {
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

    /// get the semantics data from the parent chunk of a node
    fn add_features_from_parent_chunk(&self, addr: u64) -> Vec<usize> {
        let mut feature: Vec<usize> = Vec::new();

        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
        let parent_chn_node: &Node = self.graph_annotate.graph_data.addr_to_node.get(
            &node.get_parent_chn_addr().unwrap()
        ).unwrap();

        // add features from parent chn node
        match parent_chn_node {
            Node::ChunkHeaderNode(chunk_header_node) => {
                feature.push(chunk_header_node.byte_size);
                feature.push(((node.get_address() - chunk_header_node.addr) / crate::params::BLOCK_BYTE_SIZE as u64) as usize);
                feature.push(chunk_header_node.nb_pointer_nodes);
                feature.push(chunk_header_node.nb_value_nodes);
            },
            _ => // if the node is not in a chunk, we return a vector of 0
                feature.append(&mut vec![0; 3]),
        }

        feature
    }

    /// generate the value embedding of a value node
    fn generate_value_sample(&self, addr: u64) -> Vec<usize> {
        let mut feature: Vec<usize> = self.add_features_from_parent_chunk(addr);
        let mut ancestor_features = self.get_neighbors(addr, petgraph::Direction::Incoming);
        

        feature.append(&mut ancestor_features); // ancestor_feature is left empty
        feature
    }

    /// get the children/ancestor (given direction) of a node
    /// in order : chn_depth_1, ptr_depth_1, chn_depth_2, ptr_depth_2, ... , chn_depth_n, ptr_depth_n
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

            let mut nb_chn = 0;
            let mut nb_ptr = 0;

            for ancestor_addr in current_node_addrs.iter() {
                let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(ancestor_addr).unwrap();

                // count current nodes types
                match node {
                    Node::ChunkHeaderNode(_) => nb_chn += 1,
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
                result.push(nb_chn); // add number of chns
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
            SelectAnnotationLocation::ValueNode,
            false,
        ).unwrap();

        graph_embedding.save_samples_and_labels_to_csv(
            params::TEST_CSV_EMBEDDING_FILE_PATH.clone()
        );
    }
}