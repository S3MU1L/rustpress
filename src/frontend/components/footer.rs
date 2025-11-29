//! Footer component for RustPress

use leptos::prelude::*;

/// Main footer component
#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="footer-content">
                <p class="footer-brand">"🦀 RustPress"</p>
                <p class="footer-tagline">"Modern CMS. Built with Rust."</p>
                <div class="footer-links">
                    <a href="https://github.com/S3MU1L/rustpress" target="_blank">"GitHub"</a>
                    <span class="divider">"|"</span>
                    <a href="/admin">"Admin Console"</a>
                    <span class="divider">"|"</span>
                    <a href="https://docs.rs" target="_blank">"Documentation"</a>
                </div>
                <p class="copyright">"© 2024 RustPress Team. MIT License."</p>
            </div>
        </footer>
    }
}
