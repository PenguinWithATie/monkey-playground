#[cfg(test)]
#[test]

fn closure_test() {
    use crate::evaluator::{Binding, Env, Evaluation};
    use crate::lexer::{Lexer, Token};
    use crate::parser::{Block, Expr, Fn, Infix, Let, Parser};
    let input = "
    let counter = fn(x) {\
        if (x > a) {\
            return true;\
        } else {\
         let foobar = x;
         counter(x+1);\
        }\
    };
    counter(1);
    ";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let program = parser.program().unwrap();
    let mut env = Env::default();
    let binding = program.eval(&mut env).unwrap();
    assert!(matches!(binding, Binding::Bool(true)));
}

#[test]
fn nested_fn_test() {
    use crate::evaluator::{Binding, Env, Evaluation};
    use crate::lexer::{Lexer, Token};
    use crate::parser::{Block, Expr, Fn, Infix, Let, Parser};
    let input = "
    let new_adder = fn(x) {return fn(y){x + y}; };
    let add_5 = new_adder(5);
    add_5(10);
    ";
    let lexer = Lexer::new(input.bytes());
    let mut parser = Parser::new(lexer);
    let mut env = Env::default();
    let program = parser.program().unwrap().eval(&mut env);
    assert!(matches!(program, Ok(Binding::Int(15))));
}
