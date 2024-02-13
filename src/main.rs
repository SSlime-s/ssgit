mod consts;
mod handler;
mod index;
mod object;
mod parser;
mod tree;

use anyhow::Result;

fn main() -> Result<()> {
    let cli = parser::parse();

    handler::handle(&cli)
}
