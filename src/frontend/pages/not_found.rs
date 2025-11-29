//! 404 Not Found page for RustPress

use leptos::prelude::*;

/// 404 Not Found page component
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center px-6 bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
            <div class="text-center">
                <h1 class="text-8xl md:text-9xl font-bold bg-gradient-to-r from-orange-400 to-amber-400 bg-clip-text text-transparent mb-4">
                    "404"
                </h1>
                <p class="text-2xl font-semibold text-white mb-2">"Page not found"</p>
                <p class="text-slate-400 mb-8 max-w-md">
                    "The page you're looking for doesn't exist or has been moved."
                </p>
                <a href="/" class="btn-primary">"Return Home"</a>
            </div>
        </div>
    }
}
