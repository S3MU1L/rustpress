//! Admin page for RustPress

use leptos::prelude::*;

/// Admin page component
#[component]
pub fn AdminPage() -> impl IntoView {
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
