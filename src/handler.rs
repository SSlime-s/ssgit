use crate::parser::{Cli, Commands};
use anyhow::{bail, Result};

mod add;
mod branch;
mod cat_file;
mod commit;
mod commit_tree;
mod hash_object;
mod init;
mod ls_files;
mod switch;
mod update_index;
mod update_ref;
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
        Commands::UpdateRef(args) => update_ref::handle(args),
        Commands::Init(args) => init::handle(args),
        Commands::Add(args) => add::handle(args),
        Commands::Commit(args) => commit::handle(args),
        Commands::Switch(args) => switch::handle(args),
        Commands::Branch(args) => branch::handle(args),
    }
}
