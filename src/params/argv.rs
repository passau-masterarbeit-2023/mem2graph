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
    #[arg(value_enum, short, long, default_value = "value-embedding")]
    pub pipeline: Pipeline,

    /// The directory to output the results
    #[arg(short, long, required = false)]
    pub output: Option<String>,

    /// If the graph is annotated
    /// NOTE : By default the graph is annotated, if you want to disable it use this flag
    #[arg(short = 'a', long, action)]
    pub no_annotation: bool,

    /// if their is no value node or pointer node in the graph
    /// NOTE : By default the graph contains value node and pointer node, if you want to disable it use this flag
    /// NOTE : This flag is only used if the pipeline is 'SemanticEmbeddingDTN' or 'DtsExtraction' or 'graph'
    #[arg(short = 'v', long, action)]
    pub no_value_node: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Pipeline {
    /// make the value embedding
    ValueEmbedding,
    /// make the graph and save it
    Graph,
    /// make a semantic embedding of the DTN
    SemanticEmbeddingDTN,
    /// make a statistic embedding of the DTN
    StatisticEmbeddingDTN,
    /// extract the dts informations from the heap dump
    DtsExtraction,
}

pub fn get_program_args() -> Argv {
    return Argv::parse();
}