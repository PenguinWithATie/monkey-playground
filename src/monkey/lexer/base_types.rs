use std::iter::Peekable;

use super::main::{TokenIterator, Tokenizer};
use crate::monkey::lexer::tokens::Token;

pub struct Lexer<I: TokenIterator> {
    tokenizer: Peekable<Tokenizer<I>>,
}

impl<I: TokenIterator> Lexer<I> {
    pub fn new(input: I) -> Self {
        Self {
            tokenizer: Tokenizer::new(input).peekable(),
        }
    }
    pub fn peek(&mut self) -> Option<&Token> {
        self.tokenizer.peek()
    }
    pub fn next_if_eq(&mut self, token: &Token) -> Option<Token> {
        self.tokenizer.next_if(|t| token == t)
    }
    pub fn next_if(&mut self, f: impl Fn(&Token) -> bool) -> Option<Token> {
        self.tokenizer.next_if(f)
    }
}
impl<I: TokenIterator> Iterator for Lexer<I> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.tokenizer.next()
    }
}
