//! Login page for RustPress

use leptos::prelude::*;

/// Login page component
#[component]
pub fn LoginPage() -> impl IntoView {
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
