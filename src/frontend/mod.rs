//! Frontend module for RustPress
//!
//! This module contains all frontend components organized into:
//! - `components/` - Reusable UI components (Nav, Footer, FeatureCard, etc.)
//! - `pages/` - Page-level components (Landing, Login, Register, Admin, etc.)

pub mod components;
pub mod pages;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use pages::{AdminPage, LandingPage, LoginPage, NotFound, RegisterPage};

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
