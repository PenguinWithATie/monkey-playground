use super::types::{Binding, Builtin, Closure, Primitive};
use crate::monkey::vm::types::Op;
use std::{
    collections::{HashMap, VecDeque},
    fmt::Write,
};

#[derive(Debug)]
struct Frame {
    closure: Closure,
    ip: usize,
    base: usize,
}

impl Frame {
    fn new(closure: Closure, base: usize) -> Self {
        Self {
            closure,
            ip: 0,
            base,
        }
    }
    fn next(&mut self) -> u8 {
        if self.ip < self.closure.fn_.body.len() {
            let ret = self.closure.fn_.body[self.ip];
            self.ip += 1;
            ret
        } else {
            panic!("Reached end of frame");
        }
    }
    fn next_u16(&mut self) -> u16 {
        u16::from_be_bytes([self.next(), self.next()])
    }
    fn next_u8(&mut self) -> u8 {
        u8::from_be_bytes([self.next()])
    }
    fn set_exec(&mut self, pos: usize) {
        self.ip = pos;
    }
    fn valid_pos(&self) -> bool {
        self.ip < self.closure.fn_.body.len()
    }
}

pub struct Machine {
    stack: Vec<Binding>,
    sp: usize,
    globals: Vec<Binding>,
    frames: Vec<Frame>,
    stdout: String,
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            sp: 0,
            stack: vec![Binding::Null; u16::MAX as usize],
            globals: vec![Binding::Null; u16::MAX as usize],
            frames: Vec::new(),
            stdout: String::new(),
        }
    }
}

impl Machine {
    pub fn run(&mut self, constants: Vec<Primitive>, closure: Closure) {
        println!("Constants: {:?}", constants);

        let main_frame = Frame::new(closure, 0);
        self.frames.push(main_frame);

        while let Some(frame) = self.frames.last() {
            if !frame.valid_pos() {
                break;
            }
            let op = Op::from(self.frame().next());
            match op {
                Op::Add
                | Op::Sub
                | Op::Mul
                | Op::Div
                | Op::Eq
                | Op::Neq
                | Op::Lt
                | Op::Index
                | Op::Mod => {
                    self.binary_op(op);
                }
                Op::Bang => {
                    let pref = self.pop().clone();
                    match pref {
                        Binding::Primitive(Primitive::Bool(b)) => {
                            self.push(Primitive::Bool(!b).into());
                        }
                        Binding::Primitive(Primitive::Int(i)) => {
                            self.push(Primitive::Bool(i != 0).into());
                        }
                        Binding::Null => {
                            self.push(Primitive::Bool(true).into());
                        }
                        _ => panic!("Invalid types for bang"),
                    }
                }
                Op::Minus => {
                    let pref = self.pop().clone();
                    match pref {
                        Binding::Primitive(Primitive::Int(i)) => {
                            self.push(Primitive::Int(-i).into());
                        }
                        _ => panic!("Invalid types for minus"),
                    }
                }
                Op::Jmp => {
                    let ix = self.frame().next_u16();
                    self.frame().set_exec(ix as usize);
                }
                Op::JmpIfFalse => {
                    let ix = self.frame().next_u16();
                    let value = self.pop();
                    let is_truthy = match value {
                        Binding::Primitive(Primitive::Bool(b)) => *b,
                        Binding::Primitive(Primitive::Int(i)) => *i != 0,
                        Binding::Null => true,
                        _ => panic!("Invalid types for JmpIfFalse"),
                    };
                    if !is_truthy {
                        self.frame().set_exec(ix as usize);
                    }
                }
                Op::Call => {
                    let num_args_called = self.frame().next_u8();
                    let binding = self.stack[self.sp - 1 - num_args_called as usize].clone();
                    match binding {
                        Binding::Closure(closure) => {
                            let fn_ = closure.fn_;
                            let num_args = fn_.num_args as usize;
                            let num_locals = fn_.num_locals as usize;
                            if num_args_called != num_args as u8 {
                                panic!(
                                    "Expected {} args for call but got {}",
                                    fn_.num_args, num_args_called
                                );
                            }
                            let closure = Closure::new(fn_, closure.free);
                            let frame = Frame::new(closure, self.sp - num_args);
                            self.sp = frame.base + num_locals;

                            self.frames.push(frame);
                        }
                        Binding::Builtin(builtin) => {
                            self.builtin_call(builtin, num_args_called);
                        }
                        _ => panic!("Expected closure or builtin for call"),
                    }
                }
                Op::Array => {
                    let n = self.frame().next_u16();
                    println!("Array of length {}", n);
                    let mut array = VecDeque::new();
                    for _ in 0..n {
                        array.push_front(self.pop().clone());
                    }
                    self.push(Binding::Array(array.into()));
                }
                Op::Hash => {
                    let n = self.frame().next_u16();
                    println!("Hash of length {}", n);
                    let mut hash = HashMap::new();
                    for _ in 0..n {
                        let value = self.pop().clone();
                        let key = self.pop();
                        if let Binding::Primitive(key) = key {
                            if !matches!(key, Primitive::Fn(_)) {
                                hash.insert(key.clone(), value);
                            }
                        } else {
                            panic!("Invalid key for hash");
                        }
                    }
                    self.push(Binding::Hash(hash));
                }
                Op::Closure => {
                    let ix = self.frame().next_u16();
                    let num_free = self.frame().next_u8() as usize;
                    let fn_ = constants[ix as usize].clone();
                    let fn_ = if let Primitive::Fn(fn_) = fn_ {
                        fn_
                    } else {
                        panic!("Expected Function literal for closure");
                    };
                    let mut free = Vec::new();
                    for i in 0..num_free {
                        free.push(self.stack[self.sp - num_free + i].clone());
                    }
                    self.sp -= num_free;
                    let closure = Closure::new(fn_, free);
                    self.push(Binding::Closure(closure));
                }
                Op::Null => {
                    self.push(Binding::Null);
                }
                Op::Constant => {
                    let ix = self.frame().next_u16();
                    self.push(Binding::Primitive(constants[ix as usize].clone()));
                }
                Op::Pop => {
                    let value = self.pop();
                    println!("Popped {:?} from stack", value);
                }
                Op::True => {
                    self.push(Primitive::Bool(true).into());
                }
                Op::False => {
                    self.push(Primitive::Bool(false).into());
                }
                Op::SetGlobal => {
                    let ix = self.frame().next_u16();
                    self.globals[ix as usize] = self.pop().clone();
                }
                Op::SetLocal => {
                    let ix = self.frame().next_u16() as usize;
                    let base = self.frame().base;
                    let value = self.pop().clone();
                    self.stack[base + ix] = value;
                }
                Op::GetGlobal => {
                    let ix = self.frame().next_u16();
                    self.push(self.globals[ix as usize].clone());
                }
                Op::GetLocal => {
                    let ix = self.frame().next_u16() as usize;
                    let base = self.frame().base;
                    self.push(self.stack[base + ix].clone());
                }
                Op::GetBuiltin => {
                    let builtin = Builtin::from(self.frame().next_u8());
                    println!("Builtin: {}", builtin);
                    self.push(Binding::Builtin(builtin));
                }
                Op::GetFree => {
                    let ix = self.frame().next_u8() as usize;
                    let val = self.frame().closure.free[ix].clone();
                    self.push(val);
                }
                Op::CurrentClosure => {
                    let closure = self.frame().closure.clone();
                    self.push(Binding::Closure(closure));
                }
                Op::ReturnVal => {
                    let val = self.pop().clone();
                    let frame = self.frames.pop().expect("No frame to return to");
                    self.sp = frame.base - 1;
                    self.push(val);
                }
                Op::Return => {
                    let frame = self.frames.pop().expect("No frame to return to");
                    self.sp = frame.base - 1;
                    self.push(Binding::Null);
                }
            }
        }
    }

    fn pop(&mut self) -> &Binding {
        self.sp -= 1;
        &self.stack[self.sp]
    }

    fn push(&mut self, binding: Binding) {
        self.stack[self.sp] = binding;
        self.sp += 1;
    }

    fn frame(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }
    pub fn get_stdout(&self) -> String {
        self.stdout.clone()
    }
    pub fn get_last_expr(&mut self) -> Binding {
        match self.sp {
            0 => Binding::Null,
            1 => self.pop().clone(),
            _ => panic!("Expected only one expression on stack"),
        }
    }
    fn binary_op(&mut self, op: Op) {
        let right = self.pop().clone();
        let left = self.pop().clone();
        match (left, right) {
            (Binding::Primitive(left), Binding::Primitive(right)) => match (left, right) {
                (Primitive::Int(l), Primitive::Int(r)) => match op {
                    Op::Add => self.push(Primitive::Int(l + r).into()),
                    Op::Sub => self.push(Primitive::Int(l - r).into()),
                    Op::Mul => self.push(Primitive::Int(l * r).into()),
                    Op::Div => self.push(Primitive::Int(l / r).into()),
                    Op::Eq => self.push(Primitive::Bool(l == r).into()),
                    Op::Neq => self.push(Primitive::Bool(l != r).into()),
                    Op::Lt => self.push(Primitive::Bool(l < r).into()),
                    Op::Mod => self.push(Primitive::Int(l % r).into()),
                    _ => panic!("Invalid op for ints"),
                },
                (Primitive::Bool(l), Primitive::Bool(r)) => match op {
                    Op::Eq => self.push(Primitive::Bool(l == r).into()),
                    Op::Neq => self.push(Primitive::Bool(l != r).into()),
                    _ => panic!("Invalid op for bools"),
                },
                (Primitive::String_(l), Primitive::String_(r)) => match op {
                    Op::Eq => self.push(Primitive::Bool(l == r).into()),
                    Op::Neq => self.push(Primitive::Bool(l != r).into()),
                    Op::Add => self.push(Primitive::String_(format!("{}{}", l, r)).into()),
                    _ => panic!("Invalid op for string"),
                },
                _ => panic!("Invalid types for binary op",),
            },
            (Binding::Array(l), Binding::Primitive(Primitive::Int(r))) => match op {
                Op::Index => {
                    if r < 0 {
                        panic!("Expected positive index for array");
                    } else if r as usize >= l.len() {
                        panic!("Expected index less than array length");
                    } else {
                        self.push(l[r as usize].clone());
                    }
                }
                _ => panic!("Invalid op for array"),
            },
            (Binding::Hash(l), Binding::Primitive(key)) => match op {
                Op::Index => {
                    if let Some(key) = l.get(&key) {
                        self.stack.push(key.clone());
                    } else {
                        panic!("Invalid key for hash");
                    }
                }
                _ => panic!("Invalid op for hash"),
            },
            _ => panic!("Invalid types for binary op"),
        }
    }

    fn builtin_call(&mut self, builtin: Builtin, num_args: u8) {
        //Gather args
        let mut args = Vec::new();
        for _ in 0..num_args {
            args.push(self.pop().clone());
        }
        //Clean itself off the stack
        self.pop();
        println!("Builtin args: {:?}", args);
        match builtin {
            Builtin::Len => {
                if args.len() != 1 {
                    panic!("Expected single argument for len builtin");
                }
                match args.pop().unwrap() {
                    Binding::Array(a) => self.push(Primitive::Int(a.len() as i64).into()),
                    Binding::Hash(h) => self.push(Primitive::Int(h.len() as i64).into()),
                    Binding::Primitive(Primitive::String_(s)) => {
                        self.push(Primitive::Int(s.len() as i64).into())
                    }
                    _ => panic!("Expected array, hash or string for len builtin"),
                }
            }
            Builtin::First => {
                if args.len() != 1 {
                    panic!("Expected single argument for first builtin");
                }
                match args.pop().unwrap() {
                    Binding::Array(a) => {
                        if let Some(first) = a.first() {
                            self.push(first.clone());
                        } else {
                            self.push(Binding::Null);
                        }
                    }
                    _ => panic!("Expected array for first builtin"),
                }
            }
            Builtin::Last => {
                if args.len() != 1 {
                    panic!("Expected single argument for last builtin");
                }
                match args.pop().unwrap() {
                    Binding::Array(a) => {
                        if let Some(last) = a.last() {
                            self.push(last.clone());
                        } else {
                            self.push(Binding::Null);
                        }
                    }
                    _ => panic!("Expected array for last builtin"),
                }
            }
            Builtin::Rest => {
                if args.len() != 1 {
                    panic!("Expected single argument for rest builtin");
                }
                match args.pop().unwrap() {
                    Binding::Array(a) => {
                        self.push(Binding::Array(a.iter().skip(1).cloned().collect()))
                    }
                    _ => panic!("Expected array for rest builtin"),
                }
            }
            Builtin::Push => {
                if args.len() != 2 {
                    panic!("Expected two arguments for push builtin");
                }
                match (args.pop().unwrap(), args.pop().unwrap()) {
                    (Binding::Array(a), new) => {
                        let mut ret = a.clone();
                        ret.push(new.clone());
                        self.push(Binding::Array(ret));
                    }
                    _ => panic!("Expected array and int or string for push builtin"),
                }
            }
            Builtin::Puts => {
                while let Some(arg) = args.pop() {
                    self.stdout.write_fmt(format_args!("{}\n", arg)).unwrap();
                }
                self.push(Binding::Null);
            }
        }
    }
}
