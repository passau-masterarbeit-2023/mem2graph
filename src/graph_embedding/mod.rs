mod embedding;


mod utils_embedding;
mod neighboring;

#[cfg(test)]
use crate::exe_pipeline::value_embedding::save_value_embeding;
use crate::graph_annotate::GraphAnnotate;
use crate::params::argv::SelectAnnotationLocation;

use std::path::PathBuf;

use self::embedding::chunk_semantic_embedding::generate_chunk_semantic_embedding;
use self::embedding::chunk_statistic_embedding::generate_chunk_statistic_embedding;
use self::embedding::value_node_semantic_embedding::generate_value_node_semantic_embedding;

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
        generate_chunk_statistic_embedding(&self, n_gram, block_size)
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
        generate_chunk_semantic_embedding(&self)
    }


    // ----------------------------- value embedding -----------------------------//

    /// generate semantic embedding of the value nodes only
    /// Samples [
    ///     [0.3233, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     [0.1234, ..., 0.1234],
    ///     ... 
    /// ]
    /// 
    /// Labels [0.0, 1.0, ..., 0.0],
    pub fn generate_semantic_block_embedding(&self) -> (Vec<Vec<usize>>, Vec<usize>) {
        generate_value_node_semantic_embedding(&self)
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