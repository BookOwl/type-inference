// This program is a port of https://github.com/prakhar1989/type-inference 
// from OCaml to Rust. Right now it is pretty much just a line-per-line port,
// but I plan to refactor the code to be more Rust-y and add more features soon.
//
// TODO:
// * Fix https://github.com/prakhar1989/type-inference/issues/5
// * Add typing for more features. 

extern crate type_inference;
use type_inference::infer::*;
use type_inference::parser;

use std::io;
use std::io::Write;

fn main() {
    repl();
}

fn repl() {
    println!("Welcome to the type inference REPL");
    println!("Hit ^C to quit ");
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    loop {
        print!("> ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let expr = match parser::parse_Expr(&input) {
            Ok(expr) => expr,
            Err(_) => {
                println!("syntax error");
                continue;
            }
        };
        let mut var_gen = VarGenerator::new();
        let env = top_level_env(&mut var_gen);
        match type_of(&expr, &env, &mut var_gen) {
            Ok(typ) => println!("{}", typ),
            Err(e) => println!("{:?}", e),
        }
    }
}