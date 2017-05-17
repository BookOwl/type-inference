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

use std::fmt::Display;
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Op {
    Add,
    Mul,
    Gt,
    Lt,
    And,
    Or,
}
impl Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Op::Add => "+",
            Op::Mul => "*",
            Op::Gt  => ">",
            Op::Lt  => "<",
            Op::And => "&&",
            Op::Or  => "||",
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum PrimitiveType {
    Num,
    Bool,
    Var(String),
    Fun(Box<PrimitiveType>, Box<PrimitiveType>),
}
impl Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            PrimitiveType::Num  => "num".to_owned(),
            PrimitiveType::Bool => "bool".to_owned(),
            PrimitiveType::Var(ref s) => format!("'{}", s),
            PrimitiveType::Fun(ref a, ref r) => format!("({} -> {})", a, r),
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Expr {
    Num(i32),
    Bool(bool),
    Var(String),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Fun(String, Box<Expr>),
    App(Box<Expr>,  Box<Expr>),
}
impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Expr::Num(ref n) => format!("{}", n),
            Expr::Bool(ref b) => format!("{}", b),
            Expr::Var(ref v) => format!("{}", v),
            Expr::BinOp(ref l, ref op, ref r) => format!("({} {} {})", l, op, r),
            Expr::Fun(ref id, ref body) => format!("(fun {} -> {})", id, body),
            Expr::App(ref func, ref op) => format!("({} {})", func, op),
    })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum AnnotedExpr {
    Num(i32, PrimitiveType),
    Bool(bool, PrimitiveType),
    Var(String, PrimitiveType),
    BinOp(Box<AnnotedExpr>, Op, Box<AnnotedExpr>, PrimitiveType),
    Fun(String, Box<AnnotedExpr>, PrimitiveType),
    App(Box<AnnotedExpr>,  Box<AnnotedExpr>, PrimitiveType),
}
impl Display for AnnotedExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            AnnotedExpr::Num(ref n, ref type_) => format!("({} : {})", n, type_),
            AnnotedExpr::Bool(ref b, ref type_) => format!("({} : {})", b, type_),
            AnnotedExpr::Var(ref v, ref type_) => format!("({} : {})", v, type_),
            AnnotedExpr::BinOp(ref l, ref op, ref r, ref type_) => format!("({} {} {} : {})", l, op, r, type_),
            AnnotedExpr::Fun(ref id, ref body, ref type_) => format!("(fun {} -> {}) : {}", id, body, type_),
            AnnotedExpr::App(ref func, ref op, ref type_) => format!("({} {}) : {}", func, op, type_),
        })
    }
}

/// A mapping between names and their types
type Enviroment = HashMap<String, PrimitiveType>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Constraint(PrimitiveType, PrimitiveType);
impl Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Constraint(ref t1, ref t2) = *self;
        write!(f, "{} == {}", t1, t2)
    }
}
type Constraints = Vec<Constraint>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Substitution(String, PrimitiveType);
impl Display for Substitution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Substitution(ref n, ref t) = *self;
        write!(f, "{} = {}", n, t)
    }
}
type Substitutions = Vec<Substitution>;

/// Creates an AnnotedExpr from an Expr or an Error 
/// if there is an undefined name.
fn annote(e: &Expr, env: &Enviroment, name_gen: &mut NameGenerator) -> Result<AnnotedExpr> {
    match *e {
        Expr::Num(n) => Ok(AnnotedExpr::Num(n, PrimitiveType::Num)),
        Expr::Bool(b) => Ok(AnnotedExpr::Bool(b, PrimitiveType::Bool)),
        Expr::Var(ref name) => {
            if let Some(t) = env.get(name) {
                Ok(AnnotedExpr::Var(name.clone(), t.clone()))
            } else {
                Err(Error::UndefinedName(name.clone()))
            }
        },
        Expr::BinOp(ref l, ref op, ref r) => {
            let l = annote(l, env, name_gen)?;
            let r = annote(r, env, name_gen)?;
            let new_name = name_gen.next_name();
            let new_type = PrimitiveType::Var(new_name);
            Ok(AnnotedExpr::BinOp(box l, *op, box r, new_type))
        },
        Expr::Fun(ref arg_name, ref body) => {
            let annoted_body = annote(body, env, name_gen)?;
            if let Some(t) = env.get(arg_name) {
                Ok(AnnotedExpr::Fun(arg_name.clone(), box annoted_body, 
                   PrimitiveType::Fun(box t.clone(), box PrimitiveType::Var(name_gen.next_name()))))
            } else {
                Err(Error::UndefinedName(arg_name.clone()))
            }
        },
        Expr::App(ref func, ref arg) => {
            let func = annote(func, env, name_gen)?;
            let arg = annote(arg, env, name_gen)?;
            let new_name = name_gen.next_name();
            let new_type = PrimitiveType::Var(new_name);
            Ok(AnnotedExpr::App(box func, box arg, new_type))
        },
    } 
}

/// Returns the type of an AnnotedExpr
fn type_of(e: &AnnotedExpr) -> PrimitiveType {
    match *e {
        AnnotedExpr::Num(_, ref t)         |
        AnnotedExpr::Bool(_, ref t)        |
        AnnotedExpr::Var(_, ref t)         |
        AnnotedExpr::App(_, _, ref t)      |
        AnnotedExpr::BinOp(_, _, _, ref t) |
        AnnotedExpr::Fun(_, _, ref t)       => t.clone() 
    }
}

fn collect(e: &AnnotedExpr) -> Result<Constraints> {
    match *e {
        AnnotedExpr::Bool(..) | AnnotedExpr::Num(..) => Ok(vec![]),
        // A single occurence of a variable gives us no info
        AnnotedExpr::Var(..) => Ok(vec![]), 
        AnnotedExpr::BinOp(ref left, ref op, ref right, ref res_type) => {
            let left_type = type_of(left);
            let right_type = type_of(right);
            let op_constraints = match *op {
                Op::Add | Op::Mul => vec![Constraint(left_type, PrimitiveType::Num),
                                          Constraint(right_type, PrimitiveType::Num),
                                          Constraint(res_type.clone(), PrimitiveType::Num)],
                Op::Gt | Op::Lt => vec![Constraint(left_type, right_type),
                                        Constraint(res_type.clone(), PrimitiveType::Bool)],
                Op::And | Op::Or => vec![Constraint(left_type, PrimitiveType::Bool),
                                         Constraint(right_type, PrimitiveType::Bool),
                                         Constraint(res_type.clone(), PrimitiveType::Bool)],
            };
            Ok([collect(left)?.as_slice(), collect(right)?.as_slice(), op_constraints.as_slice()].concat())
        },
        AnnotedExpr::Fun(ref arg_name, ref body, ref fun_type) => {
            match *fun_type {
                PrimitiveType::Fun(ref arg_type, ref ret_type) => {
                    let mut constraints = collect(body)?;
                    constraints.push(Constraint(type_of(body), *ret_type.clone()));
                    Ok(constraints)
                },
                _ => Err(Error::TypeError("Invalid AnnotedExpr: Applying non-Fun".to_owned())),
            }
        },
        AnnotedExpr::App(ref fun, ref arg, ref res_type) => {
            match type_of(fun) {
                PrimitiveType::Fun(ref fun_arg_type, ref fun_res_type) => {
                    let mut constraints = [collect(fun)?, collect(arg)?].concat();
                    constraints.push(Constraint(res_type.clone(), *fun_res_type.clone()));
                    constraints.push(Constraint(*fun_arg_type.clone(), type_of(arg)));
                    Ok(constraints)
                },
                PrimitiveType::Var(_) => {
                    Ok([collect(fun)?, collect(arg)?, 
                     vec![Constraint(type_of(fun), PrimitiveType::Fun(
                         box type_of(arg), box res_type.clone()
                     ))]].concat())
                },
                _ => Err(Error::TypeError("Invalid AnnotedExpr: Applying non-Fun".to_owned()))
            }
        }
    }
}

fn substitute(u: &PrimitiveType, x: &str, t: &PrimitiveType) -> PrimitiveType {
    match *t {
        PrimitiveType::Bool | PrimitiveType::Num => t.clone(),
        PrimitiveType::Var(ref n) => if *n == x { u.clone() } else { t.clone() },
        PrimitiveType::Fun(ref t1, ref t2) => PrimitiveType::Fun(box substitute(u, x, t1), box substitute(u, x, t2))
    }
}

fn apply(subs: &Substitutions, t: &PrimitiveType) -> PrimitiveType {
    subs.into_iter().rev().fold(t.clone(), |t, ref sub| {let sub = sub.clone(); substitute(&sub.1, &sub.0, &t)})
}

fn unify(constraints: &mut Constraints) -> Result<Substitutions> {
    if constraints.is_empty() {
        Ok(vec![])
    } else {
        let Constraint(x, y) = constraints.remove(0); // :(
        let t2 = unify(constraints)?;
        let t1 = unify_one(&apply(&t2, &x), &apply(&t2, &y))?;
        Ok([t1, t2].concat())
    }
}

fn unify_one(t1: &PrimitiveType, t2: &PrimitiveType) -> Result<Substitutions> {
    match (t1.clone(), t2.clone()) {
        (PrimitiveType::Num, PrimitiveType::Num) |
         (PrimitiveType::Bool, PrimitiveType::Bool) => Ok(vec![]),
        (PrimitiveType::Var(ref x), ref z) |
         (ref z, PrimitiveType::Var(ref x)) => Ok(vec![Substitution(x.clone(), z.clone())]),
        (PrimitiveType::Fun(ref a, ref b), PrimitiveType::Fun(ref x, ref y)) => {
            unify(&mut vec![Constraint(*a.clone(), *x.clone()),
                            Constraint(*b.clone(), *y.clone())])
        },
        _ => Err(Error::TypeError("mismatched types".to_owned()))
    }
}

fn apply_subs_to_expr(subs: &Substitutions, expr: &AnnotedExpr) -> AnnotedExpr {
    match *expr {
        AnnotedExpr::Num(ref n, ref t) => AnnotedExpr::Num(*n, apply(subs, t)),
        AnnotedExpr::Bool(ref b, ref t) => AnnotedExpr::Bool(*b, apply(subs, t)),
        AnnotedExpr::Var(ref v, ref t) => AnnotedExpr::Var(v.clone(), apply(subs, t)),
        AnnotedExpr::BinOp(ref l, ref op, ref r, ref t) => {
            let l = apply_subs_to_expr(subs, l);
            let r = apply_subs_to_expr(subs, r);
            let t = apply(subs, t);
            AnnotedExpr::BinOp(box l, *op, box r, t)
        },
        AnnotedExpr::Fun(ref arg_name, ref body, ref t) => {
            AnnotedExpr::Fun(arg_name.clone(), 
                             box apply_subs_to_expr(subs, body), 
                             apply(subs, t))
        },
        AnnotedExpr::App(ref fun, ref arg, ref t) => {
            let fun = apply_subs_to_expr(subs, fun);
            let arg = apply_subs_to_expr(subs, arg);
            let t = apply(subs, t);
            AnnotedExpr::App(box fun, box arg, t)
        }
    }
}

fn infer(env: &Enviroment, expr: &Expr, name_gen: &mut NameGenerator) -> Result<AnnotedExpr> {
    let annoted = annote(expr, &env, name_gen)?;
    let mut constraints = collect(&annoted)?;
    let subs = unify(&mut constraints)?;
    Ok(apply_subs_to_expr(&subs, &annoted))
}

/// A NameGenerator is responsible for generating unique names
struct NameGenerator {
    next: usize,
}
impl NameGenerator {
    fn new() -> NameGenerator {
        NameGenerator { next: 0 }
    }
    fn next_name(&mut self) -> String {
        self.next += 1;
        format!("t{}", self.next)
    }
}
/// An enum of errors that can occur
#[derive(Debug, Clone)]
enum Error {
    UndefinedName(String),
    TypeError(String),
}
/// A type allias to make type definitions shorter
type Result<T> = std::result::Result<T, Error>;

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
fn display_test() {
    println!("Type display test");
    println!("{}, {}, {}, {}", PrimitiveType::Bool, PrimitiveType::Num,
                               PrimitiveType::Var("a".to_owned()),
                               PrimitiveType::Fun(Box::new(PrimitiveType::Num),
                                                  Box::new(PrimitiveType::Num)));
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
    println!("\nExpr display tests");
    for (name, expr) in exprs {
        println!("{}: {}", name, expr);
    }
    let aexprs =  vec![
        ("Number", AnnotedExpr::Num(123, PrimitiveType::Num)),
        ("Bool", AnnotedExpr::Bool(true, PrimitiveType::Bool)),
        ("Var", AnnotedExpr::Var("x".to_owned(), PrimitiveType::Var("a".to_owned()))),
        ("BinOp", AnnotedExpr::BinOp(
            box AnnotedExpr::Num(1, PrimitiveType::Num),
            Op::Add,
            box AnnotedExpr::Num(2, PrimitiveType::Num),
            PrimitiveType::Num,
        )),
        ("BinOp 2", AnnotedExpr::BinOp(
            box AnnotedExpr::BinOp(
                box AnnotedExpr::Num(3, PrimitiveType::Num),
                Op::Add,
                box AnnotedExpr::Num(-2, PrimitiveType::Num),
                PrimitiveType::Num,
            ),
            Op::Mul,
            box AnnotedExpr::BinOp(
                box AnnotedExpr::Var("spam".to_owned(), PrimitiveType::Num),
                Op::Add,
                box AnnotedExpr::Num(42, PrimitiveType::Num),
                PrimitiveType::Num,
            ),
            PrimitiveType::Num,
        )),
        ("Function", AnnotedExpr::Fun(
            "x".to_owned(),
            box AnnotedExpr::BinOp(
                box AnnotedExpr::Var("x".to_owned(), PrimitiveType::Num),
                Op::Mul,
                box AnnotedExpr::Var("x".to_owned(), PrimitiveType::Num),
                PrimitiveType::Num,
            ),
            PrimitiveType::Fun(box PrimitiveType::Num, box PrimitiveType::Num)
        )),
    ];
    println!("\nAnnotedExpr display test");
    for (name, aexpr) in aexprs {
        println!("{}: {}", name, aexpr);
    }
}