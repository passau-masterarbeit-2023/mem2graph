use std::collections::HashMap;
use std::{path::PathBuf, fs::File, io::Write};
use crate::graph_embedding::GraphEmbedding;
use crate::graph_embedding::embedding::chunk_semantic_embedding::generate_semantic_samples_of_a_chunk;
use crate::graph_embedding::embedding::value_node_semantic_embedding::generate_value_sample;
use crate::graph_structs::Node;
use crate::params::argv::SelectAnnotationLocation;

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

    let embedding_fields_str = embedding_fields.join(",") + ",entropy";
    let mut node_addr_to_embedding_str = HashMap::new();

    for (node_addr, node_embedding, entropy) in node_embeddings {
        let mut node_embedding_str = String::new();
        for i in 0..embedding_fields.len() {
            let field = embedding_fields.get(i).unwrap();
            node_embedding_str.push_str(&node_embedding.get(field).unwrap().to_string());
            if i < embedding_fields.len() - 1 {
                node_embedding_str.push_str(",");
            }
        }

        // add entropy as a last field
        node_embedding_str.push_str(&format!(",{}", entropy));
        
        node_addr_to_embedding_str.insert(
            *node_addr, 
            format!("[{}]", node_embedding_str).to_string(),
        );
    }

    // save the graph to dot file
    let mut dot_file = File::create(output_file_path).unwrap();
    
    let dot_graph_with_comments = format!("{}", 
        graph.stringify_with_comment_hashmap( 
            embedding_fields_str, 
            &node_addr_to_embedding_str
        )
    );

    dot_file.write_all(
        dot_graph_with_comments.as_bytes()).unwrap(); // using the custom formatter
    return 0; // no samples, only the graph
}
