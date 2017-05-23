use std::fmt::Display;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Op {
    Add,
    Mul,
    Sub,
    Div,
    Equal,
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
            Op::Sub => "-",
            Op::Div => "/",
            Op::Equal => "=",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Num(i32),
    Bool(bool),
    Var(String),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Fun(String, Box<Expr>),
    App(Box<Expr>,  Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
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
            Expr::Let(ref id, ref val, ref body) => format!("(let {} = {} in {})", id, val, body),
            Expr::If(ref pred, ref then, ref otherwise) => format!("(if {} then {} else {})", pred, then, otherwise),
    })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotedExpr {
    Num(i32, PrimitiveType),
    Bool(bool, PrimitiveType),
    Var(String, PrimitiveType),
    BinOp(Box<AnnotedExpr>, Op, Box<AnnotedExpr>, PrimitiveType),
    Fun(String, Box<AnnotedExpr>, PrimitiveType),
    App(Box<AnnotedExpr>,  Box<AnnotedExpr>, PrimitiveType),
    Let(String, Box<AnnotedExpr>, Box<AnnotedExpr>, PrimitiveType),
    If(Box<AnnotedExpr>, Box<AnnotedExpr>, Box<AnnotedExpr>, PrimitiveType),
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
            AnnotedExpr::Let(ref id, ref val, ref body, ref type_) => format!("(let {} = {} in {}) : {}", id, val, body, type_),
            AnnotedExpr::If(ref pred, ref then, ref otherwise, ref type_) => format!("(if {} then {} else {}) : {}", pred, then, otherwise, type_),
        })
    }
}