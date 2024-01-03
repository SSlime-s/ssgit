use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    CatFile(CatFileArgs),
}

#[derive(Args, Debug)]
pub struct CatFileArgs {
    pub hash: String,

    #[command(flatten)]
    pub options: CatFileArgsOptions,
}
#[derive(Args, Debug)]
#[group(required = true, multiple=false)]
pub struct CatFileArgsOptions {
    #[arg(short)]
    pub pretty: bool,
    #[arg(short)]
    pub exists: bool,

    #[arg(short)]
    pub type_: bool,
    #[arg(short)]
    pub size: bool,
}

pub fn parse() -> Cli {
    Cli::parse()
}
