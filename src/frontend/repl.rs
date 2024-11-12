use leptos::{logging, prelude::*, tachys::reactive_graph::bind};
use std::{fmt::Write, rc::Rc};

use crate::{
    frontend::EngineSelector,
    monkey::{
        evaluator::{self, Evaluation},
        vm::{self, Compilation},
    },
};

use super::{utils::generate_program, EngineType};

#[component]
pub fn Repl(engine_type: RwSignal<EngineType>) -> impl IntoView {
    let output = RwSignal::new(String::new());
    let (input, input_set) = signal(String::new());
    if engine_type() == EngineType::Both {
        engine_type.set(EngineType::VM);
    }
    let mut machine = vm::Machine::default();
    let mut ctx = vm::CompiledContext::default();
    let env = Rc::new(evaluator::Env::default());
    view! {
        <EngineSelector engine_type />
        <pre id="output" class="w-full h-4/5 font-mono">
            {output}
        </pre>
        <div class="flex flex-col">
            <div class="flex flex-row">
                <label for="input" class="mx-2 h-1/5 font-mono">
                    ">>"
                </label>
                <input
                    type="text"
                    id="input"
                    class="py-1 w-5/6 font-mono"
                    placeholder="type an expression here"
                    prop:value=input
                    on:input=move |ev| input_set(event_target_value(&ev))
                    on:keydown=move |e| {
                        if e.key() == "Enter" && !input().is_empty() {
                            match engine_type() {
                                EngineType::Evaluator => {
                                    run_eval(input(), output, &env);
                                }
                                EngineType::VM => {
                                    ctx.clear_instructions();
                                    run_vm(input(), output, &mut machine, &mut ctx);
                                }
                                EngineType::Both => unreachable!(),
                            }
                            //output.update(|s| s.write_fmt(format_args!("{}\n", input())).unwrap());
                            input_set(String::new());
                        }
                    }
                />
            </div>
        </div>
    }
}

fn run_vm(
    input: String,
    output: RwSignal<String>,
    machine: &mut vm::Machine,
    ctx: &mut vm::CompiledContext,
) {
    let (program, _time) = generate_program(input);
    program.compile(ctx);
    ctx.remove_last_pop();
    machine.run(ctx.get_constants(), ctx.make_main_closure());
    let binding = machine.get_last_expr();
    output.update(|s| s.write_fmt(format_args!("{}\n", binding)).unwrap());
}

fn run_eval(input: String, output: RwSignal<String>, env: &Rc<evaluator::Env>) {
    let (program, _time) = generate_program(input);
    let binding = program.eval(env).unwrap();
    logging::log!("{}", binding);
    output.update(|s| s.write_fmt(format_args!("{}\n", binding)).unwrap());
}
