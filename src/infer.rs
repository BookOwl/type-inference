use std::fmt::Display;
use std::fmt;
use std::collections::HashSet;
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
pub enum Enviroment {
    Empty,
    Frame(String, TypeScheme, Box<Enviroment>),
}
impl Enviroment {
    pub fn empty() -> Enviroment {
        Enviroment::Empty
    }
    pub fn extend(&self, n: String, t: TypeScheme) -> Enviroment {
        Enviroment::Frame(n, t, box self.clone())
    }
    pub fn lookup(&self, key: &str) -> Option<TypeScheme> {
        match *self {
            Enviroment::Empty => None,
            Enviroment::Frame(ref n, ref s, ref p) => {
                if n == key {
                    Some(s.clone())
                } else {
                    p.lookup(key)
                }
            }
        }
    }
    pub fn schemes(&self) -> Vec<TypeScheme> {
        let mut schemes = vec![];
        let mut s = self;
        loop {
            match *s {
                Enviroment::Empty => return schemes,
                Enviroment::Frame(_, ref scm, ref p) => {
                    schemes.push(scm.clone());
                    s = p;
                }
            }
        }
    }
    pub fn type_vars(&self) -> HashSet<u32> {
        let mut vars = HashSet::new();
        for scm in &self.schemes() {
            vars = vars.union(&scm.type_vars()).cloned().collect();
        }
        vars
    }
}

#[derive(Debug, Clone)]
pub struct Subst {
    prev: Option<Box<Subst>>,
    x: Option<PrimitiveType>,
    t: Option<PrimitiveType>,
}
impl Subst {
    pub fn empty() -> Subst {
        Subst {
            prev: None,
            x: None,
            t: None,
        }
    }
    pub fn extend(&self, x: PrimitiveType, t: PrimitiveType) -> Subst {
        Subst {
            prev: Some(box self.clone()),
            x: Some(x),
            t: Some(t),
        }
    }
    pub fn lookup(&self, y: &PrimitiveType) -> PrimitiveType {
        if let Some(ref prev) = self.prev {
            if self.x == Some(y.clone()) { self.t.clone().unwrap() } else {prev.lookup(y)}
        } else {
            y.clone()
        }
    }
    pub fn apply(&self, t: &PrimitiveType) -> PrimitiveType {
        match *t {
            ref tv @ PrimitiveType::Var(_) => {
                let u = self.lookup(tv);
                if *t == u { t.clone() } else { self.apply(&u) }
            },
            PrimitiveType::Fun(ref a, ref r) => PrimitiveType::Fun(box self.apply(a), box self.apply(r)),
            PrimitiveType::Con(ref name, ref typs) => PrimitiveType::Con(name.clone(), typs.iter().map(|t| self.apply(t)).collect())
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeScheme {
    typ: PrimitiveType,
    vars: HashSet<u32>,
}
impl TypeScheme {
    pub fn new(typ: PrimitiveType, vars: HashSet<u32>) -> TypeScheme {
        TypeScheme{typ, vars}
    }
    pub fn new_instance(&self, var_gen: &mut VarGenerator) -> PrimitiveType {
        self.vars.iter().fold(Subst::empty(), |sub, var| sub.extend(PrimitiveType::Var(*var), 
                                                                    var_gen.next_typevar())).apply(&self.typ)
    }
    pub fn type_vars(&self) -> HashSet<u32> {
        self.typ.type_vars().difference(&self.vars).cloned().collect()
    }
    pub fn from_type(t: &PrimitiveType, env: &Enviroment) -> TypeScheme {
        TypeScheme::new(t.clone(), t.type_vars().difference(&env.type_vars()).cloned().collect())
    }
}

fn mgu(t: &PrimitiveType, u: &PrimitiveType, s: &Subst) -> Result<Subst> {
    use ast::PrimitiveType::*;
    match (s.apply(t), s.apply(u)) {
        (Var(a), Var(b)) if a == b => Ok(s.clone()),
        (Var(a), _) if !u.type_vars().contains(&a) => Ok(s.extend(Var(a), u.clone())),
        (_, Var(_)) => mgu(u, t, s),
        (Fun(ref t1, ref t2), Fun(ref u1, ref u2)) => mgu(t1, u1, &mgu(t2, u2, s)?),
        (Con(ref n1, ref ts), Con(ref n2, ref us)) if n1 == n2 => {
            Ok(ts.iter().zip(us).fold(s.clone(), |ref acc, (ref a, ref b)| mgu(a, b, acc).unwrap())) // FIXME
        },
        (ref a, ref b) => Err(Error::TypeError(format!("cannot unify {} with {}", a, b)))
    }
}

fn tp(exp: &Expr, t: &PrimitiveType, env: &Enviroment, s: &Subst, var_gen: &mut VarGenerator) -> Result<Subst> {
    match *exp {
        Expr::Var(ref n) => {
            if let Some(scm) = env.lookup(n) {
                mgu(&scm.new_instance(var_gen), t, s)
            } else {
                Err(Error::UndefinedName(format!("{} is undefined!", n)))
            }
        },
        Expr::Fun(ref arg, ref body) => {
            let a = var_gen.next_typevar();
            let b = var_gen.next_typevar();
            let s1 = mgu(t, &PrimitiveType::Fun(box a.clone(), box b.clone()), s)?;
            let env1 = env.extend(arg.clone(), TypeScheme::new(a.clone(), HashSet::new()));
            tp(body, &b, &env1, &s1, var_gen)
        },
        Expr::App(ref e1, ref e2) => {
            let a = var_gen.next_typevar();
            let s1 = tp(e1, &PrimitiveType::Fun(box a.clone(), box t.clone()), env, s, var_gen)?;
            tp(e2, &a, env, &s1, var_gen)
        },
        Expr::Let(ref x, ref e1, ref e2) => {
            let a = var_gen.next_typevar();
            let s1 = tp(e1, &a.clone(), env, s, var_gen)?;
            let env2 = env.extend(x.clone(), TypeScheme::from_type(&s1.apply(&a), env));
            tp(e2, t, &env2, &s1, var_gen)
        },
        Expr::LetRec(ref x, ref e1, ref e2) => {
            let a = var_gen.next_typevar();
            let env1 = env.extend(x.clone(), TypeScheme::from_type(&a, env));
            let s1 = tp(e1, &a.clone(), &env1, s, var_gen)?;
            let env2 = env.extend(x.clone(), TypeScheme::from_type(&s1.apply(&a), env));
            tp(e2, t, &env2, &s1, var_gen)
        },
        Expr::Num(_) => mgu(t, &int_type(), s),
        Expr::Bool(_) => mgu(t, &bool_type(), s),
        Expr::BinOp(ref l, ref op, ref r) => {
            let (expected_l_type, op_type, expected_r_type) = match *op {
                Op::Add | Op::Sub | Op::Div | Op::Mul => (int_type(), int_type(), int_type()),
                Op::And | Op::Or => (bool_type(), bool_type(), bool_type()),
                Op::Lt | Op::Gt | Op::Equal => {
                    let a = var_gen.next_typevar();
                    (a.clone(), bool_type(), a.clone())
                },
            };
            let s1 = tp(l, &expected_l_type, env, s, var_gen)?;
            let s2 = tp(r, &expected_r_type, env, &s1, var_gen)?;
            mgu(t, &op_type, &s2)
        },
        Expr::If(ref pred, ref then, ref otherwise) => {
            let s1 = tp(pred, &bool_type(), env, s, var_gen)?;
            let s2 = tp(then, t, env, &s1, var_gen)?;
            tp(otherwise, t, env, &s2, var_gen)
        }
    }
}

pub fn type_of(expr: &Expr, env: &Enviroment, var_gen: &mut VarGenerator) -> Result<PrimitiveType> {
    let a = var_gen.next_typevar();
    let s = tp(expr, &a, env, &Subst::empty(), var_gen)?;
    Ok(s.apply(&a))
}

pub fn top_level_env(var_gen: &mut VarGenerator) -> Enviroment {
    let a = var_gen.next_typevar();
    let abstract_list = list_type(a);
    let env = Enviroment::Empty;
    let env = env.extend("nil".to_owned(), TypeScheme::from_type(&abstract_list.clone(), &env));
    let a = var_gen.next_typevar();
    let pair = PrimitiveType::Fun(box a.clone(), box PrimitiveType::Fun(box list_type(a.clone()), box list_type(a)));
    let env = env.extend("pair".to_owned(), TypeScheme::from_type(&pair, &env));
    let a = var_gen.next_typevar();
    let first = PrimitiveType::Fun(box list_type(a.clone()), box a.clone());
    let env = env.extend("first".to_owned(), TypeScheme::from_type(&first, &env));
    let a = var_gen.next_typevar();
    let rest = PrimitiveType::Fun(box list_type(a.clone()), box list_type(a.clone()));
    let env = env.extend("rest".to_owned(), TypeScheme::from_type(&rest, &env));
    let a = var_gen.next_typevar();
    let is_nil = PrimitiveType::Fun(box list_type(a.clone()), box bool_type());
    let env = env.extend("is_nil".to_owned(), TypeScheme::from_type(&is_nil, &env));
    env
}

pub fn int_type() -> PrimitiveType {
    PrimitiveType::Con("int".to_owned(), vec![])
}
pub fn bool_type() -> PrimitiveType {
    PrimitiveType::Con("bool".to_owned(), vec![])
}
pub fn list_type(t: PrimitiveType) -> PrimitiveType {
    PrimitiveType::Con("List".to_owned(), vec![t])
}

#[derive(Debug, Clone)]
pub struct VarGenerator {
    next_var: u32
}
impl VarGenerator {
    pub fn new() -> VarGenerator {
        VarGenerator { next_var: 0 }
    }
    pub fn next_typevar(&mut self) -> PrimitiveType {
        self.next_var += 1;
        PrimitiveType::Var(self.next_var)
    }
}