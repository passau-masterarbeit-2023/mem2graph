use crate::graph_structs::{
    Node,
};
use crate::graph_annotate::GraphAnnotate;

use std::path::PathBuf;


pub struct GraphEmbedding {
    graph_annotate: GraphAnnotate,
}

impl GraphEmbedding {
    pub fn new(heap_dump_raw_file_path: PathBuf, pointer_byte_size: usize) -> GraphEmbedding {
        let graph_annotate = GraphAnnotate::new(heap_dump_raw_file_path, pointer_byte_size);
        
        GraphEmbedding {
            graph_annotate,
        }
    }

    fn save_samples_and_labels_to_csv(&self, csv_path: PathBuf) {
        // let (samples, labels) = self.generate_samples_and_labels();

        // let mut csv_writer = csv::Writer::from_path(csv_path).unwrap();

        // for (sample, label) in samples.iter().zip(labels.iter()) {
        //     let mut row = Vec::new();
        //     row.extend_from_slice(sample);
        //     row.push(*label);

        //     csv_writer.write_record(row).unwrap();
        // }

        // csv_writer.flush().unwrap();
    }

    /// Samples [
    ///     [0.3233, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     ... 
    /// ]
    /// 
    /// Labels [0.0, 1.0, ..., 0.0],
    pub fn generate_samples_and_labels(&self) -> (Vec<Vec<u32>>, Vec<u32>) {
        let mut samples = Vec::new();
        let mut labels = Vec::new();

        for addr in self.graph_annotate.graph_data.unannotated_value_node_addrs.iter() {
            let sample = self.generate_sample(*addr);
            let label = self.generate_label(*addr);

            samples.push(sample);
            labels.push(label);
        }
        
        (samples, labels)
    }

    fn add_features_from_associated_dtn(&self, child_addr: u64) -> Vec<u32> {
        let feature: Vec<u32> = Vec::new();

        // get the DNT from the child_addr
        feature
    }

    fn generate_sample(&self, addr: u64) -> Vec<u32> {
        let feature: Vec<u32> = Vec::new();


        

        feature
    }

    fn generate_label(&self, addr: u64) -> u32 {
        let node: &Node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
        if node.is_key() {
            1
        } else {
            0
        }
    }
}