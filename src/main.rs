pub mod tokenize;
pub mod parse;
pub mod generate;
pub mod trans;

use std::io;
use std::io::Read;
use std::fs::File;
use std::env;
use std::io::BufWriter;

fn main() -> io::Result<()> {
    let mut args = env::args();
    let _program = args.next().unwrap();
    let in_filename = args.next().expect("Please specify input filename");
    let out_filename = args.next().expect("Please specify output filename");
    let mut in_file = File::open(in_filename).expect("Cannot open file");
    let mut code = String::new();
    in_file.read_to_string(&mut code)?;
    let tokens = tokenize::tokenize(&code).expect("Failde to tokenize");
    println!("Tokens = {:?}", tokens);
    let ast = parse::parse(&tokens).expect("Failed to parse");
    println!("AST = {:?}", ast);
    let prog = generate::generate(&ast).expect("Failed to generate program");
    println!("Program = {:?}", prog);
    let mut out_file_buf = BufWriter::new(File::create(out_filename).expect("Cannot create file"));
    trans::trans(&prog, &mut out_file_buf);

    Ok(())
}
