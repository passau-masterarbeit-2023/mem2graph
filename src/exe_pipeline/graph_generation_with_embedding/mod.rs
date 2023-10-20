use std::collections::HashMap;
use std::{path::PathBuf, fs::File, io::Write};
use crate::graph_embedding::GraphEmbedding;
use crate::graph_embedding::embedding::chunk_semantic_embedding::generate_semantic_samples_of_a_chunk;
use crate::graph_embedding::embedding::value_node_semantic_embedding::generate_value_sample;
use crate::graph_structs::Node;
use crate::params::argv::SelectAnnotationLocation;

/// Generate a string representing the header of the embedding.
/// It is composed of the names of the fields of the embedding, 
/// separated by commas.
fn generate_embedding_header_and_embedding_length(
    graph_embedding: &GraphEmbedding,
    embedding_fields: &Vec<String>,
) -> String {
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

/// Generate a graph to dot file for the given file.
pub fn gen_and_save_memory_graph_with_embedding_comments(
    output_file_path: PathBuf, 
    graph_embedding: &GraphEmbedding,
) -> usize {
    // parse each node of the graph to generate its embedding
    let graph =  &graph_embedding.graph_annotate.graph_data;

    // generate the node embeddings
    let mut node_embeddings = Vec::new();
    for (node_addr, node) in graph.addr_to_node.iter() {
        match node {
            Node::ValueNode(vn) => {
                if graph_embedding.graph_annotate.annotation != SelectAnnotationLocation::ValueNode {
                    continue;
                }
                
                // compute embedding
                let node_embedding = generate_value_sample(
                    graph_embedding, *node_addr
                );

                // add entropy of parent chunk
                let chunk_parent_node = graph.addr_to_node.get(&vn.chn_addr).unwrap();
                let entropy = match chunk_parent_node {
                    Node::ChunkHeaderNode(chn) => {
                        chn.start_data_bytes_entropy
                    },
                    _ => {
                        panic!("Value node parent is not a chunk header node");
                    }
                };

                node_embeddings.push((node_addr, node_embedding, entropy));
            },
            Node::ChunkHeaderNode(chn) => {
                if graph_embedding.graph_annotate.annotation != SelectAnnotationLocation::ChunkHeaderNode {
                    continue;
                }
                let node_embedding = generate_semantic_samples_of_a_chunk(
                    graph_embedding, *node_addr
                );
                node_embeddings.push((node_addr, node_embedding, chn.start_data_bytes_entropy));
            },
            Node::PointerNode(_) => {continue;},
            Node::FooterNode(_) => {continue;},
        }
    }

    // transform the node embeddings to vectors then to string
    let (_, first_embedding, _) = node_embeddings.get(0).unwrap();

    let mut embedding_fields = first_embedding.keys().cloned().collect::<Vec<String>>();
    embedding_fields.sort();

    // header of the embedding
    let header_embedding_fields = format!("[{}]",
        generate_embedding_header_and_embedding_length(
            graph_embedding, 
            &embedding_fields
        )
    );
    let header_embedding_length = get_len_of_str_list(&header_embedding_fields);
    
    let mut node_addr_to_embedding_str = HashMap::new();

    for (node_addr, node_embedding, entropy) in node_embeddings {
        let mut node_embedding_str = String::new();

        // convert the embedding to a string
        for i in 0..embedding_fields.len() {
            let field = embedding_fields.get(i).unwrap();
            node_embedding_str.push_str(&node_embedding.get(field).unwrap().to_string());
            if i < embedding_fields.len() - 1 {
                node_embedding_str.push_str(",");
            }
        }

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
                "The length of the node embedding is not correct: {} != {}",
                embedding_length,
                header_embedding_length
            );
        }

        node_addr_to_embedding_str.insert(
            *node_addr, 
            embedding_list_as_str,
        );
    }

    // save the graph to dot file
    let mut dot_file = File::create(output_file_path).unwrap();
    
    let dot_graph_with_comments = format!("{}", 
        graph.stringify_with_comment_hashmap( 
            header_embedding_fields, 
            &node_addr_to_embedding_str
        )
    );

    dot_file.write_all(
        dot_graph_with_comments.as_bytes()).unwrap(); // using the custom formatter
    return 0; // no samples, only the graph
}
