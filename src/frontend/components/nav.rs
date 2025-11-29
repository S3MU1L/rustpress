use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="fixed top-0 left-0 right-0 z-50 backdrop-blur-md bg-slate-950/80 border-b border-slate-800">
            <div class="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
                <a href="/" class="flex items-center gap-3 group">
                    <span class="text-3xl group-hover:animate-bounce">"🦀"</span>
                    <span class="text-xl font-bold bg-gradient-to-r from-orange-400 to-amber-400 bg-clip-text text-transparent">
                        "RustPress"
                    </span>
                </a>
                <div class="flex items-center gap-6">
                    <a href="#features" class="text-slate-400 hover:text-white transition-colors">"Features"</a>
                    <a href="#about" class="text-slate-400 hover:text-white transition-colors">"About"</a>
                    <a href="/login" class="btn-ghost text-sm px-4 py-2">"Login"</a>
                    <a href="/register" class="btn-primary text-sm px-4 py-2">"Get Started"</a>
                </div>
            </div>
        </nav>
    }
}
