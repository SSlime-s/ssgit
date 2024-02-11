macro_rules! GIT_DIRECTORY {
    () => {
        ".git"
    };
}

pub const OBJECTS_DIRECTORY: &str = concat!(GIT_DIRECTORY!(), "/objects");
pub const GIT_INDEX_PATH: &str = concat!(GIT_DIRECTORY!(), "/index");
