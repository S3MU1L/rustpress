//! 404 Not Found page for RustPress

use leptos::prelude::*;

/// 404 Not Found page component
#[component]
pub fn NotFound() -> impl IntoView {
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
