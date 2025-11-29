use leptos::prelude::*;

use crate::frontend::components::{FeatureCard, Footer, Nav};

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex flex-col">
            <Nav/>

            <section class="pt-32 pb-20 px-6 flex flex-col items-center text-center">
                <div class="w-full max-w-2xl mb-12 rounded-xl overflow-hidden border border-slate-700 bg-slate-900 shadow-2xl shadow-black/50">
                    <div class="flex items-center gap-2 px-4 py-3 bg-slate-800 border-b border-slate-700">
                        <span class="w-3 h-3 rounded-full bg-red-500"></span>
                        <span class="w-3 h-3 rounded-full bg-yellow-500"></span>
                        <span class="w-3 h-3 rounded-full bg-green-500"></span>
                        <span class="ml-4 text-xs text-slate-500 font-mono">"rustpress — bash"</span>
                    </div>
                    // Terminal Body
                    <div class="p-6 font-mono text-sm text-left space-y-1">
                        <p>
                            <span class="text-green-400">"$ "</span>
                            <span class="text-slate-300">"cargo install rustpress"</span>
                        </p>
                        <p class="text-slate-500">"    Compiling rustpress v0.1.0"</p>
                        <p class="text-slate-500">"    Finished release [optimized] target(s)"</p>
                        <p>
                            <span class="text-green-400">"$ "</span>
                            <span class="text-slate-300">"rustpress serve"</span>
                        </p>
                        <p class="text-emerald-400">"🚀 RustPress running at http://localhost:8080"</p>
                        <p>
                            <span class="text-green-400 animate-blink">"$ "</span>
                            <span class="text-orange-400 animate-blink">"_"</span>
                        </p>
                    </div>
                </div>

                <h1 class="text-5xl md:text-6xl font-bold mb-6 leading-tight">
                    "Content Management"<br/>
                    <span class="bg-gradient-to-r from-orange-400 via-amber-400 to-orange-500 bg-clip-text text-transparent">
                        "Blazingly Fast"
                    </span>
                </h1>
                <p class="text-xl text-slate-400 max-w-2xl mb-10">
                    "A modern CMS built entirely in Rust. Memory-safe, concurrent, and incredibly fast."
                </p>

                <div class="flex flex-wrap gap-4 justify-center">
                    <a href="/register" class="btn-primary btn-large">"Start Building"</a>
                    <a
                        href="https://github.com/S3MU1L/rustpress"
                        target="_blank"
                        class="btn-ghost btn-large"
                    >
                        "View on GitHub"
                    </a>
                </div>
            </section>

            // Features Section
            <section id="features" class="py-20 px-6 bg-slate-900/30">
                <div class="max-w-6xl mx-auto">
                    <h2 class="text-3xl md:text-4xl font-bold text-center mb-12">
                        "Why "<span class="text-orange-400">"RustPress"</span>"?"
                    </h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                        <FeatureCard
                            icon="⚡"
                            title="Lightning Fast"
                            desc="Built with Rust and Actix for maximum performance. Handle thousands of requests with minimal latency."
                        />
                        <FeatureCard
                            icon="🔒"
                            title="Memory Safe"
                            desc="No null pointers, no data races. Rust's guarantees mean your CMS stays stable and secure."
                        />
                        <FeatureCard
                            icon="🎨"
                            title="Template System"
                            desc="Wide range of built-in templates with full support for custom designs and themes."
                        />
                        <FeatureCard
                            icon="🔧"
                            title="Extensible"
                            desc="Modular architecture makes customization and extension straightforward."
                        />
                    </div>
                </div>
            </section>

            // About Section
            <section id="about" class="py-20 px-6">
                <div class="max-w-3xl mx-auto text-center">
                    <h2 class="text-3xl md:text-4xl font-bold mb-8">
                        "Built for "<span class="text-orange-400">"Developers"</span>
                    </h2>
                    <p class="text-lg text-slate-400 leading-relaxed mb-10">
                        "RustPress is a high-performance, concurrent Content Management System developed entirely in Rust. "
                        "Inspired by traditional WordPress, RustPress brings the power of Rust's memory safety and concurrency "
                        "to web content management, offering a modern, fast, and reliable platform."
                    </p>
                    <div class="flex flex-wrap gap-3 justify-center">
                        <span class="px-4 py-2 rounded-full bg-orange-500/10 text-orange-400 border border-orange-500/30 text-sm font-medium">
                            "Rust"
                        </span>
                        <span class="px-4 py-2 rounded-full bg-purple-500/10 text-purple-400 border border-purple-500/30 text-sm font-medium">
                            "Actix"
                        </span>
                        <span class="px-4 py-2 rounded-full bg-sky-500/10 text-sky-400 border border-sky-500/30 text-sm font-medium">
                            "Leptos"
                        </span>
                        <span class="px-4 py-2 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 text-sm font-medium">
                            "Tailwind"
                        </span>
                    </div>
                </div>
            </section>

            <div class="flex-grow"></div>
            <Footer/>
        </div>
    }
}
