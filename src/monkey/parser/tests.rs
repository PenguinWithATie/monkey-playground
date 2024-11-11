#[cfg(test)]
#[test]
fn test_let_valid() {
    use super::{Expr, Infix, Let, Parser};
    use crate::lexer::{Lexer, Token};
    let input = "
        let x = 1;\n\
        let the_third = x + lele;";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    assert_eq!(program.statements.len(), 2);
    assert_eq!(
        program.statements[0],
        Expr::Let(Let {
            name: "x".to_string(),
            value: Box::new(Expr::Int(1))
        })
    );
    assert_eq!(
        program.statements[1],
        Expr::Let(Let {
            name: "the_third".to_string(),
            value: Box::new(Expr::Infix(Box::new(Infix {
                left: Expr::Identifier("x".to_string()),
                token: Token::Plus,
                right: Expr::Identifier("lele".to_string())
            })))
        })
    );
}

#[test]
fn test_return_valid() {
    use super::{Expr, Infix, Parser};
    use crate::lexer::{Lexer, Token};
    let input = "
        return 1;\n\
        return a;
        return 1+2;
        return true;";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    assert_eq!(program.statements.len(), 4);
    assert_eq!(program.statements[0], Expr::Return(Box::new(Expr::Int(1))));
    assert_eq!(
        program.statements[1],
        Expr::Return(Box::new(Expr::Identifier("a".to_string())))
    );
    assert_eq!(
        program.statements[2],
        Expr::Return(Box::new(Expr::Infix(Box::new(Infix {
            left: Expr::Int(1),
            token: Token::Plus,
            right: Expr::Int(2)
        }))))
    );
    assert_eq!(
        program.statements[3],
        Expr::Return(Box::new(Expr::Bool(true)))
    );
}

#[test]
fn test_simple_prefix() {
    use super::{Expr, Parser, Prefix};
    use crate::lexer::{Lexer, Token};
    let input = "!foobar;
    -5";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    assert_eq!(program.statements.len(), 2);
    assert_eq!(
        program.statements[0],
        Expr::Prefix(Box::new(Prefix {
            token: Token::Bang,
            right: Expr::Identifier("foobar".to_string())
        }))
    );
    assert_eq!(
        program.statements[1],
        Expr::Prefix(Box::new(Prefix {
            token: Token::Minus,
            right: Expr::Int(5)
        }))
    );
}

#[test]
fn test_order_of_operations() {
    use super::{Expr, Infix, Parser};
    use crate::lexer::Lexer;
    let input = "x/y+z-1*5";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    assert_eq!(program.statements.len(), 1);
    let string = format!("{}", program.statements[0]);
    assert_eq!(string, "(((x/y)+z)-(1*5))");
}

#[test]
pub fn test_if_valid() {
    use super::{Block, Expr, If, Infix, Parser};
    use crate::lexer::{Lexer, Token};
    let input = "
        if (x < y) {
            return true;
        } else {
            return false;
        }
    ";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    assert_eq!(program.statements.len(), 1);
    assert_eq!(
        program.statements[0],
        Expr::If(If {
            condition: Box::new(Expr::Infix(Box::new(Infix {
                left: Expr::Identifier("x".to_string()),
                token: Token::Lt,
                right: Expr::Identifier("y".to_string())
            }))),
            consequence: Block(vec![Expr::Return(Box::new(Expr::Bool(true)))]),
            alternative: Some(Block(vec![Expr::Return(Box::new(Expr::Bool(false)))]))
        })
    );
}

#[test]
pub fn test_fn_valid() {
    use super::{Block, Expr, Fn, Infix, Parser};
    use crate::lexer::{Lexer, Token};
    let input = "
        fn(x, y ){
            x + y;
        }

        fn() {
            return true;
        }
    ";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    assert_eq!(program.statements.len(), 2);
    assert_eq!(
        program.statements[0],
        Expr::Fn(Fn {
            args: vec![Token::Ident("x".to_string()), Token::Ident("y".to_string())],
            body: Block(vec![Expr::Infix(Box::new(Infix {
                left: Expr::Identifier("x".to_string()),
                token: Token::Plus,
                right: Expr::Identifier("y".to_string())
            }))])
        })
    );
    assert_eq!(
        program.statements[1],
        Expr::Fn(Fn {
            args: vec![],
            body: Block(vec![Expr::Return(Box::new(Expr::Bool(true)))])
        })
    );
}

#[test]
fn nested_fn_test() {
    use crate::lexer::{Lexer, Token};
    use crate::parser::{Block, Expr, Fn, Infix, Let, Parser};
    let input = "
    let new_adder = fn(x) {fn(y){x + y};};
    ";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    assert_eq!(program.statements.len(), 1);
    assert_eq!(
        program.statements[0],
        Expr::Let(Let {
            name: "new_adder".to_string(),
            value: Box::new(Expr::Fn(Fn {
                args: vec![Token::Ident("x".to_string())],
                body: Block(vec![Expr::Fn(Fn {
                    args: vec![Token::Ident("y".to_string())],
                    body: Block(vec![Expr::Infix(Box::new(Infix {
                        left: Expr::Identifier("x".to_string()),
                        token: Token::Plus,
                        right: Expr::Identifier("y".to_string())
                    }))])
                })])
            }))
        })
    );
}

#[test]
fn test_array_valid() {
    use super::{Expr, Infix, Let, Parser};
    use crate::lexer::{Lexer, Token};
    let input = "
        let x = [1, 2, 3];
        let y = [x[0], x[1], x[2]];
    ";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    assert_eq!(program.statements.len(), 1);
    assert_eq!(
        program.statements[0],
        Expr::Let(Let {
            name: "x".to_string(),
            value: Box::new(Expr::Array(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]))
        })
    );
    assert_eq!(
        program.statements[1],
        Expr::Let(Let {
            name: "y".to_string(),
            value: Box::new(Expr::Array(vec![
                Expr::Infix(Box::new(Infix {
                    left: Expr::Identifier("x".to_string()),
                    token: Token::LBracket,
                    right: Expr::Int(0)
                })),
                Expr::Infix(Box::new(Infix {
                    left: Expr::Identifier("x".to_string()),
                    token: Token::LBracket,
                    right: Expr::Int(1)
                })),
                Expr::Infix(Box::new(Infix {
                    left: Expr::Identifier("x".to_string()),
                    token: Token::LBracket,
                    right: Expr::Int(2)
                }))
            ]))
        })
    );
    /*assert_eq!(
        program.statements[2],
        Expr::Infix(Box::new(Infix {
            left: Expr::Identifier("y".to_string()),
            token: Token::LBracket,
            right: Expr::Int(0)
        }))
    );*/
}
