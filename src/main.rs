use leptos::prelude::*;
mod frontend;
mod monkey;
use frontend::{EngineType, EvalMode, ModeSelector, Runner, SnippetSetter, FIB_CODE};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let engine_type = RwSignal::new(EngineType::Both);
    let (text, set_text) = signal::<String>(FIB_CODE.to_string());
    let mode = RwSignal::new(EvalMode::Runner);
    view! {
        <div class="m-4 flex gap-2">
            <ModeSelector mode />
            <SnippetSetter set_text />
        </div>
        <Runner text set_text engine_type />
    }
}
