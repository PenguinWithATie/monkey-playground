use std::rc::Rc;

use super::types::*;
use crate::monkey::lexer::{Keyword, Lexer, Token, TokenIterator};

const PREFIX_PRECEDENCE: u8 = 10;
const LOWEST_PRECEDENCE: u8 = 0;
const CALL_PRECEDENCE: u8 = 12;
const INDEXER_PRECEDENCE: u8 = 15;

pub struct Program {
    pub statements: Vec<Expr>,
}

#[derive(Debug)]
pub enum Error {
    ExpectedToken(Token),
    BadPrefix,
    BadReturn,
    BadInt,
    BadArray,
    BadInfix,
    BadCall,
    BadFn,
    BadHash,
    NoToken,
}
type Result<T> = std::result::Result<T, Error>;

pub struct Parser<I: TokenIterator> {
    tokens: Lexer<I>,
}

impl<I: TokenIterator> Parser<I> {
    pub fn new(tokens: Lexer<I>) -> Self {
        Self { tokens }
    }
    pub fn program(&mut self) -> Result<Program> {
        let mut stmts = Vec::new();
        while !matches!(self.tokens.peek(), Some(Token::Eof) | None) {
            stmts.push(self.statement()?);
        }
        Ok(Program { statements: stmts })
    }
    fn statement(&mut self) -> Result<Expr> {
        match self.tokens.peek().unwrap() {
            Token::Keyword(Keyword::Let) => self.let_(),
            Token::Keyword(Keyword::Return) => self.return_(),
            _ => {
                let ret = self.expr(LOWEST_PRECEDENCE);
                //Optional semicolon at the end of expresion statement
                let _ = self.next_if_semicolon();
                ret
            }
        }
    }
    fn block(&mut self) -> Result<Block> {
        let mut stmts = Vec::new();
        self.next_if_lbrace()?;
        while !matches!(
            self.tokens.peek(),
            Some(Token::Eof) | None | Some(Token::RBrace)
        ) {
            stmts.push(self.statement()?);
        }
        self.next_if_rbrace()?;
        Ok(Block(stmts))
    }
    fn expr(&mut self, precedence: u8) -> Result<Expr> {
        let mut left_expr = Rc::new(self.prefix()?);
        while !matches!(self.tokens.peek(), Some(Token::Semicolon) | None)
            && precedence < self.current_precedence()
        {
            match self.infix(left_expr.clone())? {
                Some(expr) => {
                    left_expr = Rc::new(expr);
                }
                _ => break,
            }
        }
        Ok(Rc::try_unwrap(left_expr).unwrap())
    }

    fn grouped_expr(&mut self) -> Result<Expr> {
        self.tokens.next();
        let expr = self.expr(LOWEST_PRECEDENCE)?;
        self.next_if_rparen()?;
        Ok(expr)
    }

    fn let_(&mut self) -> Result<Expr> {
        self.next_if_let()?;
        let name = if let Token::Ident(n) = self.next_if_ident()? {
            n
        } else {
            return Err(Error::ExpectedToken(Token::Ident("".to_string())));
        };
        self.next_if_assign()?;
        let mut value = Box::new(self.expr(LOWEST_PRECEDENCE)?);
        self.next_if_semicolon()?;
        if let Expr::Fn(fn_) = value.as_ref() {
            value = Box::new(Expr::Fn(Rc::new(Fn {
                name: Some(name.clone()),
                body: fn_.body.clone(),
                args: fn_.args.clone(),
            })));
        }
        Ok(Expr::Let(Let { name, value }))
    }
    fn return_(&mut self) -> Result<Expr> {
        self.next_if_return()?;
        let expr = self.expr(LOWEST_PRECEDENCE);
        self.next_if_semicolon()?;
        if let Ok(expr) = expr {
            Ok(Expr::Return(Box::new(expr)))
        } else {
            Err(Error::BadReturn)
        }
    }
    fn if_(&mut self) -> Result<Expr> {
        self.next_if_if()?;

        self.next_if_lparen()?;
        let condition = self.expr(LOWEST_PRECEDENCE)?;
        self.next_if_rparen()?;

        let body = self.block()?;

        if self.next_if_else().is_err() {
            Ok(Expr::If(If {
                condition: Box::new(condition),
                consequence: body,
                alternative: None,
            }))
        } else {
            let alternative = self.block()?;
            Ok(Expr::If(If {
                condition: Box::new(condition),
                consequence: body,
                alternative: Some(alternative),
            }))
        }
    }
    fn prefix(&mut self) -> Result<Expr> {
        let token = self.tokens.peek();
        if let Some(token) = token {
            match token {
                Token::Ident(_) => match self.tokens.next() {
                    Some(Token::Ident(i)) => Ok(Expr::Identifier(i)),
                    _ => unreachable!(),
                },
                Token::Int(_) => match self.tokens.next() {
                    Some(Token::Int(t)) => t
                        .parse::<i64>()
                        .map_or(Err(Error::BadInt), |t| Ok(Expr::Int(t))),
                    _ => unreachable!(),
                },
                Token::String(_) => match self.tokens.next() {
                    Some(Token::String(t)) => Ok(Expr::String(t)),
                    _ => unreachable!(),
                },
                Token::Keyword(Keyword::True) => {
                    self.tokens.next();
                    Ok(Expr::Bool(true))
                }
                Token::Keyword(Keyword::False) => {
                    self.tokens.next();
                    Ok(Expr::Bool(false))
                }
                Token::LParen => self.grouped_expr(),
                Token::Bang | Token::Minus => {
                    let token = self.tokens.next().unwrap();
                    Ok(Expr::Prefix(Box::new(Prefix {
                        token,
                        right: self.expr(PREFIX_PRECEDENCE)?,
                    })))
                }
                Token::Keyword(Keyword::If) => self.if_(),
                Token::Keyword(Keyword::Function) => self.fn_(),
                Token::LBracket => self.array(),
                Token::LBrace => self.hash(),
                _ => Err(Error::BadPrefix),
            }
        } else {
            Err(Error::BadPrefix)
        }
    }
    fn infix(&mut self, left: Rc<Expr>) -> Result<Option<Expr>> {
        let token = self.tokens.peek();
        if let Some(token) = token {
            match token {
                Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Eq
                | Token::Neq
                | Token::Lt
                | Token::Gt
                | Token::Percent => {
                    let precedence = self.current_precedence();
                    let token = self.tokens.next().unwrap();
                    let right = self.expr(precedence)?;
                    Ok(Some(Expr::Infix(Box::new(Infix { left, token, right }))))
                }
                Token::LParen => {
                    let expr = self.call(left)?;
                    Ok(Some(expr))
                }
                Token::LBracket => {
                    let expr = self.indexer(left)?;
                    Ok(Some(expr))
                }
                _ => Ok(None),
            }
        } else {
            Err(Error::BadInfix)
        }
    }
    fn call(&mut self, left: Rc<Expr>) -> Result<Expr> {
        let mut args = Vec::new();
        self.next_if_lparen()?;
        while !matches!(self.peek()?, Token::RParen | Token::Eof) {
            if let Ok(expr) = self.expr(LOWEST_PRECEDENCE) {
                args.push(expr);
                match self.peek()? {
                    Token::Comma => {
                        self.tokens.next();
                    }
                    Token::RParen => {
                        continue;
                    }
                    _ => {
                        return Err(Error::BadCall);
                    }
                }
            } else {
                return Err(Error::BadCall);
            }
        }
        self.next_if_rparen()?;
        Ok(Expr::Call(Call { expr: left, args }))
    }
    fn fn_(&mut self) -> Result<Expr> {
        self.next_if_fn()?;
        let mut args = Vec::new();
        self.next_if_lparen()?;
        while !matches!(self.peek()?, Token::RParen | Token::Eof) {
            if matches!(self.peek()?, Token::Ident(_)) {
                match self.tokens.next() {
                    Some(Token::Ident(i)) => args.push(i),
                    _ => unreachable!(),
                }
                match self.peek()? {
                    Token::Comma => {
                        self.tokens.next();
                    }
                    Token::RParen => {
                        continue;
                    }
                    _ => {
                        return Err(Error::BadFn);
                    }
                }
            } else {
                return Err(Error::BadFn);
            }
        }
        self.next_if_rparen()?;
        let body = self.block()?;
        Ok(Expr::Fn(Rc::new(Fn {
            args,
            body,
            name: None,
        })))
    }

    fn array(&mut self) -> Result<Expr> {
        let mut elemns = Vec::new();
        self.next_if_lbracket()?;
        while !matches!(self.peek()?, Token::RBracket | Token::Eof) {
            let expr = self.expr(LOWEST_PRECEDENCE);
            if let Ok(expr) = expr {
                elemns.push(expr);
                match self.peek()? {
                    Token::Comma => {
                        self.tokens.next();
                    }
                    Token::RBracket => {
                        continue;
                    }
                    _ => {
                        return Err(Error::BadArray);
                    }
                }
            }
        }
        self.next_if_rbracket()?;
        Ok(Expr::Array(elemns))
    }
    fn indexer(&mut self, left: Rc<Expr>) -> Result<Expr> {
        self.next_if_lbracket()?;
        let index = self.expr(LOWEST_PRECEDENCE)?;
        self.next_if_rbracket()?;
        Ok(Expr::Infix(Box::new(Infix {
            left,
            token: Token::LBracket,
            right: index,
        })))
    }
    fn hash(&mut self) -> Result<Expr> {
        self.next_if_lbrace()?;
        let mut hash = Vec::new();
        while !matches!(self.tokens.peek(), Some(Token::RBrace) | None) {
            let key = self.expr(LOWEST_PRECEDENCE)?;
            self.next_if_colon()?;
            let value = self.expr(LOWEST_PRECEDENCE)?;
            hash.push((key, value));
            match self.peek()? {
                Token::Comma => {
                    self.tokens.next();
                }
                Token::RBrace => {
                    continue;
                }
                _ => {
                    return Err(Error::BadHash);
                }
            }
        }
        self.next_if_rbrace()?;
        Ok(Expr::Hash(hash))
    }
    fn current_precedence(&mut self) -> u8 {
        match self.tokens.peek() {
            Some(Token::Eq) => 1,
            Some(Token::Neq) => 1,
            Some(Token::Lt) => 2,
            Some(Token::Gt) => 2,
            Some(Token::Plus) => 3,
            Some(Token::Minus) => 3,
            Some(Token::Star) => 4,
            Some(Token::Slash) => 4,
            Some(Token::Percent) => 4,
            Some(Token::LParen) => CALL_PRECEDENCE,
            Some(Token::LBracket) => INDEXER_PRECEDENCE,
            _ => 100,
        }
    }
    pub fn next_if_let(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::Keyword(Keyword::Let))
            .ok_or(Error::ExpectedToken(Token::Keyword(Keyword::Let)))
    }
    pub fn next_if_assign(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::Assign)
            .ok_or(Error::ExpectedToken(Token::Assign))
    }
    pub fn next_if_semicolon(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::Semicolon)
            .ok_or(Error::ExpectedToken(Token::Semicolon))
    }
    pub fn next_if_ident(&mut self) -> Result<Token> {
        self.tokens
            .next_if(|t| matches!(t, Token::Ident(_)))
            .ok_or(Error::ExpectedToken(Token::Ident("".to_string())))
    }
    pub fn next_if_return(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::Keyword(Keyword::Return))
            .ok_or(Error::ExpectedToken(Token::Keyword(Keyword::Return)))
    }
    pub fn next_if_lparen(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::LParen)
            .ok_or(Error::ExpectedToken(Token::LParen))
    }
    pub fn next_if_rparen(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::RParen)
            .ok_or(Error::ExpectedToken(Token::RParen))
    }
    pub fn next_if_lbrace(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::LBrace)
            .ok_or(Error::ExpectedToken(Token::LBrace))
    }
    pub fn next_if_rbrace(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::RBrace)
            .ok_or(Error::ExpectedToken(Token::RBrace))
    }
    pub fn next_if_else(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::Keyword(Keyword::Else))
            .ok_or(Error::ExpectedToken(Token::Keyword(Keyword::Else)))
    }
    pub fn next_if_if(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::Keyword(Keyword::If))
            .ok_or(Error::ExpectedToken(Token::Keyword(Keyword::If)))
    }

    pub fn next_if_fn(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::Keyword(Keyword::Function))
            .ok_or(Error::ExpectedToken(Token::Keyword(Keyword::Function)))
    }
    pub fn next_if_lbracket(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::LBracket)
            .ok_or(Error::ExpectedToken(Token::LBracket))
    }
    pub fn next_if_rbracket(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::RBracket)
            .ok_or(Error::ExpectedToken(Token::RBracket))
    }
    pub fn next_if_colon(&mut self) -> Result<Token> {
        self.tokens
            .next_if_eq(&Token::Colon)
            .ok_or(Error::ExpectedToken(Token::Colon))
    }
    pub fn peek(&mut self) -> Result<&Token> {
        self.tokens.peek().ok_or(Error::NoToken)
    }
}
