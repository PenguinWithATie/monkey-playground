use crate::monkey::{
    lexer::Token,
    parser::{Block, Expr, Program},
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::iter::zip;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum Builtin {
    Len,
    First,
    Last,
    Rest,
    Push,
    Puts,
}

impl FromStr for Builtin {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "len" => Ok(Builtin::Len),
            "first" => Ok(Builtin::First),
            "last" => Ok(Builtin::Last),
            "rest" => Ok(Builtin::Rest),
            "push" => Ok(Builtin::Push),
            "puts" => Ok(Builtin::Puts),
            _ => Err("Invalid builtin".to_string()),
        }
    }
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Builtin::Len => write!(f, "BUILTIN len"),
            Builtin::First => write!(f, "BUILTIN first"),
            Builtin::Last => write!(f, "BUILTIN last"),
            Builtin::Rest => write!(f, "BUILTIN rest"),
            Builtin::Push => write!(f, "BUILTIN push"),
            Builtin::Puts => write!(f, "BUILTIN puts"),
        }
    }
}
impl Builtin {
    fn eval(self, args: &[Binding]) -> Result<Binding, &'static str> {
        match self {
            Builtin::Len => match args {
                [Binding::Primitive(String_(s))] => Ok(Binding::Primitive(Int(s.len() as i64))),
                [Binding::Array(a)] => Ok(Binding::Primitive(Int(a.len() as i64))),
                _ => Err("Expected single string or array for len builtin"),
            },
            Builtin::First => match args {
                [Binding::Array(a)] => {
                    if let Some(first) = a.first() {
                        Ok(first.clone())
                    } else {
                        Err("Expected array with at least one element for first builtin")
                    }
                }
                _ => Err("Expected array for first builtin"),
            },
            Builtin::Last => match args {
                [Binding::Array(a)] => {
                    if let Some(last) = a.last() {
                        Ok(last.clone())
                    } else {
                        Err("Expected array with at least one element for last builtin")
                    }
                }

                _ => Err("Expected array for last builtin"),
            },
            Builtin::Rest => match args {
                [Binding::Array(a)] => Ok(Binding::Array(a.iter().skip(1).cloned().collect())),
                _ => Err("Expected array for rest builtin"),
            },
            Builtin::Push => match args {
                [Binding::Array(a), new] => {
                    let mut ret = a.clone();
                    ret.push(new.clone());
                    Ok(Binding::Array(ret))
                }
                _ => Err("Expected array and int or string for push builtin"),
            },
            Builtin::Puts => {
                for arg in args {
                    logging::log!("{}", arg);
                }
                Ok(Binding::Null)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Primitive {
    Int(i64),
    String_(String),
    Bool(bool),
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Int(i) => write!(f, "{}", i),
            Primitive::Bool(b) => write!(f, "{}", b),
            Primitive::String_(s) => write!(f, "'{}'", s),
        }
    }
}

use crate::monkey::parser::Fn;
use leptos::logging;
use Primitive::*;
#[derive(Debug, Clone)]
pub enum Binding {
    Primitive(Primitive),
    Hash(HashMap<Primitive, Binding>),
    Array(Vec<Binding>),
    Fn(Rc<Env>, Rc<Fn>),
    //booleans
    Return(Box<Binding>),
    //null
    Null,
    Builtin(Builtin),
}
pub trait Evaluation {
    fn eval(&self, env: &Rc<Env>) -> Result<Binding, &'static str>;
}
#[derive(Debug)]
pub struct Env {
    pub local: RefCell<HashMap<String, Binding>>,
    pub enclosing: Option<Rc<Env>>,
}

impl Env {
    pub fn new(enclosing: Option<Env>) -> Self {
        let enclosing = enclosing.map(Rc::new);
        Self {
            local: RefCell::new(HashMap::new()),
            enclosing,
        }
    }
    pub fn get(&self, name: &str) -> Option<Binding> {
        if let Some(val) = self.local.borrow().get(name) {
            return Some(val.clone());
        }
        if let Some(enclosing) = self.enclosing.as_ref() {
            enclosing.get(name)
        } else {
            None
        }
    }
}
impl Default for Env {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::Primitive(Primitive::Int(i)) => write!(f, "{}", i),
            Binding::Primitive(Primitive::Bool(b)) => write!(f, "{}", b),
            Binding::Primitive(Primitive::String_(s)) => write!(f, "'{}'", s),
            Binding::Null => write!(f, "null"),
            Binding::Return(r) => write!(f, "{}", r),
            Binding::Builtin(b) => write!(f, "{}", b),
            Binding::Fn(_, fn_) => write!(f, "fn({}) {{ {} }}", fn_.args.join(", "), fn_.body),
            Binding::Array(a) => write!(
                f,
                "[{}]",
                a.iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Binding::Hash(h) => write!(
                f,
                "{{ {} }}",
                h.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl Evaluation for Expr {
    fn eval(&self, env: &Rc<Env>) -> Result<Binding, &'static str> {
        match self {
            Expr::Int(i) => Ok(Binding::Primitive(Int(*i))),
            Expr::Bool(b) => Ok(Binding::Primitive(Bool(*b))),
            Expr::String(s) => Ok(Binding::Primitive(String_(s.clone()))),
            Expr::Identifier(i) => {
                if let Some(binding) = env.get(i) {
                    Ok(binding.clone())
                } else if let Ok(builtin) = Builtin::from_str(i) {
                    Ok(Binding::Builtin(builtin))
                } else {
                    println!("{:?}", env.local);
                    Err("Undefined variable")
                }
            }
            Expr::Prefix(p) => match p.token {
                Token::Bang => {
                    let operand = p.right.eval(env)?;
                    match operand {
                        Binding::Primitive(Int(i)) => Ok(Binding::Primitive(Bool(i == 0))),
                        Binding::Primitive(Bool(b)) => Ok(Binding::Primitive(Bool(!b))),
                        _ => Err("Expected int or bool for ! operator"),
                    }
                }
                Token::Minus => {
                    let operand = p.right.eval(env)?;
                    match operand {
                        Binding::Primitive(Int(i)) => Ok(Binding::Primitive(Int(-i))),
                        _ => Err("Expected int for - operator"),
                    }
                }
                _ => Err("Expected bang"),
            },
            Expr::Infix(i) => {
                let left = i.left.eval(env)?;
                let right = i.right.eval(env)?;
                match (left, right) {
                    (Binding::Primitive(Int(l)), Binding::Primitive(Int(r))) => match i.token {
                        Token::Plus => Ok(Binding::Primitive(Int(l + r))),
                        Token::Minus => Ok(Binding::Primitive(Int(l - r))),
                        Token::Star => Ok(Binding::Primitive(Int(l * r))),
                        Token::Slash => Ok(Binding::Primitive(Int(l / r))),
                        Token::Eq => Ok(Binding::Primitive(Bool(l == r))),
                        Token::Neq => Ok(Binding::Primitive(Bool(l != r))),
                        Token::Lt => Ok(Binding::Primitive(Bool(l < r))),
                        Token::Gt => Ok(Binding::Primitive(Bool(l > r))),
                        Token::Percent => Ok(Binding::Primitive(Int(l % r))),
                        _ => Err("invald infix operator for int"),
                    },
                    (Binding::Primitive(Bool(l)), Binding::Primitive(Bool(r))) => match i.token {
                        Token::Eq => Ok(Binding::Primitive(Bool(l == r))),
                        Token::Neq => Ok(Binding::Primitive(Bool(l != r))),
                        _ => Err("invald infix operator for bool"),
                    },
                    (Binding::Primitive(String_(l)), Binding::Primitive(String_(r))) => {
                        match i.token {
                            Token::Eq => Ok(Binding::Primitive(Bool(l == r))),
                            Token::Neq => Ok(Binding::Primitive(Bool(l != r))),
                            Token::Plus => Ok(Binding::Primitive(String_(format!("{}{}", l, r)))),
                            _ => Err("invald infix operator for string"),
                        }
                    }
                    (Binding::Array(l), Binding::Primitive(Int(r))) => match i.token {
                        Token::LBracket => {
                            if r < 0 {
                                Err("Expected positive index for array")
                            } else if r as usize >= l.len() {
                                Err("Expected index less than array length")
                            } else {
                                Ok(l[r as usize].clone())
                            }
                        }
                        _ => Err("Invalid infix operator for array"),
                    },
                    (Binding::Hash(l), Binding::Primitive(key)) => {
                        if let Some(key) = l.get(&key) {
                            Ok(key.clone())
                        } else {
                            Err("Invalid key for hash")
                        }
                    }
                    _ => Err("invalid infix operator and types"),
                }
            }
            Expr::If(i) => {
                let condition = i.condition.eval(env)?;
                logging::log!("condition: {:?}", condition);
                match condition {
                    Binding::Primitive(Bool(true)) | Binding::Primitive(Int(_)) => {
                        i.consequence.eval(env)
                    }
                    _ => match &i.alternative {
                        Some(alternative) => alternative.eval(env),
                        None => Ok(Binding::Null),
                    },
                }
            }
            Expr::Fn(f) => Ok(Binding::Fn(env.clone(), f.clone())),
            Expr::Call(c) => {
                let fn_ = c.expr.eval(env)?;
                let mut args = Vec::new();
                for arg in &c.args {
                    args.push(arg.eval(env)?);
                }
                if let Binding::Fn(fn_env, fn_) = fn_ {
                    let fn_env = Env {
                        local: fn_env.local.clone(),
                        enclosing: Some(env.clone()),
                    };
                    zip(fn_.args.clone(), args).for_each(|(name, arg)| {
                        fn_env.local.borrow_mut().insert(name, arg);
                    });
                    fn_.body.eval(&Rc::new(fn_env))
                } else if let Binding::Builtin(builtin) = fn_ {
                    builtin.eval(args.as_slice())
                } else {
                    Err("Expected function literal or identifier")
                }
            }
            Expr::Return(e) => {
                let boxed = Box::new(e.eval(env)?);
                Ok(Binding::Return(boxed))
            }
            Expr::Let(l) => {
                let value = l.value.eval(env)?;
                println!("Let binding {} {}", l.name, value);
                env.local.borrow_mut().insert(l.name.clone(), value.clone());
                Ok(value)
            }
            Expr::Array(a) => {
                let mut array = Vec::new();
                for elem in a {
                    array.push(elem.eval(env)?);
                }
                Ok(Binding::Array(array))
            }
            //Expr::Block(b) => b.eval(env),
            Expr::Hash(h) => {
                let mut hash = HashMap::new();
                for (key, value) in h {
                    let key = key.eval(env)?;
                    let value = value.eval(env)?;
                    if let Binding::Primitive(key) = key {
                        hash.insert(key, value);
                    }
                }
                Ok(Binding::Hash(hash))
            }
        }
    }
}

impl Evaluation for Vec<Expr> {
    fn eval(&self, env: &Rc<Env>) -> Result<Binding, &'static str> {
        let mut bind = Binding::Null;
        for s in self {
            bind = s.eval(env)?;
            if matches!(bind, Binding::Return(_)) {
                break;
            }
        }
        Ok(bind)
    }
}
impl Evaluation for Block {
    fn eval(&self, env: &Rc<Env>) -> Result<Binding, &'static str> {
        self.0.eval(env)
    }
}

impl Evaluation for Program {
    fn eval(&self, env: &Rc<Env>) -> Result<Binding, &'static str> {
        self.statements.eval(env)
    }
}
