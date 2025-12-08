use leptos::prelude::*;

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
                <a
                    href="/"
                    class="inline-flex items-center justify-center px-8 py-4 text-lg font-semibold rounded-lg
                           bg-gradient-to-r from-orange-500 to-amber-500 text-white
                           hover:from-orange-600 hover:to-amber-600
                           hover:shadow-lg hover:shadow-orange-500/25
                           transition-all duration-200"
                >
                    "Return Home"
                </a>
            </div>
        </div>
    }
}
