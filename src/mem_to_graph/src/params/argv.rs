use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Argv {
    /// File to path to heap dump file
    #[arg(short, long, required = false, group = "file_input_group")]
    pub file: Option<Vec<String>>,

    /// The directory containing the heap dump files
    #[arg(short, long, required = false, group = "file_input_group")]
    pub directory: Option<Vec<String>>,

    #[arg(long, requires = "file_input_group")]
    pub file_input: Option<Vec<String>>,

    /// only processing the graph (annotate, but not embedding) and save .gv file
    #[arg(short, long, required = false)]
    pub only_graph: bool,
}

pub fn get_program_args() -> Argv {
    return Argv::parse();
}