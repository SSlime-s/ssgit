use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    CatFile(CatFileArgs),
    HashObject(HashObjectArgs),
    UpdateIndex(UpdateIndexArgs),
    WriteTree(WriteTreeArgs),
    LsFiles(LsFilesArgs),
    CommitTree(CommitTreeArgs),
    UpdateRef(UpdateRefArgs),
    Init(InitArgs),
    Add(AddArgs),
}

#[derive(Args, Debug)]
pub struct CatFileArgs {
    pub hash: String,

    #[command(flatten)]
    pub options: CatFileArgsOptions,
}
#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
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

#[derive(Args, Debug)]
pub struct HashObjectArgs {
    pub file: String,

    #[arg(short)]
    pub write: bool,

    #[arg(short, value_enum, default_value_t=ObjectType::Blob)]
    pub type_: ObjectType,
}

#[derive(Args, Debug)]
pub struct UpdateIndexArgs {
    pub file: Vec<String>,

    #[arg(long)]
    pub add: bool,

    #[arg(long)]
    pub remove: bool,

    #[arg(long, num_args(3),value_names(&["mode", "object", "file"]))]
    pub cacheinfo: Option<Vec<String>>,
}

#[derive(Args, Debug)]
pub struct WriteTreeArgs {}

#[derive(Args, Debug)]
pub struct LsFilesArgs {
    #[arg(short, long, default_value_t = true)]
    pub cached: bool,

    #[arg(short, long)]
    pub stage: bool,
}

#[derive(Args, Debug)]
pub struct CommitTreeArgs {
    pub tree: String,

    #[arg(short)]
    pub parent: Vec<String>,

    #[arg(short)]
    pub message: Vec<String>,
}

#[derive(Args, Debug)]
pub struct UpdateRefArgs {
    pub ref_: String,
    pub newvalue: String,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(short = 'b', long)]
    pub initial_branch: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    pub files: Vec<String>,
}

pub fn parse() -> Cli {
    Cli::parse()
}
