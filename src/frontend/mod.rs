pub mod landing;

use landing::LandingPage;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

/// HTML shell for SSR - provides the full document structure
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// Main application component with routing
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/rustpress.css"/>
        <Title text="RustPress - Modern CMS Built with Rust"/>
        <Meta name="description" content="A high-performance, concurrent Content Management System built entirely in Rust"/>

        <Router>
            <main>
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=LandingPage/>
                    <Route path=path!("/login") view=LoginPage/>
                    <Route path=path!("/register") view=RegisterPage/>
                    <Route path=path!("/admin") view=AdminPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// 404 Not Found page
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="not-found">
            <div class="not-found-content">
                <h1 class="not-found-code">"404"</h1>
                <p class="not-found-message">"Page not found"</p>
                <p class="not-found-desc">"The page you're looking for doesn't exist or has been moved."</p>
                <a href="/" class="btn-primary">"Return Home"</a>
            </div>
        </div>
    }
}

/// Login page placeholder - will be implemented fully later
#[component]
fn LoginPage() -> impl IntoView {
    view! {
        <div class="auth-page">
            <div class="auth-container">
                <h1>"Login"</h1>
                <p class="auth-subtitle">"Welcome back to RustPress"</p>
                <form class="auth-form">
                    <div class="form-group">
                        <label for="email">"Email"</label>
                        <input type="email" id="email" name="email" placeholder="you@example.com" required/>
                    </div>
                    <div class="form-group">
                        <label for="password">"Password"</label>
                        <input type="password" id="password" name="password" placeholder="••••••••" required/>
                    </div>
                    <button type="submit" class="btn-primary btn-full">"Sign In"</button>
                </form>
                <p class="auth-link">"Don't have an account? "<a href="/register">"Sign up"</a></p>
                <a href="/" class="back-link">"← Back to home"</a>
            </div>
        </div>
    }
}

/// Register page placeholder - will be implemented fully later
#[component]
fn RegisterPage() -> impl IntoView {
    view! {
        <div class="auth-page">
            <div class="auth-container">
                <h1>"Create Account"</h1>
                <p class="auth-subtitle">"Get started with RustPress"</p>
                <form class="auth-form">
                    <div class="form-group">
                        <label for="username">"Username (optional)"</label>
                        <input type="text" id="username" name="username" placeholder="rustacean"/>
                    </div>
                    <div class="form-group">
                        <label for="email">"Email"</label>
                        <input type="email" id="email" name="email" placeholder="you@example.com" required/>
                    </div>
                    <div class="form-group">
                        <label for="password">"Password"</label>
                        <input type="password" id="password" name="password" placeholder="••••••••" required/>
                    </div>
                    <button type="submit" class="btn-primary btn-full">"Create Account"</button>
                </form>
                <p class="auth-link">"Already have an account? "<a href="/login">"Sign in"</a></p>
                <a href="/" class="back-link">"← Back to home"</a>
            </div>
        </div>
    }
}

/// Admin page placeholder - will be expanded later
#[component]
fn AdminPage() -> impl IntoView {
    view! {
        <div class="admin-page">
            <nav class="admin-nav">
                <div class="admin-brand">
                    <span>"🦀"</span>
                    <span>"RustPress Admin"</span>
                </div>
                <a href="/" class="btn-ghost">"← Back to Site"</a>
            </nav>
            <div class="admin-content">
                <h1>"Admin Console"</h1>
                <p>"The admin dashboard is coming soon."</p>
                <div class="admin-stats">
                    <div class="stat-card">
                        <span class="stat-icon">"📝"</span>
                        <span class="stat-value">"0"</span>
                        <span class="stat-label">"Posts"</span>
                    </div>
                    <div class="stat-card">
                        <span class="stat-icon">"📄"</span>
                        <span class="stat-value">"0"</span>
                        <span class="stat-label">"Pages"</span>
                    </div>
                    <div class="stat-card">
                        <span class="stat-icon">"👥"</span>
                        <span class="stat-value">"1"</span>
                        <span class="stat-label">"Users"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
