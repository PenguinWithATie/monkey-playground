use super::utils::{bytecode_engine, eval_engine, generate_program, RunResult};
use super::EngineType;
use crate::frontend::EngineSelector;
use leptos::prelude::*;
#[component]
pub fn Runner(
    text: ReadSignal<String>,
    set_text: WriteSignal<String>,
    engine_type: RwSignal<EngineType>,
) -> impl IntoView {
    let vm = RwSignal::new(RunResult::default());
    let eval = RwSignal::new(RunResult::default());
    let parse_time = RwSignal::new(0);
    view! {
        <div class="flex gap-2 m-4 h-screen">
            <div class="flex flex-col h-full">
                <textarea
                    prop:value=text
                    on:input=move |ev| { set_text(event_target_value(&ev)) }
                    cols=65
                    class="h-3/5 font-mono bg-gray-100"
                />
                <button
                    class="px-4 py-2 m-2 font-bold text-white bg-orange-600 rounded hover:bg-orange-800"
                    on:click=move |_| {
                        let (program, timer) = generate_program(text());
                        parse_time.set(timer);
                        if matches!(engine_type(), EngineType::Both | EngineType::VM) {
                            let result = bytecode_engine(&program);
                            vm.set(result);
                        }
                        if matches!(engine_type(), EngineType::Both | EngineType::Evaluator) {
                            let result = eval_engine(&program);
                            eval.set(result);
                        }
                    }
                >
                    "Run"
                </button>
            </div>
            <div class="flex flex-col w-full">
                <EngineSelector engine_type eval vm />
                <div class="m-2 text-xl font-bold">"Lexing and Parsing took "{parse_time}" ms"</div>
                <Show when=move || {
                    matches!(engine_type(), EngineType::Both | EngineType::Evaluator)
                }>
                    <div class="m-2 text-xl font-bold">
                        "Evaluator result(took " {move || eval().time} " ms)"
                    </div>
                    <pre class="overflow-auto p-4 m-2 max-h-72 bg-gray-100 rounded">
                        {move || eval().result}
                    </pre>
                </Show>
                <Show when=move || matches!(engine_type(), EngineType::Both | EngineType::VM)>
                    <div class="m-2 text-xl font-bold">
                        "VM result (took " {move || vm().time} " ms)"
                    </div>
                    <pre class="overflow-auto p-4 m-2 max-h-72 bg-gray-100 rounded">
                        {move || vm().result}
                    </pre>
                </Show>
            </div>
        </div>
    }
}
