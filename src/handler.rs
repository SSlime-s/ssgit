use crate::parser::{Cli, Commands};
use anyhow::{bail, Result};

mod cat_file;
mod commit_tree;
mod hash_object;
mod ls_files;
mod update_index;
mod write_tree;

pub fn handle(cli: &Cli) -> Result<()> {
    let Some(commands) = cli.command.as_ref() else {
        bail!("No command provided");
    };

    match commands {
        Commands::CatFile(args) => cat_file::handle(args),
        Commands::HashObject(args) => hash_object::handle(args),
        Commands::UpdateIndex(args) => update_index::handle(args),
        Commands::WriteTree(args) => write_tree::handle(args),
        Commands::LsFiles(args) => ls_files::handle(args),
        Commands::CommitTree(args) => commit_tree::handle(args),
    }
}
