use crate::object::GitObject;
use crate::parser::{HashObjectArgs, ObjectType};
use anyhow::Result;

pub fn handle(args: &HashObjectArgs) -> Result<()> {
    dbg!(args);

    if args.type_ != ObjectType::Blob {
        todo!();
    }

    let file_content = std::fs::read(args.file.as_str())?;

    let object = GitObject::new(args.type_.into(), file_content);
    let hash = object.hash();
    let object_bytes = object.to_raw()?;

    let path = hash.get_object_path();

    if path.exists() {
        let existing_content = std::fs::read(&path)?;
        dbg!(existing_content == object_bytes);
        println!("{}", hash);
        return Ok(());
    }

    if args.write {
        object.write()?;
    }

    println!("{}", hash);

    Ok(())
}
