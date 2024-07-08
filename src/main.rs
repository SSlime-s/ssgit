mod consts;
mod entity;
mod handler;
mod parser;

use anyhow::Result;

fn main() -> Result<()> {
    let cli = parser::parse();

    handler::handle(&cli)
}
