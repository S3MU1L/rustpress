//! Navigation component for RustPress

use leptos::prelude::*;

/// Main navigation bar component
#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="nav">
            <div class="nav-brand">
                <span class="logo">"🦀"</span>
                <span class="brand-text">"RustPress"</span>
            </div>
            <div class="nav-links">
                <a href="#features">"Features"</a>
                <a href="#about">"About"</a>
                <a href="/login" class="btn-ghost">"Login"</a>
                <a href="/register" class="btn-primary">"Get Started"</a>
            </div>
        </nav>
    }
}
