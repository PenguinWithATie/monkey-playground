use std::rc::Rc;

use leptos::{ logging, prelude::*};
mod monkey;
use monkey::{
    Lexer,Parser, Program,
    evaluator::{Env, Evaluation},
    vm::{Compilation, CompiledContext, Machine},
};
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (text, set_text) = signal("Controlled".to_string());
    let input = move |ev| {
        set_text(event_target_value(&ev));
    };
    let (result, result_setter) = signal("Result".to_string());
    view! {
        <textarea id="input" on:input=input rows=20 cols=80>
            {text}
        </textarea>
        <br />
        <button on:click=move |_| {
            logging::log!("clicked!");
            monkey_eval_engine(text(), result_setter);
        }>
            "Run"
        </button>
        <div>
        <h2>"Result"</h2>
        <p>{result}</p>
        </div>
    }
}


fn generate_program(text: String) -> Program {
    let lexer = Lexer::new(text.bytes());
    let mut parser = Parser::new(lexer);
    parser.program().unwrap()
}
fn monkey_eval_engine(text: String, setter: WriteSignal<String>) {
    let program = generate_program(text);
    let env = Rc::new(Env::default());
    let binding = program.eval(&env).unwrap();
    setter.set(format!("{}", binding));
}

fn monkey_bytecode_engine(text: String, setter: WriteSignal<String>) {
    let program = generate_program(text);
    let mut ctx = CompiledContext::default();
    let mut machine = Machine::default();
    program.compile(&mut ctx);
    machine.run(ctx.get_constants(), ctx.make_main_closure());
    //setter.set(format!("{}", binding));
}
