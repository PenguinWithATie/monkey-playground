use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Keyword(Keyword),
    Ident(String),
    Int(String),
    String(String),
    //operators
    Assign,
    Plus,
    Minus,
    Bang,
    Star,
    Slash,
    Eq,
    Neq,
    Gt,
    Lt,
    Percent,
    //delimiters
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    //terminators
    Illegal,
    Eof,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Keyword(k) => write!(f, "{}", k),
            Token::Ident(i) => write!(f, "{}", i),
            Token::Int(i) => write!(f, "{}", i),
            Token::String(s) => write!(f, "'{}'", s),
            Token::Assign => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Bang => write!(f, "!"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Eq => write!(f, "=="),
            Token::Neq => write!(f, "!="),
            Token::Gt => write!(f, ">"),
            Token::Lt => write!(f, "<"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Colon => write!(f, ":"),
            Token::Percent => write!(f, "%"),
            Token::Illegal => write!(f, "Illegal"),
            Token::Eof => write!(f, ""),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Keyword {
    Function,
    Let,
    If,
    Else,
    True,
    False,
    Return,
}
impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Function => write!(f, "fn"),
            Keyword::Let => write!(f, "let"),
            Keyword::If => write!(f, "if"),
            Keyword::Else => write!(f, "else"),
            Keyword::True => write!(f, "true"),
            Keyword::False => write!(f, "false"),
            Keyword::Return => write!(f, "return"),
        }
    }
}
impl FromStr for Keyword {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fn" => Ok(Keyword::Function),
            "let" => Ok(Keyword::Let),
            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "true" => Ok(Keyword::True),
            "false" => Ok(Keyword::False),
            "return" => Ok(Keyword::Return),
            _ => Err("Invalid keyword".to_string()),
        }
    }
}
