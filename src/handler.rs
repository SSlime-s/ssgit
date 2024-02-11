use crate::parser::{Cli, Commands};
use anyhow::{bail, Result};

mod cat_file;
mod hash_object;
mod update_index;
mod write_tree;
mod ls_files;

pub fn handle(cli: &Cli) -> Result<()> {
    let Some(commands )= cli.command.as_ref() else {
        bail!("No command provided");
    };

    match commands {
        Commands::CatFile(args) => cat_file::handle(args),
        Commands::HashObject(args) => hash_object::handle(args),
        Commands::UpdateIndex(args) => update_index::handle(args),
        Commands::WriteTree(args) => write_tree::handle(args),
        Commands::LsFiles(args) => ls_files::handle(args),
    }
}
