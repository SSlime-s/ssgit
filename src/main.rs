mod handler;
mod parser;
mod object;
mod consts;
mod index;
mod tree;

use anyhow::Result;

fn main() -> Result<()> {
    let cli = parser::parse();

    handler::handle(&cli)
}
