//! Register page for RustPress

use leptos::prelude::*;

/// Register page component
#[component]
pub fn RegisterPage() -> impl IntoView {
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
