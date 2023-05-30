use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Argv {
    /// File to path to heap dump file
    /// NOTE: 'group = "file_input_group"' means that only one of the options in the group can be used
    /// the result is stored always in 'files_input', and on the option used (the other is None)
    #[arg(short, long, required = false, group = "file_input_group")]
    pub files: Option<Vec<String>>,

    /// The directory containing the heap dump files
    #[arg(short, long, required = false, group = "file_input_group")]
    pub directories: Option<Vec<String>>,

    #[arg(long, requires = "file_input_group")]
    pub files_input: Option<Vec<String>>, 

    /// only processing the graph (annotate, but not embedding) and save .gv file
    #[arg(short, long, required = false)]
    pub only_graph: bool,
}

pub fn get_program_args() -> Argv {
    return Argv::parse();
}