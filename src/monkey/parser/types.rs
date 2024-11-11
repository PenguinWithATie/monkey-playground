use std::{fmt::Display, rc::Rc};

use crate::monkey::lexer::Token;
#[derive(Debug, PartialEq, Clone)]
pub struct Block(pub Vec<Expr>);

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Int(i64),
    Identifier(String),
    Array(Vec<Expr>),
    Bool(bool),
    String(String),
    Prefix(Box<Prefix>),
    Infix(Box<Infix>),
    If(If),
    Fn(Rc<Fn>),
    Call(Call),
    Let(Let),
    Return(Box<Expr>),
    Hash(Vec<(Expr, Expr)>),
}
#[derive(Debug, PartialEq, Clone)]
pub struct Prefix {
    pub token: Token,
    pub right: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Infix {
    pub left: Rc<Expr>,
    pub token: Token,
    pub right: Expr,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Let {
    pub name: String,
    pub value: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub condition: Box<Expr>,
    pub consequence: Block,
    pub alternative: Option<Block>,
}

#[derive(Debug, PartialEq)]
pub struct Fn {
    pub name: Option<String>,
    pub args: Vec<String>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub expr: Rc<Expr>, //function literal or identifier
    pub args: Vec<Expr>,
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = format!("{}(", self.expr);
        for arg in &self.args {
            ret += &format!("{}, ", arg);
        }
        ret += ")";
        write!(f, "{}", ret)
    }
}

impl Display for Fn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = format!(
            "fn({}) {{ {} }}",
            self.args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.body
        );
        write!(f, "{}", ret)
    }
}

impl Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = format!("if ({}) {{ {} }}\n", self.condition, self.consequence);
        if let Some(alternative) = &self.alternative {
            ret += &format!("\n else {{ {} }}", alternative)
        }
        write!(f, "{}", ret)
    }
}
impl Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let {} = {}", self.name, self.value)
    }
}
impl Display for Infix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}{}{})", self.left, self.token, self.right)
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.token, self.right)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Identifier(i) => write!(f, "{}", i),
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Prefix(p) => write!(f, "{}", p),
            Expr::Infix(i) => write!(f, "{}", i),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::If(i) => write!(f, "{}", i),
            Expr::Fn(fn_) => write!(f, "{}", fn_),
            Expr::Call(e) => write!(f, "as eval ({})", e),
            Expr::Let(l) => write!(f, "{}", l),
            Expr::Return(e) => write!(f, "{}", e),
            //Expr::Block(b) => write!(f, "{}", b),
            Expr::String(s) => write!(f, "{}", s),
            Expr::Array(a) => write!(
                f,
                "[{}]",
                a.iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Expr::Hash(h) => write!(
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

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = "".to_string();
        for s in &self.0 {
            ret += &format!("{s}");
        }
        write!(f, "{}", ret)
    }
}
