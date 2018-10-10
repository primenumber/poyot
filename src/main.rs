pub mod tokenize;
pub mod parse;
pub mod generate;

use std::io;
use std::io::Read;
use std::fs::File;
use std::env;

fn main() -> io::Result<()> {
    let mut args = env::args();
    let _program = args.next().unwrap();
    let filename = args.next().expect("Please specify filename");
    let mut f = File::open(filename).expect("Cannot open file");
    let mut code = String::new();
    f.read_to_string(&mut code)?;
    let tokens = tokenize::tokenize(&code).expect("Failde to tokenize");
    println!("Tokens = {:?}", tokens);
    let ast = parse::parse(&tokens).expect("Failed to parse");
    println!("AST = {:?}", ast);
    let prog = generate::generate(&ast).expect("Failed to generate program");
    println!("Program = {:?}", prog);

    Ok(())
}
