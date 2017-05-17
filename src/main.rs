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

use std::collections::HashMap;

fn main() {
    infer_test()
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

fn infer_test() {
    let exprs = vec![
        ("Number", Expr::Num(123)),
        ("Bool", Expr::Bool(true)),
        ("Var", Expr::Var("x".to_owned())),
        ("BinOp", Expr::BinOp(
            box Expr::Num(1),
            Op::Add,
            box Expr::Num(2)
        )),
        ("BinOp 2", Expr::BinOp(
            box Expr::BinOp(
                box Expr::Num(3),
                Op::Add,
                box Expr::Num(-2),
            ),
            Op::Mul,
            box Expr::BinOp(
                box Expr::Var("spam".to_owned()),
                Op::Add,
                box Expr::Num(42),
            ),
        )),
        ("Function", Expr::Fun(
            "x".to_owned(),
            box Expr::BinOp(
                box Expr::Var("x".to_owned()),
                Op::Mul,
                box Expr::Var("x".to_owned()),
            ),
        )),
    ];
    let mut env = HashMap::new();
    for (_, expr) in exprs {
        let mut new_env = env.clone();
        let mut name_gen = NameGenerator::new();
        let ids = get_ids(&expr);
        for name in ids {
            new_env.insert(name, PrimitiveType::Var(name_gen.next_name()));
        }
        match infer(&new_env, &expr, &mut name_gen) {
            Ok(aexpr) => println!("expr: {} type: {}", expr, type_of(&aexpr)),
            Err(e) => println!("expr:{} Error: {:?}", expr, e),
        }
    }
}