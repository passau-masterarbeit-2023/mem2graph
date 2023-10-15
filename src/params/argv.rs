use clap::{Parser, ValueEnum};

// NOTE: 'group = "file_input_group"' means that only one of the options in the group can be used
// the result is stored always in 'files_input', and on the option used (the other is None)
// NOTE: the "///" comments are used to generate the help message
/// Graph generation and embedding program
/// NOTE : to add multiple files/folders duplicate the flags before the files/folders, 
/// like : cargo run -- -f /path/to/file1 -f /path/to/file2
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Argv {
    /// File path to heap dump file
    #[arg(short, long, required = false, group = "file_input_group")]
    pub files: Option<Vec<String>>,

    /// The directory containing the heap dump files
    #[arg(short, long, required = false, group = "file_input_group")]
    pub directories: Option<Vec<String>>,

    /// The directory or file containing the heap dump files
    #[arg(long, requires = "file_input_group")]
    pub files_input: Option<Vec<String>>, 

    /// The pipeline to run
    #[arg(value_enum, short, long, default_value = "value-node-embedding")]
    pub pipeline: Pipeline,

    /// The directory to output the results
    #[arg(short, long, required = false)]
    pub output: Option<String>,

    /// How the graph is annotated
    /// NOTE : By default the graph is annotated with value node
    #[arg(short = 'a', long, default_value = "value-node")]
    pub annotation: SelectAnnotationLocation,

    /// If the embedding is filtered with the entropy of the firsts blocks of each chunk
    /// NOTE : only used in the embedding pipeline
    #[arg(short = 'e', long, default_value = "none")]
    pub entropy_filter : EntropyFilter,

    /// if their is no value node or pointer node in the graph
    /// 
    /// NOTE : By default the graph contains value node and pointer node, 
    ///     if you want to disable it use this flag
    /// NOTE : This flag is only used if the pipeline is 
    ///     'ChunkSemanticEmbedding' or 'ChunkExtraction' or 'graph'
    #[arg(short = 'v', long, action)]
    pub no_value_node: bool,
}


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
/// control the annotation of the graph
/// This specifies where we want the annotation to be
/// By default the annotation is on the value node
/// But we can also desire to annotate the chunk containing the annotation address (here as chunk header node)
pub enum SelectAnnotationLocation {
    /// annotate the value node
    ValueNode,
    /// annotate the chunk header node
    ChunkHeaderNode,
    /// don't annotate the graph
    None,
}


/// Filter the embedding with the entropy of the firsts blocks of each chunk
/// NOTE : the entropy is computed on the firsts blocks of each chunk
/// NOTE : the annotated blocks are not filtered
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum EntropyFilter {
    /// don't filter the embedding
    None,
    /// filter the graph, keeping only the chunk with the max entropy
    OnlyMaxEntropy,
    /// filter the graph with the entropy with a minimum of x chunks (defined by an env variable)
    /// NOTE : If the entropy minimal to have x chunks is Y, then all the chunk with Y entropy or more are kept
    MinOfChunkTresholdEntropy,
}


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Pipeline {
    /// make the value embedding
    ValueNodeEmbedding,
    /// make chunk top value node semantic embedding
    ChunkTopVnSemanticEmbedding,
    /// make the graph and save it
    Graph,
    /// make a semantic embedding of the chunk
    ChunkSemanticEmbedding,
    /// make a statistic embedding of the chunk
    ChunkStatisticEmbedding,
    /// make an embeding with the beginning of each chunk (the number of bytes is controlled by CHUNK_NB_OF_START_BYTES_FOR_CHUNK_ENTROPY)
    ChunkStartBytesEmbedding,

    /// make an easy extraction of the chunk (get the user data as a hexa string, with annotation)
    ChunkExtraction,
}

pub fn get_program_args() -> Argv {
    return Argv::parse();
}