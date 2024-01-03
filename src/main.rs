mod handler;
mod parser;
mod object;

use anyhow::Result;

fn main() -> Result<()> {
    let cli = parser::parse();

    handler::handle(&cli)
}
