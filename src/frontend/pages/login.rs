use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="min-h-screen flex items-center justify-center px-6 py-12 bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
            <div class="w-full max-w-md">
                <div class="bg-slate-900/80 backdrop-blur-sm border border-slate-800 rounded-2xl p-8 shadow-xl">
                    <div class="text-center mb-8">
                        <a href="/" class="inline-block text-4xl mb-4 hover:animate-bounce">"🦀"</a>
                        <h1 class="text-2xl font-bold text-white">"Login"</h1>
                        <p class="text-slate-400 mt-2">"Welcome back to RustPress"</p>
                    </div>

                    <form class="space-y-6">
                        <div>
                            <label for="email" class="block text-sm font-medium text-slate-300 mb-2">
                                "Email"
                            </label>
                            <input
                                type="email"
                                id="email"
                                name="email"
                                placeholder="you@example.com"
                                required
                                class="w-full px-4 py-3 rounded-lg bg-slate-800 border border-slate-700
                                       text-white placeholder-slate-500
                                       focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-transparent
                                       transition-all"
                            />
                        </div>
                        <div>
                            <label for="password" class="block text-sm font-medium text-slate-300 mb-2">
                                "Password"
                            </label>
                            <input
                                type="password"
                                id="password"
                                name="password"
                                placeholder="••••••••"
                                required
                                class="w-full px-4 py-3 rounded-lg bg-slate-800 border border-slate-700
                                       text-white placeholder-slate-500
                                       focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-transparent
                                       transition-all"
                            />
                        </div>
                        <button type="submit" class="btn-primary w-full">"Sign In"</button>
                    </form>

                    <p class="text-center text-slate-400 mt-6 text-sm">
                        "Don't have an account? "
                        <a href="/register" class="text-orange-400 hover:text-orange-300 font-medium">"Sign up"</a>
                    </p>
                </div>

                <a href="/" class="block text-center text-slate-500 hover:text-slate-300 mt-6 text-sm transition-colors">
                    "← Back to home"
                </a>
            </div>
        </div>
    }
}
