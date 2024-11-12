use super::types::{Builtin, Closure, CompiledFn, Instruction, Op, Primitive};
use crate::monkey::{
    lexer::Token,
    parser::{Block, Expr, Program},
};
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
enum SymbolScope {
    Global,
    Local,
    Builtin,
    Free,
    Function,
}

#[derive(Debug, Clone)]
struct Symbol {
    name: String,
    index: u16,
    scope: SymbolScope,
}

#[derive(Debug, Default, Clone)]
struct SymbolTable {
    outer: Option<Box<SymbolTable>>,
    symbols: HashMap<String, Symbol>,
    num_definitions: usize,
    free: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new(outer: SymbolTable) -> Self {
        Self {
            outer: Some(Box::new(outer)),
            symbols: HashMap::new(),
            num_definitions: 0,
            free: Vec::new(),
        }
    }
    pub fn define(&mut self, name: String) -> Symbol {
        let index = self.num_definitions as u16;
        self.num_definitions += 1;
        let scope = if self.outer.is_some() {
            SymbolScope::Local
        } else {
            SymbolScope::Global
        };
        let symbol = Symbol {
            name: name.clone(),
            index,
            scope,
        };
        self.symbols.insert(name, symbol.clone());
        symbol
    }
    pub fn define_free(&mut self, original: Symbol) -> Symbol {
        let symbol = Symbol {
            name: original.name.clone(),
            index: self.free.len() as u16,
            scope: SymbolScope::Free,
        };
        self.free.push(original);
        self.symbols.insert(symbol.name.clone(), symbol.clone());
        symbol
    }
    pub fn define_fn(&mut self, name: String) {
        let symbol = Symbol {
            name: name.clone(),
            index: 23,
            scope: SymbolScope::Function,
        };
        self.symbols.insert(name, symbol);
    }
    pub fn resolve(&mut self, name: &str) -> Option<Symbol> {
        if let Ok(builtin) = Builtin::from_str(name) {
            Some(Symbol {
                name: name.to_string(),
                index: builtin as u16,
                scope: SymbolScope::Builtin,
            })
        } else if let Some(val) = self.symbols.get(name) {
            Some(val.clone())
        } else if let Some(outer) = &mut self.outer {
            if let Some(val) = outer.resolve(name) {
                if matches!(val.scope, SymbolScope::Global | SymbolScope::Builtin) {
                    Some(val)
                } else {
                    Some(self.define_free(val))
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Default)]
struct Scope {
    instructions: Vec<Instruction>,
    byte_size: usize,
}

impl Scope {
    fn set_return(&mut self) {
        if let Some(last) = self.instructions.last_mut() {
            if last.op == Op::Pop {
                last.op = Op::ReturnVal;
            }
        } else {
            self.instructions.push(Instruction::new(Op::Return));
        }
    }
    fn remove_last_pop(&mut self) {
        if let Some(last) = self.instructions.last() {
            if last.op == Op::Pop {
                self.byte_size -= last.len();
                self.instructions.pop();
            }
        }
    }
}

pub struct CompiledContext {
    symbols: SymbolTable,
    scopes: Vec<Scope>,
    constants: Vec<Primitive>,
}

impl Default for CompiledContext {
    fn default() -> Self {
        Self {
            symbols: SymbolTable::default(),
            constants: Vec::new(),
            scopes: vec![Scope::default()],
        }
    }
}

impl CompiledContext {
    pub fn make_main_closure(&mut self) -> Closure {
        let num_locals = self.symbols.num_definitions as u16;
        let body = self.to_bytes();
        Closure::new(
            CompiledFn {
                body,
                num_locals,
                num_args: 0,
            },
            Vec::new(),
        )
    }
    pub fn get_constants(&self) -> Vec<Primitive> {
        self.constants.clone()
    }
    pub fn remove_last_pop(&mut self) {
        if let Some(last) = self.scopes.last() {
            if last.instructions.last().unwrap().op == Op::Pop {
                self.scopes.last_mut().unwrap().instructions.pop();
            }
        }
    }
    pub fn clear_instructions(&mut self) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.instructions.clear();
            scope.byte_size = 0;
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for i in self.scopes.last().unwrap().instructions.clone() {
            bytes.extend(i.bytes());
        }
        bytes
    }
    pub fn print_instructions(&self) {
        let mut len = 0;
        for i in &self.scopes.last().unwrap().instructions {
            println!("{}: {}", len, i);
            len += i.len();
        }
    }
    fn instructions_len(&self) -> usize {
        self.scopes.last().unwrap().instructions.len()
    }
    fn instructions_size(&self) -> usize {
        self.scopes.last().unwrap().byte_size
    }
    fn set_nth_instruction(&mut self, index: usize, i: Instruction) {
        self.scopes.last_mut().unwrap().instructions[index] = i;
    }
    fn emit(&mut self, i: Instruction) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.byte_size += i.len();
            scope.instructions.push(i);
        } else {
            panic!("No scope to emit to");
        }
    }
    fn emit_infix(&mut self, token: &Token) {
        self.emit(Instruction::new(match token {
            Token::Plus => Op::Add,
            Token::Minus => Op::Sub,
            Token::Star => Op::Mul,
            Token::Slash => Op::Div,
            Token::Eq => Op::Eq,
            Token::Neq => Op::Neq,
            Token::Lt => Op::Lt,
            Token::LBracket => Op::Index,
            Token::Percent => Op::Mod,
            _ => panic!("Invalid infix token"),
        }));
    }
    fn emit_symbol(&mut self, symbol: Symbol) {
        match symbol.scope {
            SymbolScope::Global => self.emit(Instruction::new_u16(Op::GetGlobal, symbol.index)),
            SymbolScope::Local => self.emit(Instruction::new_u16(Op::GetLocal, symbol.index)),
            SymbolScope::Builtin => {
                self.emit(Instruction::new_u8(Op::GetBuiltin, symbol.index as u8))
            }
            SymbolScope::Free => self.emit(Instruction::new_u8(Op::GetFree, symbol.index as u8)),
            SymbolScope::Function => self.emit(Instruction::new(Op::CurrentClosure)),
        }
    }
    fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
        self.symbols = SymbolTable::new(self.symbols.clone());
    }
    //exiting a scope returns the bytes
    fn exit_scope(&mut self, set_return: bool) -> Vec<u8> {
        let mut scope = self.scopes.pop().unwrap();
        if set_return {
            scope.set_return();
        }
        if let Some(outer) = &self.symbols.outer {
            self.symbols = *outer.clone();
        }
        scope
            .instructions
            .into_iter()
            .flat_map(|i| i.bytes())
            .collect()
    }
}
pub trait Compilation {
    fn compile(&self, output: &mut CompiledContext);
}

impl Compilation for Block {
    fn compile(&self, out: &mut CompiledContext) {
        for s in &self.0 {
            s.compile(out);
            if !matches!(s, Expr::Return(_)) {
                out.emit(Instruction::new(Op::Pop));
            }
        }
    }
}

impl Compilation for Program {
    fn compile(&self, out: &mut CompiledContext) {
        for s in &self.statements {
            s.compile(out);
            if !matches!(s, Expr::Return(_)) {
                out.emit(Instruction::new(Op::Pop));
            }
        }
    }
}

impl Compilation for Expr {
    fn compile(&self, out: &mut CompiledContext) {
        match self {
            Expr::Int(i) => {
                out.constants.push(Primitive::Int(*i));
                out.emit(Instruction::new_u16(
                    Op::Constant,
                    out.constants.len() as u16 - 1,
                ));
            }
            Expr::Bool(b) => {
                if *b {
                    out.emit(Instruction::new(Op::True));
                } else {
                    out.emit(Instruction::new(Op::False));
                }
            }
            Expr::String(s) => {
                out.constants.push(Primitive::String_(s.clone()));
                out.emit(Instruction::new_u16(
                    Op::Constant,
                    out.constants.len() as u16 - 1,
                ));
            }
            Expr::Fn(f) => {
                out.enter_scope();
                if let Some(name) = &f.name {
                    out.symbols.define_fn(name.clone());
                }
                for arg in &f.args {
                    let symbol = out.symbols.define(arg.clone());
                    out.emit_symbol(symbol);
                }
                f.body.compile(out);
                let num_locals = out.symbols.num_definitions as u16;
                let free = out.symbols.free.clone();
                let num_free = free.len() as u8;
                let body = out.exit_scope(true);
                out.constants.push(Primitive::Fn(CompiledFn {
                    body,
                    num_locals,
                    num_args: f.args.len() as u8,
                }));
                for symbol in free {
                    out.emit_symbol(symbol);
                }
                out.emit(Instruction::new_u16_u8(
                    Op::Closure,
                    out.constants.len() as u16 - 1,
                    num_free,
                ));
            }
            Expr::Infix(i) => match i.token {
                Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Eq
                | Token::Neq
                | Token::Lt
                | Token::LBracket
                | Token::Percent => {
                    i.left.compile(out);
                    i.right.compile(out);
                    out.emit_infix(&i.token);
                }
                Token::Gt => {
                    i.right.compile(out);
                    i.left.compile(out);
                    out.emit(Instruction::new(Op::Lt));
                }
                _ => panic!("Invalid infix token"),
            },
            Expr::Prefix(p) => {
                p.right.compile(out);
                match p.token {
                    Token::Bang => {
                        out.emit(Instruction::new(Op::Bang));
                    }
                    Token::Minus => {
                        out.emit(Instruction::new(Op::Minus));
                    }
                    _ => unreachable!(),
                }
            }
            Expr::If(i) => {
                //Condition
                i.condition.compile(out);
                //Consequence
                out.emit(Instruction::new_u16(Op::JmpIfFalse, 2323));
                let jmp_false_pos = out.instructions_len() - 1;
                i.consequence.compile(out);
                out.scopes.last_mut().unwrap().remove_last_pop();
                //Alternative
                out.emit(Instruction::new_u16(Op::Jmp, 2323));
                let jump_pos: usize = out.instructions_len() - 1;
                out.set_nth_instruction(
                    jmp_false_pos,
                    Instruction::new_u16(Op::JmpIfFalse, out.instructions_size() as u16),
                );
                if let Some(alternative) = &i.alternative {
                    alternative.compile(out);
                    out.scopes.last_mut().unwrap().remove_last_pop();
                } else {
                    out.emit(Instruction::new(Op::Null));
                }
                out.set_nth_instruction(
                    jump_pos,
                    Instruction::new_u16(Op::Jmp, out.instructions_size() as u16),
                );
            }
            Expr::Let(l) => {
                let symbol = out.symbols.define(l.name.clone());
                l.value.compile(out);

                let op = if symbol.scope == SymbolScope::Global {
                    Op::SetGlobal
                } else {
                    Op::SetLocal
                };
                out.emit(Instruction::new_u16(op, symbol.index));
            }
            Expr::Identifier(i) => {
                let symbol = out.symbols.resolve(i).expect("Undefined variable");
                out.emit_symbol(symbol);
            }
            Expr::Array(a) => {
                for elem in a {
                    elem.compile(out);
                }
                out.emit(Instruction::new_u16(Op::Array, a.len() as u16));
            }
            Expr::Hash(h) => {
                for (key, value) in h {
                    key.compile(out);
                    value.compile(out);
                }
                out.emit(Instruction::new_u16(Op::Hash, h.len() as u16));
            }
            Expr::Return(e) => {
                e.compile(out);
                out.emit(Instruction::new(Op::ReturnVal));
            }
            Expr::Call(c) => {
                c.expr.compile(out);
                for arg in &c.args {
                    arg.compile(out);
                }
                out.emit(Instruction::new_u8(Op::Call, c.args.len() as u8));
            }
        }
    }
}
