use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-slate-800 bg-slate-900/50">
            <div class="max-w-6xl mx-auto px-6 py-12 text-center">
                <p class="text-2xl font-bold mb-2">"🦀 RustPress"</p>
                <p class="text-slate-400 mb-6">"Modern CMS. Built with Rust."</p>
                <p class="text-xs text-slate-600">"© 2024 RustPress Team. MIT License."</p>
            </div>
        </footer>
    }
}
