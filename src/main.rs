// This program is a port of https://github.com/prakhar1989/type-inference 
// from OCaml to Rust. Right now it is pretty much just a line-per-line port,
// but I plan to refactor the code to be more Rust-y and add more features soon.
//
// TODO:
// 1. Add better error messages for type inference errors.
// 2. Fix https://github.com/prakhar1989/type-inference/issues/5
// 3. Intergrate a parser (lalrpop?) to allow users to type their own expressions.
// 4. Add typing for more features. 
// 5. Publish as a crate on crates.io

#![feature(box_syntax)]

extern crate type_inference;
use type_inference::infer::*;
use type_inference::ast::*;
use type_inference::parser;

use std::collections::HashMap;
use std::io;
use std::io::Write;

fn main() {
    repl();
}
fn get_ids(e: &Expr) -> Vec<String> {
    match *e {
        Expr::Num(..) | Expr::Bool(..) => vec![],
        Expr::BinOp(ref l, _, ref r) => [get_ids(l), get_ids(r)].concat(),
        Expr::App(ref fun, ref arg) => [get_ids(fun), get_ids(arg)].concat(),
        Expr::Var(ref n) => vec![n.clone()],
        Expr::Fun(ref arg_name, ref body) => [vec![arg_name.clone()], get_ids(body)].concat()
    }
}

fn repl() {
    println!("Welcome to the type inference REPL");
    println!("Hit ^C to quit ");
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();
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
        let mut env = HashMap::new();
        let mut name_gen = NameGenerator::new();
        let ids = get_ids(&expr);
        for name in ids {
            env.insert(name, PrimitiveType::Var(name_gen.next_name()));
        }
        match infer(&env, &expr, &mut name_gen) {
            Ok(aexpr) => println!("{}", type_of(&aexpr)),
            Err(e) => println!("type error: {:?}", e),
        }
    }
}