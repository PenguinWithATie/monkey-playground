use std::{collections::HashMap, fmt::Display, str::FromStr};

#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Op {
    Constant = 0x00,
    Pop = 0x01,
    Add = 0x02,
    Sub = 0x03,
    Mul = 0x04,
    Div = 0x05,
    True = 0x06,
    False = 0x07,
    Eq = 0x08,
    Neq = 0x09,
    Lt = 0x0A,
    Minus = 0x0B,
    Bang = 0x0C,
    Jmp = 0x0D,
    JmpIfFalse = 0x0E,
    Null = 0x0F,
    SetGlobal = 0x10,
    GetGlobal = 0x11,
    Array = 0x12,
    Hash = 0x13,
    Index = 0x14,
    ReturnVal = 0x15,
    Call = 0x16,
    Return = 0x17,
    SetLocal = 0x18,
    GetLocal = 0x19,
    GetBuiltin = 0x1A,
    Closure = 0x1B,
    GetFree = 0x1C,
    CurrentClosure = 0x1D,
    Mod = 0x1E,
}

impl From<u8> for Op {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => Op::Constant,
            0x01 => Op::Pop,
            0x02 => Op::Add,
            0x03 => Op::Sub,
            0x04 => Op::Mul,
            0x05 => Op::Div,
            0x06 => Op::True,
            0x07 => Op::False,
            0x08 => Op::Eq,
            0x09 => Op::Neq,
            0x0A => Op::Lt,
            0x0B => Op::Minus,
            0x0C => Op::Bang,
            0x0D => Op::Jmp,
            0x0E => Op::JmpIfFalse,
            0x0F => Op::Null,
            0x10 => Op::SetGlobal,
            0x11 => Op::GetGlobal,
            0x12 => Op::Array,
            0x13 => Op::Hash,
            0x14 => Op::Index,
            0x15 => Op::ReturnVal,
            0x16 => Op::Call,
            0x17 => Op::Return,
            0x18 => Op::SetLocal,
            0x19 => Op::GetLocal,
            0x1A => Op::GetBuiltin,
            0x1B => Op::Closure,
            0x1C => Op::GetFree,
            0x1D => Op::CurrentClosure,
            0x1E => Op::Mod,
            _ => panic!("Opcode not found"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub op: Op,
    param: Vec<u8>,
}

impl Instruction {
    pub fn new(op: Op) -> Self {
        Self {
            op,
            param: Vec::new(),
        }
    }
    pub fn new_u16(op: Op, param: u16) -> Self {
        Self {
            op,
            param: param.to_be_bytes().to_vec(),
        }
    }
    pub fn new_u8(op: Op, param: u8) -> Self {
        Self {
            op,
            param: param.to_be_bytes().to_vec(),
        }
    }
    pub fn new_u16_u8(op: Op, param1: u16, param2: u8) -> Self {
        let mut param = param1.to_be_bytes().to_vec();
        param.extend(param2.to_be_bytes());
        Self { op, param }
    }
    pub fn bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.op as u8);
        bytes.extend(self.param.iter());
        bytes
    }
    pub fn len(&self) -> usize {
        self.param.len() + 1
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.op {
            Op::Constant => write!(
                f,
                "CONSTANT {}",
                u16::from_be_bytes([self.param[0], self.param[1]])
            ),
            Op::Add => write!(f, "ADD"),
            Op::Sub => write!(f, "SUB"),
            Op::Pop => write!(f, "POP"),
            Op::Mul => write!(f, "MUL"),
            Op::Div => write!(f, "DIV"),
            Op::True => write!(f, "TRUE"),
            Op::False => write!(f, "FALSE"),
            Op::Eq => write!(f, "EQ"),
            Op::Neq => write!(f, "NEQ"),
            Op::Lt => write!(f, "LT"),
            Op::Minus => write!(f, "MINUS"),
            Op::Bang => write!(f, "BANG"),
            Op::Jmp => write!(
                f,
                "JMP {}",
                u16::from_be_bytes([self.param[0], self.param[1]])
            ),
            Op::JmpIfFalse => write!(
                f,
                "JMP_FALSE {}",
                u16::from_be_bytes([self.param[0], self.param[1]])
            ),
            Op::Null => write!(f, "NULL"),
            Op::SetGlobal => write!(
                f,
                "SET_GLOBAL {}",
                u16::from_be_bytes([self.param[0], self.param[1]])
            ),
            Op::GetGlobal => write!(
                f,
                "GET_GLOBAL {}",
                u16::from_be_bytes([self.param[0], self.param[1]])
            ),
            Op::Array => write!(f, "ARRAY {}", self.param[0]),
            Op::Hash => write!(f, "HASH {}", self.param[0]),
            Op::Index => write!(f, "INDEX"),
            Op::ReturnVal => write!(f, "RETURN_VAL"),
            Op::Call => write!(f, "CALL {}", self.param[0]),
            Op::Return => write!(f, "RETURN"),
            Op::SetLocal => write!(
                f,
                "SET_LOCAL {}",
                u16::from_be_bytes([self.param[0], self.param[1]])
            ),
            Op::GetLocal => write!(
                f,
                "GET_LOCAL {}",
                u16::from_be_bytes([self.param[0], self.param[1]])
            ),
            Op::GetBuiltin => write!(f, "GET_BUILTIN {}", u8::from_be_bytes([self.param[0]])),
            Op::Closure => write!(
                f,
                "CLOSURE {} {}",
                u16::from_be_bytes([self.param[0], self.param[1]]),
                u8::from_be_bytes([self.param[2]])
            ),
            Op::GetFree => write!(f, "GET_FREE {}", u8::from_be_bytes([self.param[0]])),
            Op::CurrentClosure => write!(f, "CURRENT_CLOSURE"),
            Op::Mod => write!(f, "MOD"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub fn_: CompiledFn,
    pub free: Vec<Binding>,
}

impl Closure {
    pub fn new(fn_: CompiledFn, free: Vec<Binding>) -> Self {
        Self { fn_, free }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Builtin {
    Len,
    First,
    Last,
    Rest,
    Push,
    Puts,
}

impl Display for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Builtin::Len => write!(f, "len"),
            Builtin::First => write!(f, "first"),
            Builtin::Last => write!(f, "last"),
            Builtin::Rest => write!(f, "rest"),
            Builtin::Push => write!(f, "push"),
            Builtin::Puts => write!(f, "puts"),
        }
    }
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

impl From<u8> for Builtin {
    fn from(i: u8) -> Self {
        match i {
            0 => Builtin::Len,
            1 => Builtin::First,
            2 => Builtin::Last,
            3 => Builtin::Rest,
            4 => Builtin::Push,
            5 => Builtin::Puts,
            _ => panic!("Invalid builtin"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompiledFn {
    pub body: Vec<u8>,
    pub num_locals: u16,
    pub num_args: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Primitive {
    Int(i64),
    String_(String),
    Bool(bool),
    Fn(CompiledFn),
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Int(i) => write!(f, "{}", i),
            Primitive::Bool(b) => write!(f, "{}", b),
            Primitive::String_(s) => write!(f, "'{}'", s),
            Primitive::Fn(fn_) => {
                write!(f, "fn_bytes{{")?;
                for b in &fn_.body {
                    write!(f, "{},", b)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl From<Primitive> for Binding {
    fn from(p: Primitive) -> Self {
        Binding::Primitive(p)
    }
}

#[derive(Debug, Clone)]
pub enum Binding {
    Primitive(Primitive),
    Hash(HashMap<Primitive, Binding>),
    Array(Vec<Binding>),
    Null,
    Builtin(Builtin),
    Closure(Closure),
}

impl Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binding::Primitive(p) => write!(f, "{}", p),
            Binding::Hash(h) => {
                write!(f, "{{")?;
                for (k, v) in h {
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Binding::Array(a) => {
                write!(f, "[")?;
                for elem in a {
                    write!(f, "{},", elem)?;
                }
                write!(f, "]")
            }
            Binding::Null => write!(f, "null"),
            Binding::Builtin(b) => write!(f, "{}", b),
            Binding::Closure(c) => {
                write!(f, "closure[")?;
                write!(f, "fn_bytes{{")?;
                for b in &c.fn_.body {
                    write!(f, "{},", b)?;
                }
                write!(f, "}}, free: ")?;
                for b in &c.free {
                    write!(f, "{},", b)?;
                }
                write!(f, "]")
            }
        }
    }
}
