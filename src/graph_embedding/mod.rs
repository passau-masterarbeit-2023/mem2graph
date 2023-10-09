mod embedding;


mod utils_embedding;
mod neighboring;

use std::cmp::Ordering;
use std::collections::HashMap;

#[cfg(test)]
use crate::exe_pipeline::save_embedding;
use crate::graph_annotate::GraphAnnotate;
use crate::graph_structs::Node;
use crate::params::MIN_NB_OF_CHUNKS_TO_KEEP;
use crate::params::argv::{SelectAnnotationLocation, EntropyFilter};

use std::path::PathBuf;

use self::embedding::chunk_semantic_embedding::generate_chunk_semantic_embedding;
use self::embedding::chunk_statistic_embedding::generate_chunk_statistic_embedding;
use self::embedding::chunk_top_vn_semantic_embedding::generate_chunk_top_vn_semantic_embedding;
use self::embedding::value_node_semantic_embedding::generate_value_node_semantic_embedding;

type SamplesAndLabels = (Vec<HashMap<String, usize>>, Vec<usize>);

pub struct GraphEmbedding {
    graph_annotate: GraphAnnotate,
    depth: usize,

    entropy_treshold: Option<f64>,
}

impl GraphEmbedding {
    pub fn new(
        heap_dump_raw_file_path: PathBuf, 
        pointer_byte_size: usize,
        depth: usize,
        entropy_filter : EntropyFilter,
        annotation : SelectAnnotationLocation,
        without_value_node : bool,
    ) -> Result<GraphEmbedding, crate::utils::ErrorKind> {
        let graph_annotate = GraphAnnotate::new(heap_dump_raw_file_path, pointer_byte_size, annotation, without_value_node)?;
        let mut graph_embedding = GraphEmbedding {
            graph_annotate,
            depth,
            entropy_treshold: None,
        };

        graph_embedding.entropy_treshold = graph_embedding.calculate_entropy_treshold(entropy_filter);

        Ok(graph_embedding)
    }

    /// calculate the minimum entropy for a chunk node or a parent chunk node of data node to be kept
    fn calculate_entropy_treshold(&self, entropy_filter : EntropyFilter) -> Option<f64>{
        match entropy_filter {
            EntropyFilter::None => None,
            EntropyFilter::OnlyMaxEntropy => {
                // get the max entropy
                let mut max_entropy = f64::MIN;
                for addr in self.graph_annotate.graph_data.chn_addrs.iter() {
                    let chunk_header_node = self.graph_annotate.graph_data.addr_to_node.get(addr).unwrap();
                    match chunk_header_node {
                        Node::ChunkHeaderNode(chn) => {
                            if chn.start_data_bytes_entropy > max_entropy {
                                max_entropy = chn.start_data_bytes_entropy;
                            }
                        }
                        _ => panic!("the vector self.graph_annotate.graph_data.chn_addrs should only contains chunk header node"),
                    }

                }
                Some(max_entropy)
            },

            EntropyFilter::MinOfChunkTresholdEntropy => {
                let mut entropy_ordonned_chn_addr = self.graph_annotate.graph_data.chn_addrs.clone();

                // sort the vector by entropy descending
                // TODO: Finish the algo
                entropy_ordonned_chn_addr.sort_by(|addr_a, addr_b| {
                    let node_a = self.graph_annotate.graph_data.addr_to_node.get(addr_a).unwrap();
                    let node_b = self.graph_annotate.graph_data.addr_to_node.get(addr_b).unwrap();

                    let entropy_a = match node_a {
                        Node::ChunkHeaderNode(chn) => chn.start_data_bytes_entropy,
                        _ => panic!("the vector self.graph_annotate.graph_data.chn_addrs should only contains chunk header node"),
                    };

                    let entropy_b = match node_b {
                        Node::ChunkHeaderNode(chn) => chn.start_data_bytes_entropy,
                        _ => panic!("the vector self.graph_annotate.graph_data.chn_addrs should only contains chunk header node"),
                    };

                    entropy_b.partial_cmp(&entropy_a).unwrap_or(Ordering::Equal) // descending order
                });

                // get the entropy treshold with the min of chunk

                let nb_chunks = *MIN_NB_OF_CHUNKS_TO_KEEP;

                let chn_addr = 
                    if nb_chunks >= entropy_ordonned_chn_addr.len() {
                        entropy_ordonned_chn_addr[entropy_ordonned_chn_addr.len() - 1]
                    }else{
                        entropy_ordonned_chn_addr[nb_chunks]
                    };

                let node = self.graph_annotate.graph_data.addr_to_node.get(&chn_addr).unwrap();
                match node {
                    Node::ChunkHeaderNode(chn) => Some(chn.start_data_bytes_entropy),
                    _ => panic!("the vector self.graph_annotate.graph_data.chn_addrs should only contains chunk header node"),
                }
            },
        }
    }

    /// return true if the node is in a chunk filtered by the entropy treshold
    /// if the node is annotated, return false (no filter)
    /// if the entropy treshold is None, return false (no filter)
    pub fn is_entropy_filtered_addr(&self, addr : &u64) -> bool {
        if self.graph_annotate.graph_data.node_addr_to_annotations.contains_key(addr) {
            return false;
        }
        match self.entropy_treshold {
            None => false,
            Some(entropy_treshold) => {
                let node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
                match node {
                    Node::ChunkHeaderNode(chn) => chn.start_data_bytes_entropy < entropy_treshold,
                    _ => { // get the parent entropy
                        let parent_chn_addr = node.get_parent_chn_addr().expect("The chn addr should be set");
                        let parent_chn_node = self.graph_annotate.graph_data.addr_to_node.get(&parent_chn_addr).unwrap();

                        match parent_chn_node {
                            Node::ChunkHeaderNode(chn) => chn.start_data_bytes_entropy < entropy_treshold,
                            _ => panic!("the parent of a value node should be a chunk header node"),
                        }
                    }
                }
            },
        }
    }

    #[cfg(test)]
    fn save_samples_and_labels_to_csv(&self, csv_path: PathBuf) {
        let (samples, labels) = self.generate_value_node_semantic_embedding();
        save_embedding(samples, labels, csv_path);
    }

    // ----------------------------- statistic chunk embedding -----------------------------//
    pub fn generate_statistic_samples_for_all_chunks(&self, n_gram : &Vec<usize>, block_size : usize) -> (Vec<(HashMap<String, usize>, HashMap<String, f64>)>, Vec<usize>) {
        generate_chunk_statistic_embedding(&self, n_gram, block_size)
    }

    // ----------------------------- semantic chunk embedding -----------------------------//
    pub fn generate_chunk_semantic_embedding(&self) -> SamplesAndLabels {
        generate_chunk_semantic_embedding(&self)
    }

    // ----------------------------- value embedding -----------------------------//
    pub fn generate_value_node_semantic_embedding(&self) -> SamplesAndLabels {
        generate_value_node_semantic_embedding(&self)
    }

    // ----------------------------- chunk top value node embedding -----------------------------//
    pub fn generate_chunk_top_vn_semantic_embedding(&self) -> SamplesAndLabels {
        generate_chunk_top_vn_semantic_embedding(&self)
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
            EntropyFilter::None,
            SelectAnnotationLocation::ValueNode,
            false,
        ).unwrap();

        graph_embedding.save_samples_and_labels_to_csv(
            params::TEST_CSV_EMBEDDING_FILE_PATH.clone()
        );
    }
}