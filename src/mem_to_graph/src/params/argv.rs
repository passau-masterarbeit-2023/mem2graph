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
    /// File to path to heap dump file
    #[arg(short, long, required = false, group = "file_input_group")]
    pub files: Option<Vec<String>>,

    /// The directory containing the heap dump files
    #[arg(short, long, required = false, group = "file_input_group")]
    pub directories: Option<Vec<String>>,

    #[arg(long, requires = "file_input_group")]
    pub files_input: Option<Vec<String>>, 

    /// the pipeline to run
    #[arg(value_enum, short, long, default_value = "value-embedding")]
    pub pipeline: Pipeline,

    /// The directory to output the results
    #[arg(short, long, required = false)]
    pub output: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Pipeline {
    /// make the value embedding
    ValueEmbedding,
    /// make the graph and save it
    Graph,
}

pub fn get_program_args() -> Argv {
    return Argv::parse();
}