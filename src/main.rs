mod parser;

fn main() {
    let cli = parser::parse();

    dbg!(cli);

    println!("Hello, world!");
}
