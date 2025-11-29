//! Admin page for RustPress

use leptos::prelude::*;

/// Admin page component
#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-slate-950">
            // Admin Navigation
            <nav class="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm">
                <div class="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
                    <div class="flex items-center gap-3">
                        <span class="text-2xl">"🦀"</span>
                        <span class="text-lg font-semibold text-white">"RustPress Admin"</span>
                    </div>
                    <a href="/" class="btn-ghost text-sm px-4 py-2">"← Back to Site"</a>
                </div>
            </nav>

            // Admin Content
            <div class="max-w-6xl mx-auto px-6 py-12">
                <div class="mb-10">
                    <h1 class="text-3xl font-bold text-white mb-2">"Admin Console"</h1>
                    <p class="text-slate-400">"The admin dashboard is coming soon."</p>
                </div>

                // Stats Grid
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div class="bg-slate-900/50 border border-slate-800 rounded-xl p-6 hover:border-slate-700 transition-colors">
                        <span class="text-3xl mb-3 block">"📝"</span>
                        <span class="text-4xl font-bold text-white block mb-1">"0"</span>
                        <span class="text-slate-400 text-sm">"Posts"</span>
                    </div>
                    <div class="bg-slate-900/50 border border-slate-800 rounded-xl p-6 hover:border-slate-700 transition-colors">
                        <span class="text-3xl mb-3 block">"📄"</span>
                        <span class="text-4xl font-bold text-white block mb-1">"0"</span>
                        <span class="text-slate-400 text-sm">"Pages"</span>
                    </div>
                    <div class="bg-slate-900/50 border border-slate-800 rounded-xl p-6 hover:border-slate-700 transition-colors">
                        <span class="text-3xl mb-3 block">"👥"</span>
                        <span class="text-4xl font-bold text-white block mb-1">"1"</span>
                        <span class="text-slate-400 text-sm">"Users"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
