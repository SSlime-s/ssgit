use crate::{entity::index::Index, parser::LsFilesArgs};
use anyhow::Result;

pub fn handle(args: &LsFilesArgs) -> Result<()> {
    dbg!(args);

    let Some(index) = Index::read()? else {
        return Ok(());
    };

    if !args.stage {
        for entry in index.entries {
            println!("{}", entry.file_name);
        }
        return Ok(());
    }

    for entry in index.entries {
        println!("{} {} 0\t{}", entry.mode, entry.hash, entry.file_name);
    }

    Ok(())
}
