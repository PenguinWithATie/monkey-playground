use leptos::prelude::*;
mod frontend;
mod monkey;
use frontend::{EngineType, EvalMode, ModeSelector, Repl, Runner, SnippetSetter, FIB_CODE};

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let engine_type = RwSignal::new(EngineType::Both);
    let (text, set_text) = signal::<String>(FIB_CODE.to_string());
    let mode = RwSignal::new(EvalMode::Runner);
    view! {
        <div class="flex gap-2 m-4">
            <ModeSelector mode />
            <Show when=move || mode() == EvalMode::Runner>
                <SnippetSetter set_text />
            </Show>
        </div>
        <Show when=move || mode() == EvalMode::Runner>
            <Runner text set_text engine_type />
        </Show>
        <Show when=move || mode() == EvalMode::Repl>
            <Repl engine_type />
        </Show>
    }
}
