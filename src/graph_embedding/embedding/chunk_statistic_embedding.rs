use crate::{utils::{compute_statistics, shannon_entropy, get_bin_to_index_size, get_bin_to_index}, graph_embedding::{GraphEmbedding, utils_embedding::{get_node_label, extract_chunk_data_as_bytes, extract_chunk_data_as_bits}}};

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
pub fn generate_chunk_statistic_embedding(graph_embedding : &GraphEmbedding, n_gram : &Vec<usize>, block_size : usize) -> Vec<(Vec<usize>, Vec<f64>)> {
    let mut samples = Vec::new();
    for chn_addr in graph_embedding.graph_annotate.graph_data.chn_addrs.iter() {
        if graph_embedding.is_entropy_filtered_addr(chn_addr) {
            continue;
        }
        let sample = generate_chunk_statistic_samples(graph_embedding, *chn_addr, n_gram, block_size);
        samples.push(sample);
    }
    samples
}

/// generate statistic embedding of a chunk
fn generate_chunk_statistic_samples(graph_embedding : &GraphEmbedding, chn_addr: u64, n_gram : &Vec<usize>, block_size : usize) -> 
    (Vec<usize>, Vec<f64>) {
    let mut feature_usize: Vec<usize> = Vec::new();
    let mut feature_f64: Vec<f64> = Vec::new();
    
    // -------- usize
    
    // common information
    feature_usize.push(chn_addr.try_into().expect("addr overflow in embedding"));

    // add n-gram
    let mut n_gram_vec = generate_n_gram_for_chunk(graph_embedding, chn_addr, n_gram);
    feature_usize.append(&mut n_gram_vec);

    // add label
    feature_usize.push(get_node_label(graph_embedding, chn_addr));

    // -------- f64

    let mut common_statistics = generate_common_statistic_for_chunk(graph_embedding, chn_addr, block_size);
    feature_f64.append(&mut common_statistics);
    

    (feature_usize, feature_f64)
}

/// generate common statistic
fn generate_common_statistic_for_chunk(graph_embedding : &GraphEmbedding, addr: u64, block_size : usize) -> Vec<f64> {
    let mut statistics = Vec::new();


    let bytes = extract_chunk_data_as_bytes(graph_embedding, addr, block_size);

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
    graph_embedding : &GraphEmbedding, 
    chn_addr: u64, 
    n_grams : &Vec<usize>,
) -> Vec<usize> {
    let mut n_gram_result = vec![0; get_bin_to_index_size()];

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

            // get the index of the window
            let index = get_bin_to_index(&window);
            // increment the index
            n_gram_result[index] += 1;
        }
    }

    n_gram_result
}
