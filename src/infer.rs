use std::fmt::Display;
use std::fmt;
use std::collections::HashMap;
use ast::*;

/// An enum of errors that can occur
#[derive(Debug, Clone)]
pub enum Error {
    UndefinedName(String),
    TypeError(String),
}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Error::UndefinedName(ref name) => format!("{} is undefined", name),
            Error::TypeError(ref msg) => format!("Type error: {}", msg),
        })
    }
}

/// A type allias to make type definitions shorter
pub type Result<T> = ::std::result::Result<T, Error>;

/// A mapping between names and their types
#[derive(Debug, Clone)]
pub struct Enviroment<'a> {
    frame: HashMap<String, PrimitiveType>,
    prev: Option<&'a Enviroment<'a>>,
}
impl<'a> Enviroment<'a> {
    pub fn empty() -> Enviroment<'a> {
        Enviroment {
            frame: HashMap::new(),
            prev: None,
        }
    }
    pub fn new_frame(&'a self) -> Enviroment<'a> {
        Enviroment {
            frame: HashMap::new(),
            prev: Some(self)
        }
    }
    pub fn get(&self, key: String) -> Option<PrimitiveType> {
        if let Some(v) = self.frame.get(&key) {
            Some(v.clone())
        } else if let Some(prev) = self.prev {
            prev.get(key)
        } else {
            None
        }
    }
    pub fn insert(&mut self, key: String, val: &PrimitiveType) {
        self.frame.insert(key, val.clone());
    }
}

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
pub fn annote(e: &Expr, env: &mut Enviroment, name_gen: &mut NameGenerator) -> Result<AnnotedExpr> {
    match *e {
        Expr::Num(n) => Ok(AnnotedExpr::Num(n, PrimitiveType::Num)),
        Expr::Bool(b) => Ok(AnnotedExpr::Bool(b, PrimitiveType::Bool)),
        Expr::Var(ref name) => {
            if let Some(t) = env.get(name.clone()) {
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
            let mut new_env = env.new_frame();
            let t = PrimitiveType::Var(name_gen.next_name());
            new_env.insert(arg_name.clone(), &t);
            let annoted_body = annote(body, &mut new_env, name_gen)?;
            Ok(AnnotedExpr::Fun(arg_name.clone(), box annoted_body, 
                PrimitiveType::Fun(box t.clone(), box PrimitiveType::Var(name_gen.next_name()))))
        },
        Expr::App(ref func, ref arg) => {
            let func = annote(func, env, name_gen)?;
            let arg = annote(arg, env, name_gen)?;
            let new_name = name_gen.next_name();
            let new_type = PrimitiveType::Var(new_name);
            Ok(AnnotedExpr::App(box func, box arg, new_type))
        },
        Expr::Let(ref id, ref val, ref body) => {
            env.insert(id.clone(), &PrimitiveType::Var(name_gen.next_name()));
            let annoted_val = annote(val, env, name_gen)?;
            env.insert(id.clone(), &type_of(&annoted_val));
            let annoted_body = annote(body, env, name_gen)?;
            if let Some(t) = env.get(id.clone()) {
                Ok(AnnotedExpr::Let(id.clone(),
                   box annoted_val,
                   box annoted_body,
                   PrimitiveType::Var(name_gen.next_name())))
            } else {
                Err(Error::UndefinedName(id.clone()))
            }
        },
        Expr::If(ref pred, ref then, ref otherwise) => {
            let annoted_pred = annote(pred, env, name_gen)?;
            let annoted_then = annote(then, env, name_gen)?;
            let annoted_otherwise = annote(otherwise, env, name_gen)?;
            Ok(AnnotedExpr::If(
                box annoted_pred,
                box annoted_then,
                box annoted_otherwise,
                PrimitiveType::Var(name_gen.next_name())
            ))
        },
    } 
}

/// Returns the type of an AnnotedExpr
pub fn type_of(e: &AnnotedExpr) -> PrimitiveType {
    match *e {
        AnnotedExpr::Num(_, ref t)         |
        AnnotedExpr::Bool(_, ref t)        |
        AnnotedExpr::Var(_, ref t)         |
        AnnotedExpr::App(_, _, ref t)      |
        AnnotedExpr::BinOp(_, _, _, ref t) |
        AnnotedExpr::Fun(_, _, ref t)      |
        AnnotedExpr::Let(_, _, _, ref t)   |
        AnnotedExpr::If(_, _, _, ref t)     => t.clone() 
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
                Op::Add | Op::Mul | Op::Sub | Op::Div => {
                    vec![Constraint(left_type, PrimitiveType::Num),
                         Constraint(right_type, PrimitiveType::Num),
                         Constraint(res_type.clone(), PrimitiveType::Num)]
                },
                Op::Gt | Op::Lt | Op::Equal => vec![Constraint(left_type, right_type),
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
        },
        AnnotedExpr::Let(ref id, ref val, ref body, ref type_) => {
            let mut constraints = [collect(body)?, collect(val)?].concat();
            constraints.push(Constraint(type_of(body), type_.clone()));
            Ok(constraints)
        },
        AnnotedExpr::If(ref pred, ref then, ref otherwise, ref type_) => {
            let mut constraints = [collect(pred)?, collect(then)?, collect(otherwise)?].concat();
            constraints.push(Constraint(type_of(pred), PrimitiveType::Bool));
            constraints.push(Constraint(type_of(then), type_.clone()));
            constraints.push(Constraint(type_of(then), type_of(otherwise)));
            Ok(constraints)
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
        _ => Err(Error::TypeError(format!("mismatched types: {} != {}", t1, t2)))
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
        },
        AnnotedExpr::Let(ref id, ref val, ref body, ref t) => {
            AnnotedExpr::Let(id.clone(), 
                             box apply_subs_to_expr(subs, val),
                             box apply_subs_to_expr(subs, body), 
                             apply(subs, t))
        },
        AnnotedExpr::If(ref pred, ref then, ref otherwise, ref t) => {
            AnnotedExpr::If(
                box apply_subs_to_expr(subs, pred),
                box apply_subs_to_expr(subs, then),
                box apply_subs_to_expr(subs, otherwise), 
                apply(subs, t)
            )
        },
    }
}

pub fn infer(env: &mut Enviroment, expr: &Expr, name_gen: &mut NameGenerator) -> Result<AnnotedExpr> {
    let annoted = annote(expr, env, name_gen)?;
    let mut constraints = collect(&annoted)?;
    let subs = unify(&mut constraints)?;
    Ok(apply_subs_to_expr(&subs, &annoted))
}

/// A NameGenerator is responsible for generating unique names
pub struct NameGenerator {
    next: usize,
}
impl NameGenerator {
    pub fn new() -> NameGenerator {
        NameGenerator { next: 0 }
    }
    pub fn next_name(&mut self) -> String {
        self.next += 1;
        format!("t{}", self.next)
    }
}