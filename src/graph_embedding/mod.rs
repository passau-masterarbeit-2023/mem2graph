pub mod embedding;

mod utils_embedding;
mod neighboring;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

#[cfg(test)]
use crate::exe_pipeline::save_embedding;
use crate::graph_annotate::GraphAnnotate;
use crate::graph_structs::Node;
use crate::params::{MIN_NB_OF_CHUNKS_TO_KEEP, CHUNK_BYTES_SIZE_TO_KEEP_FILTER};
use crate::params::argv::{SelectAnnotationLocation, EntropyFilter, ChunkByteSizeFilter};

use std::path::PathBuf;

use self::embedding::chunk_extract::generate_chunk_extract;
use self::embedding::chunk_semantic_embedding::generate_chunk_semantic_embedding;
use self::embedding::chunk_start_bytes_embedding::generate_chunk_start_bytes_embedding;
use self::embedding::chunk_statistic_embedding::generate_chunk_statistic_embedding;
use self::embedding::chunk_top_vn_semantic_embedding::generate_chunk_top_vn_semantic_embedding;
use self::embedding::value_node_semantic_embedding::generate_value_node_semantic_embedding;

type SamplesAndLabels = (Vec<HashMap<String, usize>>, Vec<usize>);

pub struct GraphEmbedding {
    pub graph_annotate: GraphAnnotate,
    depth: usize,

    entropy_treshold: Option<f64>,
    chunk_bytes_size_to_keep_filter : Option<HashSet<usize>>,
}

impl GraphEmbedding {
    pub fn new(
        heap_dump_raw_file_path: PathBuf, 
        pointer_byte_size: usize,
        depth: usize,
        entropy_filter : EntropyFilter,
        chunk_byte_size_filter : ChunkByteSizeFilter,
        annotation : SelectAnnotationLocation,
        without_value_node : bool,
    ) -> Result<GraphEmbedding, crate::utils::ErrorKind> {
        let graph_annotate = GraphAnnotate::new(heap_dump_raw_file_path, pointer_byte_size, annotation, without_value_node)?;
        let mut graph_embedding = GraphEmbedding {
            graph_annotate,
            depth,
            entropy_treshold: None,
            chunk_bytes_size_to_keep_filter : None,
        };


        graph_embedding.chunk_bytes_size_to_keep_filter = Self::get_chunk_byte_size_filter(chunk_byte_size_filter);
        graph_embedding.entropy_treshold = graph_embedding.calculate_entropy_treshold(entropy_filter);

        Ok(graph_embedding)
    }

    /// manage the chunk byte size filter
    fn get_chunk_byte_size_filter(chunk_byte_size_filter : ChunkByteSizeFilter) -> Option<HashSet<usize>> {
        match chunk_byte_size_filter {
            ChunkByteSizeFilter::Activate => {
                Some((*CHUNK_BYTES_SIZE_TO_KEEP_FILTER).clone())
            },
            ChunkByteSizeFilter::None => None,
        }
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

    // --------------------------------------- filter    -------------------------------------------------//

    fn is_entropy_filtered(&self, addr : &u64) -> bool {
        match self.entropy_treshold {
            None => {
                false
            },
            Some(entropy_treshold) => {
                let node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
                match node {
                    Node::ChunkHeaderNode(chn) => {
                        chn.start_data_bytes_entropy < entropy_treshold
                    }
                    _ => { // get the parent entropy
                        let parent_chn_addr = node.get_parent_chn_addr().expect("The chn addr should be set");
                        let parent_chn_node = self.graph_annotate.graph_data.addr_to_node.get(&parent_chn_addr).unwrap();

                        match parent_chn_node {
                            Node::ChunkHeaderNode(chn) => {
                                chn.start_data_bytes_entropy < entropy_treshold
                            },
                            _ => {
                                panic!("the parent of a value node should be a chunk header node");
                            },
                        }
                    }
                }
            },
        }
    }

    fn is_byte_size_filtered(&self, addr : &u64) -> bool {
        match &self.chunk_bytes_size_to_keep_filter {
            None => {
                false
            }
            Some(filter) => {
                let node = self.graph_annotate.graph_data.addr_to_node.get(&addr).unwrap();
                match node {
                    Node::ChunkHeaderNode(chn) => {
                        !filter.contains(&chn.byte_size)
                    },
                    _ => { // get the parent entropy
                        let parent_chn_addr = node.get_parent_chn_addr().expect("The chn addr should be set");
                        let parent_chn_node = self.graph_annotate.graph_data.addr_to_node.get(&parent_chn_addr).unwrap();

                        match parent_chn_node {
                            Node::ChunkHeaderNode(chn) => {
                                !filter.contains(&chn.byte_size)
                            },
                            _ => panic!("the parent of a value node should be a chunk header node"),
                        }
                    }
                }
            }
        }
    }

    /// return true if the node is in a chunk filtered by the entropy treshold or the chunk byte size filter
    /// if the node is annotated, return false (no filter)
    pub fn is_filtered_addr(&self, addr : &u64) -> bool {
        // if the node is annotated, return false (no filter)
        if self.graph_annotate.graph_data.node_addr_to_annotations.contains_key(addr) {
            return false;
        }
        
        self.is_entropy_filtered(addr) || self.is_byte_size_filtered(addr)
    }

    pub fn is_filtering_active(&self) -> bool {
        self.entropy_treshold.is_some() || self.chunk_bytes_size_to_keep_filter.is_some()
    }

    // ----------------------------------------- test   -----------------------------------------------//

    #[cfg(test)]
    fn save_samples_and_labels_to_csv(&self, csv_path: PathBuf) {
        let (samples, labels) = self.generate_value_node_semantic_embedding();
        save_embedding(samples, labels, csv_path);
    }

    // ----------------------------- statistic chunk embedding -----------------------------//
    pub fn generate_chunk_statistic_embedding(&self, n_gram : &Vec<usize>, block_size : usize) -> (Vec<(HashMap<String, usize>, HashMap<String, f64>)>, Vec<usize>) {
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

    // ----------------------------- chunk start bytes embedding -----------------------------//
    pub fn generate_chunk_start_bytes_embedding(&self) -> SamplesAndLabels {
        generate_chunk_start_bytes_embedding(&self)
    }


    // ----------------------------------------------------------------------------------------//
    // ------------------------------------ chunk extraction --------------------------------------------//

    pub fn generate_chunk_extract(&self) -> (Vec<HashMap<String, String>>, Vec<usize>) {
        generate_chunk_extract(&self)
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
            ChunkByteSizeFilter::None,
            SelectAnnotationLocation::ValueNode,
            false,
        ).unwrap();

        graph_embedding.save_samples_and_labels_to_csv(
            params::TEST_CSV_EMBEDDING_FILE_PATH.clone()
        );
    }
}