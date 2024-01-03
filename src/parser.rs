use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Test {
        #[arg(short, long)]
        list: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
