mod monkey;
use monkey::{
    evaluator::{Env, Evaluation},
    lexer::Lexer,
    parser::Parser,
    vm::{Compile, CompiledContext, Machine},
};
use std::{io::Write, rc::Rc, time::SystemTime};
fn main() {
    let mut machine = Machine::default();
    let mut ctx = CompiledContext::default();
    let env = Rc::new(Env::default());
    println!("Here's a REPL for monkeylang");
    loop {
        let mut input = String::new();
        print!(">> ");
        std::io::stdout().flush().expect("Failed to flush stdout");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let timer = SystemTime::now();
        let lexer = Lexer::new(input.bytes());
        let program = Parser::new(lexer).program();
        println!("Time taken for lexing and parsing: {:?}", timer.elapsed());
        let timer = SystemTime::now();
        match program {
            Ok(program) => {
                let result_eval = program.eval(&env);
                println!("Result: {:?}", result_eval);
                println!("Evaluator took {:?}", timer.elapsed());
                println!("-----------------------------");
                ctx.clear_instructions();
                let timer = SystemTime::now();
                program.compile(&mut ctx);
                println!("Compilation took {:?}", timer.elapsed());
                let timer = SystemTime::now();
                machine.run(ctx.get_constants(), ctx.make_main_closure());
                println!("Execution took {:?}", timer.elapsed());
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
