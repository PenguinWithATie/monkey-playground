use std::rc::Rc;

use chrono::Local;
use leptos::prelude::*;

use crate::monkey::{
    evaluator::{Env, Evaluation},
    vm::{Compilation, CompiledContext, Machine},
    Lexer, Parser, Program,
};

use super::code_snips::*;
const EVAL_STYLE: &str = "hover:text-white border border-green-700 hover:bg-green-800 focus:ring-4 focus:outline-none focus:ring-green-300 font-medium rounded-lg text-sm px-5 py-1.5 text-center me-2 mb-2";
const SNIP_STYLE: &str = "bg-transparent hover:bg-blue-500 text-blue-700 font-semibold hover:text-white py-2 px-4 border border-blue-500 hover:border-transparent rounded";

#[derive(Default, Clone)]
pub struct RunResult {
    pub result: String,
    pub time: i64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EngineType {
    Evaluator,
    VM,
    Both,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EvalMode {
    Repl,
    Runner,
}

#[component]
pub fn EngineSelector(
    engine_type: RwSignal<EngineType>,
    #[prop(optional)] eval: Option<RwSignal<RunResult>>,
    #[prop(optional)] vm: Option<RwSignal<RunResult>>,
    #[prop(optional)] repl: Option<RwSignal<String>>,
) -> impl IntoView {
    let eval_style = move |selected: EngineType| {
        if engine_type() == selected {
            EVAL_STYLE.to_string() + " bg-green-800 text-white"
        } else {
            EVAL_STYLE.to_string() + " text-green-700"
        }
    };
    view! {
        <div class="flex">
            <button
                type="button"
                class=move || eval_style(EngineType::Evaluator)
                on:click=move |_| {
                    if engine_type() != EngineType::Evaluator {
                        if let Some(repl) = repl {
                            repl.set("".to_string());
                        }
                    engine_type.set(EngineType::Evaluator);
                    if let Some(eval) = eval {
                        eval.set(RunResult::default());
                    }
                }
                }
            >
                "Evaluator"
            </button>
            <button
                type="button"
                class=move || eval_style(EngineType::VM)
                on:click=move |_| {
                    engine_type.set(EngineType::VM);
                    if let Some(vm) = vm {
                        vm.set(RunResult::default());
                    }
                }
            >
                "VM"
            </button>
            <Show when=move || vm.is_some() && eval.is_some()>
                <button
                    type="button"
                    class=move || eval_style(EngineType::Both)
                    on:click=move |_| {
                        engine_type.set(EngineType::Both);
                        if let Some(vm) = vm {
                            vm.set(RunResult::default());
                        }
                        if let Some(eval) = eval {
                            eval.set(RunResult::default());
                        }
                    }
                >
                    "Both"
                </button>
            </Show>
        </div>
    }
}

#[component]
pub fn SnippetSetter(set_text: WriteSignal<String>) -> impl IntoView {
    view! {
        <span class="m-2 font-bold">"Snippets"</span>
        <button class=SNIP_STYLE on:click=move |_| set_text(FIB_CODE.to_string())>
            "Fibonacci"
        </button>
        <button class=SNIP_STYLE on:click=move |_| set_text(FIZZBUZZ_CODE.to_string())>
            "FizzBuzz"
        </button>
        <button class=SNIP_STYLE on:click=move |_| set_text(DOUBLE_W_MAP.to_string())>
            "Double map"
        </button>
    }
}

#[component]
pub fn ModeSelector(mode: RwSignal<EvalMode>) -> impl IntoView {
    view! {
        <label class="inline-flex items-center cursor-pointer">
            <input
                type="checkbox"
                value=""
                class="sr-only peer"
                on:click=move |_| {
                    if mode() == EvalMode::Repl {
                        mode.set(EvalMode::Runner);
                    } else {
                        mode.set(EvalMode::Repl);
                    }
                }
            />
            <span class="mx-2 text-sm font-medium text-gray-900 ms-3">"Runner mode"</span>
            <div class="relative w-9 h-5 bg-green-600 peer-focus:outline-none  rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
            <span class="mx-2 text-sm font-medium text-gray-900 ms-3">"REPL mode (CLI)"</span>
        </label>
    }
}

pub fn generate_program(text: String) -> (Program, i64) {
    let timer = Local::now();
    let lexer = Lexer::new(text.bytes());
    let mut parser = Parser::new(lexer);
    let timer = (Local::now() - timer).num_milliseconds();
    (parser.program().unwrap(), timer)
}
pub fn eval_engine(program: &Program) -> RunResult {
    let env = Rc::new(Env::default());
    let timer = Local::now();
    let _ = program.eval(&env).unwrap();
    let timer = (Local::now() - timer).num_milliseconds();
    RunResult {
        result: format!("{}", env.stdout.borrow()),
        time: timer,
    }
}

pub fn bytecode_engine(program: &Program) -> RunResult {
    let mut ctx = CompiledContext::default();
    let mut machine = Machine::default();
    let timer = Local::now();
    program.compile(&mut ctx);
    machine.run(ctx.get_constants(), ctx.make_main_closure());
    let timer = (Local::now() - timer).num_milliseconds();
    RunResult {
        result: machine.get_stdout(),
        time: timer,
    }
}
