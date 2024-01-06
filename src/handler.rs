use crate::parser::{Cli, Commands};
use anyhow::{Result, bail};

mod cat_file;
mod hash_object;

pub fn handle(cli: &Cli) -> Result<()> {
    let Some(commands )= cli.command.as_ref() else {
        bail!("No command provided");
    };

    match commands {
        Commands::CatFile(args) => {
            cat_file::handle(args)
        }
        Commands::HashObject(args) => {
            hash_object::handle(args)
        }
    }
}
