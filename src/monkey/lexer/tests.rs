#[cfg(test)]
#[test]
fn simple_test() {
    use crate::lexer::Lexer;
    use crate::lexer::Token;
    let expected = vec![
        Token::Assign,
        Token::Plus,
        Token::LParen,
        Token::RParen,
        Token::LBrace,
        Token::RBrace,
        Token::Comma,
        Token::Semicolon,
        Token::Minus,
        Token::Star,
        Token::Lt,
        Token::Gt,
        Token::Slash,
        Token::Bang,
        Token::Eof,
    ];
    let lexer = Lexer::new("=+(){},;-*<>/!\0".bytes());
    let output = lexer.collect::<Vec<_>>();
    assert_eq!(output, expected);
}

#[test]
fn complex_expr_test() {
    use crate::lexer::Lexer;
    use crate::lexer::{Keyword::*, Token};
    let mut lexer = Lexer::new(
        "let five    = 5;\n\
         let ten = 10;\
         let add_5_under = fn(x, y){\n\
             x + y;\
         };\
         let result = add(five, ten);\
         !-/*5;
         5 < 10 > 5;\
         if (5 < 10) {\
             return true;\
         } else {\
             return false;\
         }\
         10 == 10;\
         10 != 9;\
         "
        .bytes(),
    );
    assert_eq!(lexer.next(), Some(Token::Keyword(Let)));
    assert_eq!(lexer.next(), Some(Token::Ident("five".to_string())));
    assert_eq!(lexer.next(), Some(Token::Assign));
    assert_eq!(lexer.next(), Some(Token::Int("5".to_string())));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::Keyword(Let)));
    assert_eq!(lexer.next(), Some(Token::Ident("ten".to_string())));
    assert_eq!(lexer.next(), Some(Token::Assign));
    assert_eq!(lexer.next(), Some(Token::Int("10".to_string())));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::Keyword(Let)));
    assert_eq!(lexer.next(), Some(Token::Ident("add_5_under".to_string())));
    assert_eq!(lexer.next(), Some(Token::Assign));
    assert_eq!(lexer.next(), Some(Token::Keyword(Function)));
    assert_eq!(lexer.next(), Some(Token::LParen));
    assert_eq!(lexer.next(), Some(Token::Ident("x".to_string())));
    assert_eq!(lexer.next(), Some(Token::Comma));
    assert_eq!(lexer.next(), Some(Token::Ident("y".to_string())));
    assert_eq!(lexer.next(), Some(Token::RParen));
    assert_eq!(lexer.next(), Some(Token::LBrace));
    assert_eq!(lexer.next(), Some(Token::Ident("x".to_string())));
    assert_eq!(lexer.next(), Some(Token::Plus));
    assert_eq!(lexer.next(), Some(Token::Ident("y".to_string())));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::RBrace));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::Keyword(Let)));
    assert_eq!(lexer.next(), Some(Token::Ident("result".to_string())));
    assert_eq!(lexer.next(), Some(Token::Assign));
    assert_eq!(lexer.next(), Some(Token::Ident("add".to_string())));
    assert_eq!(lexer.next(), Some(Token::LParen));
    assert_eq!(lexer.next(), Some(Token::Ident("five".to_string())));
    assert_eq!(lexer.next(), Some(Token::Comma));
    assert_eq!(lexer.next(), Some(Token::Ident("ten".to_string())));
    assert_eq!(lexer.next(), Some(Token::RParen));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::Bang));
    assert_eq!(lexer.next(), Some(Token::Minus));
    assert_eq!(lexer.next(), Some(Token::Slash));
    assert_eq!(lexer.next(), Some(Token::Star));
    assert_eq!(lexer.next(), Some(Token::Int("5".to_string())));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::Int("5".to_string())));
    assert_eq!(lexer.next(), Some(Token::Lt));
    assert_eq!(lexer.next(), Some(Token::Int("10".to_string())));
    assert_eq!(lexer.next(), Some(Token::Gt));
    assert_eq!(lexer.next(), Some(Token::Int("5".to_string())));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::Keyword(If)));
    assert_eq!(lexer.next(), Some(Token::LParen));
    assert_eq!(lexer.next(), Some(Token::Int("5".to_string())));
    assert_eq!(lexer.next(), Some(Token::Lt));
    assert_eq!(lexer.next(), Some(Token::Int("10".to_string())));
    assert_eq!(lexer.next(), Some(Token::RParen));
    assert_eq!(lexer.next(), Some(Token::LBrace));
    assert_eq!(lexer.next(), Some(Token::Keyword(Return)));
    assert_eq!(lexer.next(), Some(Token::Keyword(True)));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::RBrace));
    assert_eq!(lexer.next(), Some(Token::Keyword(Else)));
    assert_eq!(lexer.next(), Some(Token::LBrace));
    assert_eq!(lexer.next(), Some(Token::Keyword(Return)));
    assert_eq!(lexer.next(), Some(Token::Keyword(False)));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::RBrace));
    assert_eq!(lexer.next(), Some(Token::Int("10".to_string())));
    assert_eq!(lexer.next(), Some(Token::Eq));
    assert_eq!(lexer.next(), Some(Token::Int("10".to_string())));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), Some(Token::Int("10".to_string())));
    assert_eq!(lexer.next(), Some(Token::Neq));
    assert_eq!(lexer.next(), Some(Token::Int("9".to_string())));
    assert_eq!(lexer.next(), Some(Token::Semicolon));
    assert_eq!(lexer.next(), None);
}
