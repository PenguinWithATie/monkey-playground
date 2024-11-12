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
        <div class="flex h-screen m-4 gap-2">
            <div class="flex flex-col h-full">
                <textarea
                    prop:value=text
                    on:input=move |ev| { set_text(event_target_value(&ev)) }
                    cols=80
                    class="h-4/5 font-mono"
                />
                <button
                    class="m-2 bg-orange-600 hover:bg-orange-800 text-white font-bold py-2 px-4 rounded"
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
            <div class="flex flex-col">
                <EngineSelector engine_type eval vm />
                <div class="font-bold text-xl m-2">"Lexing and Parsing took "{parse_time}" ms"</div>
                <Show when=move || {
                    matches!(engine_type(), EngineType::Both | EngineType::Evaluator)
                }>
                    <div class="font-bold text-xl m-2">
                        "Evaluator result(took " {move || eval().time} " ms)"
                    </div>
                    <pre class="m-2 p-4 bg-gray-100 rounded overflow-auto max-h-72">
                        {move || eval().result}
                    </pre>
                </Show>
                <Show when=move || matches!(engine_type(), EngineType::Both | EngineType::VM)>
                    <div class="font-bold text-xl m-2">
                        "VM result (took " {move || vm().time} " ms)"
                    </div>
                    <pre class="m-2 p-4 bg-gray-100 rounded overflow-auto max-h-72">
                        {move || vm().result}
                    </pre>
                </Show>
            </div>
        </div>
    }
}
