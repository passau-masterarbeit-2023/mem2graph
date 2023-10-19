use std::collections::HashMap;
use crate::graph_embedding::GraphEmbedding;
use crate::graph_embedding::utils_embedding::{get_node_label, extract_chunk_data_as_bytes, extract_chunk_data_as_bits, get_chunk_basics_informations};
use crate::utils::{compute_statistics, shannon_entropy, get_bin_to_nb_starting};

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
pub fn generate_chunk_statistic_embedding(
    graph_embedding : &GraphEmbedding, 
    n_gram : &Vec<usize>, 
    block_size : usize
) -> (Vec<(HashMap<String, usize>, HashMap<String, f64>)>, Vec<usize>) {
    let mut samples = Vec::new();
    let mut labels = Vec::new();
    for chn_addr in graph_embedding.graph_annotate.graph_data.chn_addrs.iter() {
        if graph_embedding.is_filtered_addr(chn_addr) {
            continue;
        }
        let sample = generate_chunk_statistic_samples(graph_embedding, *chn_addr, n_gram, block_size);
        samples.push(sample);
        labels.push(get_node_label(graph_embedding, *chn_addr));
    }
    (samples, labels)
}

/// generate statistic embedding of a chunk
fn generate_chunk_statistic_samples(graph_embedding : &GraphEmbedding, chn_addr: u64, n_gram : &Vec<usize>, block_size : usize) -> 
    (HashMap<String, usize>, HashMap<String, f64>) {
    let mut feature_usize = get_chunk_basics_informations(graph_embedding, chn_addr);
    let mut feature_f64 = HashMap::new();
    
    // -------- usize

    // add n-gram
    let n_gram_vec = generate_n_gram_for_chunk(graph_embedding, chn_addr, n_gram);
    feature_usize.extend(n_gram_vec);

    // -------- f64

    let common_statistics = generate_common_statistic_for_chunk(graph_embedding, chn_addr, block_size);
    feature_f64.extend(common_statistics);
    

    (feature_usize, feature_f64)
}

/// generate common statistic
fn generate_common_statistic_for_chunk(graph_embedding : &GraphEmbedding, addr: u64, block_size : usize) -> HashMap<String, f64> {


    let bytes = extract_chunk_data_as_bytes(graph_embedding, addr, block_size);

    let mut result = compute_statistics(&bytes);


    result.insert("shannon_entropy".to_string(), shannon_entropy(&bytes));
    
    result
}

/// generate all the n-gram of the chunk
fn generate_n_gram_for_chunk(
    graph_embedding : &GraphEmbedding, 
    chn_addr: u64, 
    n_grams : &Vec<usize>,
) -> HashMap<String, usize> {
    let mut n_gram_result = get_bin_to_nb_starting();

    // get bits of the chunk
    let chunk_bits = extract_chunk_data_as_bits(graph_embedding, chn_addr);

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

            // increment the counter of the window
            let x = n_gram_result.get_mut(&window).unwrap();
            *x += 1;
        }
    }

    n_gram_result
}
