//! Feature card component for displaying feature highlights

use leptos::prelude::*;

/// Feature card component for the features section
#[component]
pub fn FeatureCard(icon: &'static str, title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="feature-card">
            <span class="feature-icon">{icon}</span>
            <h3>{title}</h3>
            <p>{desc}</p>
        </div>
    }
}
