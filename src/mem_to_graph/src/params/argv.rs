use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Argv {
    /// File to path to heap dump file
    #[arg(short, long, required = false, exclusive = true)]
    pub file: Option<String>,

    /// Number of times to greet
    #[arg(short, long, required = false, exclusive = true)]
    pub directory: Option<String>,
}

pub fn get_program_args() -> Argv {
    return Argv::parse();
}