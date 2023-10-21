use std::collections::HashMap;
use std::{path::PathBuf, fs::File, io::Write};

use crate::graph_embedding::GraphEmbedding;
use crate::graph_embedding::embedding::chunk_semantic_embedding::generate_semantic_samples_of_a_chunk;
use crate::graph_embedding::embedding::chunk_start_bytes_embedding::generate_chunk_start_bytes_sample;
use crate::graph_embedding::embedding::chunk_statistic_embedding::generate_chunk_statistic_samples;
use crate::graph_structs::Node;
use crate::params::{N_GRAM, BLOCK_BYTE_SIZE, self};
use crate::params::argv::{SelectAnnotationLocation, Pipeline};

/// Generate a string representing the header of the embedding.
/// It is composed of the names of the fields of the embedding, 
/// separated by commas.
fn generate_embedding_header(
    graph_embedding: &GraphEmbedding,
    node_embeddings: &Vec<(&u64, HashMap<String, String>, f64)>,
) -> String {
    // sorted header of the embedding
    let (_, first_embedding, _) = node_embeddings.get(0).unwrap();

    let mut embedding_fields = first_embedding.keys().cloned().collect::<Vec<String>>();
    embedding_fields.sort();

    let optional_filtering = {
        if graph_embedding.is_filtering_active() {
            ",filtered"
        } else {
            ""
        }
    };

    let header_embedding_fields = 
        embedding_fields.join(",") 
        + ",entropy" 
        + optional_filtering;

    header_embedding_fields
}

/// A list as a string is a string of the form "[a,b,c,d]"
/// This function returns the number of elements in the list
fn get_len_of_str_list(list_as_str: &String) -> usize {
    let mut nb_of_elements = 0;
    let mut is_inside_list = false;
    let mut is_inside_element = false;

    for c in list_as_str.chars() {
        match c {
            '[' => {
                is_inside_list = true;
                is_inside_element = false;
            }
            ']' => {
                is_inside_list = false;
                if is_inside_element {
                    nb_of_elements += 1;
                }
            }
            ',' => {
                if is_inside_list && is_inside_element {
                    nb_of_elements += 1;
                }
                is_inside_element = false;
            }
            ' ' => {}
            _ => {
                if is_inside_list {
                    is_inside_element = true;
                }
            }
        }
    }
    nb_of_elements
}

/// Parse each node of the graph to generate its embedding
/// Returns a list of tuples (node_addr, node_embedding, entropy)
fn generate_base_nodes_embedding(
    graph_embedding: &GraphEmbedding,
) ->  Vec<(&u64, HashMap<String, String>, f64)>{
    // parse each node of the graph to generate its embedding
    let graph =  &graph_embedding.graph_annotate.graph_data;

    // generate the node embeddings
    let mut node_embeddings: Vec<(&u64, HashMap<String, String>, f64)> = Vec::new();
    for chn_addr in graph.chn_addrs.iter() {
        let chn = match graph.addr_to_node.get(chn_addr) {
            Some(chn) => chn,
            None => {
                panic!("🚩 The CHN node [addr: {:?}] doesn't exist", chn_addr);
            }
        };
        match chn {
            Node::ChunkHeaderNode(chn) => {

                if graph_embedding.graph_annotate.annotation != SelectAnnotationLocation::ChunkHeaderNode {
                    panic!("🚩 For chunk embedding, the annotation must be ChunkHeaderNode");
                }
                
                // compute embedding
                let node_embedding: HashMap<String, String> = match params::ARGV.graph_comment_embedding_type {
                    Pipeline::ChunkSemanticEmbedding => {
                        let features = generate_semantic_samples_of_a_chunk(
                            graph_embedding, *chn_addr
                        )
                        // convert the features to string
                        .iter().map(|(k, v)| {
                            (k.clone(), v.to_string())
                        }).collect::<HashMap<String, String>>();
                        features
                    }, 
                    Pipeline::ChunkStatisticEmbedding => {
                        let (feature_usize, feature_f64) = generate_chunk_statistic_samples(
                            graph_embedding, *chn_addr, &*N_GRAM, BLOCK_BYTE_SIZE
                        );
                        // combines the two hashmaps into one of string
                        let mut features = feature_usize
                            .iter().map(|(k, v)| {
                                (k.clone(), v.to_string())
                            }).collect::<HashMap<String, String>>();
                        features.extend(
                            feature_f64.iter().map(|(k, v)| {
                                (k.clone(), v.to_string())
                            }).collect::<HashMap<String, String>>()
                        );
                        features
                    }
                    Pipeline::ChunkStartBytesEmbedding => {
                        let features = generate_chunk_start_bytes_sample(
                            graph_embedding, *chn_addr
                        ).iter().map(|(k, v)| {
                            (k.clone(), v.to_string())
                        }).collect::<HashMap<String, String>>();
                        features
                    }
                    _ => {
                        panic!("🚩 {:?} not supported for graph generation with embedding comments",
                            params::ARGV.graph_comment_embedding_type
                        );
                    }
                };

                node_embeddings.push((chn_addr, node_embedding, chn.start_data_bytes_entropy));
            },
            _ => {
                panic!("🚩 Only ChunkHeaderNode is supported for graph generation with embedding comments");
            }
        }
    }
    node_embeddings
}

fn convert_nodes_embedding_to_comment_hashmap(
    graph_embedding: &GraphEmbedding,
    node_embeddings: &Vec<(&u64, HashMap<String, String>, f64)>,
    header_embedding_length: usize,
) -> HashMap<u64, String> {
    let mut node_addr_to_embedding_comment = HashMap::new();
    for (node_addr, node_embedding, entropy) in node_embeddings {
        let mut node_embedding_str = String::new();

        // convert the embedding to a string
        node_embedding_str.push_str(
            &node_embedding.values().cloned().collect::<Vec<String>>().join(",")
        );

        // add entropy as additional field
        node_embedding_str.push_str(&format!(",{}", entropy));

        // add optional filtering as additional field
        if graph_embedding.is_filtering_active() {
            if graph_embedding.is_filtered_addr(node_addr) {
                node_embedding_str.push_str(",1")
            } else {
                node_embedding_str.push_str(",0")
            }
        }
        
        let embedding_list_as_str = format!("[{}]", node_embedding_str);

        // check that the length of the embedding is correct
        let embedding_length = get_len_of_str_list(&embedding_list_as_str);
        if embedding_length != header_embedding_length {
            panic!(
                "🚩 The length of the node embedding is not correct: {} != {}",
                embedding_length,
                header_embedding_length
            );
        }

        node_addr_to_embedding_comment.insert(
            **node_addr, 
            embedding_list_as_str,
        );
    }
    node_addr_to_embedding_comment
}

/// Generate a graph to dot file for the given file.
pub fn gen_and_save_memory_graph_with_embedding_comments(
    output_file_path: PathBuf, 
    graph_embedding: &GraphEmbedding,
) -> usize {
    let graph =  &graph_embedding.graph_annotate.graph_data;

    // generate the node embeddings
    let node_embeddings = generate_base_nodes_embedding(graph_embedding);

    // generate the header of the embedding, and its length
    let header_embedding_fields = format!("[{}]",
        generate_embedding_header(
            graph_embedding, 
            &node_embeddings
        )
    );
    let header_embedding_length = get_len_of_str_list(&header_embedding_fields);
    
    // convert the node embeddings to a hashmap of comments
    let node_addr_to_embedding_comment = convert_nodes_embedding_to_comment_hashmap(
        graph_embedding, 
        &node_embeddings, 
        header_embedding_length
    );

    // save the graph to dot file
    let mut dot_file = File::create(output_file_path).unwrap();
    
    let used_embedding_type = match params::ARGV.graph_comment_embedding_type {
        Pipeline::ChunkSemanticEmbedding => "chunk-semantic-embedding",
        Pipeline::ChunkStatisticEmbedding => "chunk-statistic-embedding",
        Pipeline::ChunkStartBytesEmbedding => "chunk-start-bytes-embedding",
        _ => {
            panic!("🚩 {:?} not supported for graph generation with embedding comments",
                params::ARGV.graph_comment_embedding_type
            );
        }
    };
    let graph_comment = format!(
        "{{ \"embedding-type\": {}, \"embedding-fields\": {} }}", 
        used_embedding_type, header_embedding_fields
    );
    let dot_graph_with_comments = format!("{}", 
        graph.stringify_with_comment_hashmap( 
            graph_comment, 
            &node_addr_to_embedding_comment
        )
    );

    dot_file.write_all(
        dot_graph_with_comments.as_bytes()).unwrap(); // using the custom formatter
    return 0; // no samples, only the graph
}
