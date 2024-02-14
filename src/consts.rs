macro_rules! GIT_DIRECTORY {
    () => {
        ".git"
    };
}

pub const GIT_DIRECTORY: &str = GIT_DIRECTORY!();
pub const HEAD_PATH: &str = concat!(GIT_DIRECTORY!(), "/HEAD");
pub const REFS_DIRECTORY: &str = concat!(GIT_DIRECTORY!(), "/refs");
pub const OBJECTS_DIRECTORY: &str = concat!(GIT_DIRECTORY!(), "/objects");
pub const GIT_INDEX_PATH: &str = concat!(GIT_DIRECTORY!(), "/index");

pub const DEFAULT_BRANCH: &str = "main";
