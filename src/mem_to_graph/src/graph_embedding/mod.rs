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
        depth: usize
    ) -> Result<GraphEmbedding, crate::utils::ErrorKind> {
        let graph_annotate = GraphAnnotate::new(heap_dump_raw_file_path, pointer_byte_size)?;
        
        Ok(GraphEmbedding {
            graph_annotate,
            depth,
        })
    }

    fn save_samples_and_labels_to_csv(&self, csv_path: PathBuf) {
        let (samples, labels) = self.generate_samples_and_labels();
        crate::exe_pipeline::save(samples, labels, csv_path, self.depth);
    }

    /// Samples [
    ///     [0.3233, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     ... 
    /// ]
    /// 
    /// Labels [0.0, 1.0, ..., 0.0],
    pub fn generate_samples_and_labels(&self) -> (Vec<Vec<usize>>, Vec<usize>) {
        let mut samples = Vec::new();
        let mut labels = Vec::new();

        for addr in self.graph_annotate.graph_data.unannotated_value_node_addrs.iter() {
            let sample = self.generate_sample(*addr);
            let label = self.generate_label(*addr);

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

    fn generate_sample(&self, addr: u64) -> Vec<usize> {
        let mut feature: Vec<usize> = self.add_features_from_associated_dtn(addr);

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
                    *ancestor_addr, petgraph::Direction::Incoming
                ) {
                    ancestor_addrs.insert(neighbor);
                }
            }
            
            if i > 0 {
                feature.push(nb_dtn); // add number of dtns
                feature.push(nb_ptr);  // add number of ptrs
            }
        }
        

        feature
    }

    fn generate_label(&self, addr: u64) -> usize {
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
            5
        ).unwrap();

        graph_embedding.save_samples_and_labels_to_csv(
            params::TEST_CSV_EMBEDDING_FILE_PATH.clone()
        );
    }
}