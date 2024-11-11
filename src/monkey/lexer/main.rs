use std::{
    iter::{Iterator, Peekable},
    str::FromStr,
};

use super::tokens::{Keyword, Token};
pub trait TokenIterator: Iterator<Item = u8> {}

impl<T: Iterator<Item = u8>> TokenIterator for T {}
pub struct Tokenizer<I: TokenIterator> {
    input: Peekable<I>,
}

impl<I: TokenIterator> Tokenizer<I> {
    pub fn new(input: I) -> Self {
        Self {
            input: input.peekable(),
        }
    }
    fn consume_whitespace(&mut self) {
        loop {
            if self
                .input
                .next_if(|b| [b' ', b'\n', b'\t', b'\r'].contains(b))
                .is_none()
            {
                break;
            }
        }
    }
}

impl<I: TokenIterator> Iterator for Tokenizer<I> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        //Consume whitespace
        self.consume_whitespace();
        match self.input.peek()? {
            b'+' => {
                self.input.next();
                Some(Token::Plus)
            }
            b'-' => {
                self.input.next();
                Some(Token::Minus)
            }
            b'*' => {
                self.input.next();
                Some(Token::Star)
            }
            b'/' => {
                self.input.next();
                Some(Token::Slash)
            }
            b'>' => {
                self.input.next();
                Some(Token::Gt)
            }
            b'<' => {
                self.input.next();
                Some(Token::Lt)
            }
            b',' => {
                self.input.next();
                Some(Token::Comma)
            }
            b';' => {
                self.input.next();
                Some(Token::Semicolon)
            }
            b'(' => {
                self.input.next();
                Some(Token::LParen)
            }
            b')' => {
                self.input.next();
                Some(Token::RParen)
            }
            b'{' => {
                self.input.next();
                Some(Token::LBrace)
            }
            b'}' => {
                self.input.next();
                Some(Token::RBrace)
            }
            0 => {
                self.input.next();
                Some(Token::Eof)
            }
            b'=' => {
                self.input.next();
                if self.input.next_if(|b| *b == b'=').is_some() {
                    Some(Token::Eq)
                } else {
                    Some(Token::Assign)
                }
            }
            b'!' => {
                self.input.next();
                if self.input.next_if(|b| *b == b'=').is_some() {
                    Some(Token::Neq)
                } else {
                    Some(Token::Bang)
                }
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let mut ident = String::new();
                while let Some(b) = self.input.peek() {
                    if b.is_ascii_alphanumeric() || *b == b'_' {
                        ident.push(*b as char);
                        self.input.next();
                    } else {
                        break;
                    }
                }
                if let Ok(key) = Keyword::from_str(&ident) {
                    Some(Token::Keyword(key))
                } else {
                    Some(Token::Ident(ident))
                }
            }
            b'0'..=b'9' => {
                let mut int = String::new();
                while let Some(b) = self.input.peek() {
                    if b.is_ascii_digit() {
                        int.push(*b as char);
                        self.input.next();
                    } else {
                        break;
                    }
                }
                Some(Token::Int(int))
            }
            b'"' => {
                self.input.next();
                let mut string = String::new();
                while let Some(b) = self.input.peek() {
                    if *b == b'"' {
                        self.input.next();
                        return Some(Token::String(string));
                    } else {
                        string.push(*b as char);
                        self.input.next();
                    }
                }
                Some(Token::Illegal)
            }
            b'[' => {
                self.input.next();
                Some(Token::LBracket)
            }
            b']' => {
                self.input.next();
                Some(Token::RBracket)
            }
            b':' => {
                self.input.next();
                Some(Token::Colon)
            }
            b'%' => {
                self.input.next();
                Some(Token::Percent)
            }
            _ => {
                self.input.next();
                Some(Token::Illegal)
            }
        }
    }
}
