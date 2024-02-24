mod consts;
mod handler;
mod parser;
mod entity;

use anyhow::Result;

fn main() -> Result<()> {
    let cli = parser::parse();

    handler::handle(&cli)
}
