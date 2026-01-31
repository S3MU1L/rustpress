pub mod components;
pub mod pages;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use pages::{LandingPage, LoginPage, NotFound, RegisterPage};

#[component]
fn LoginRoute() -> impl IntoView {
    view! { <LoginPage/> }
}

#[component]
fn RegisterRoute() -> impl IntoView {
    view! { <RegisterPage/> }
}

#[component]
pub fn Document(#[prop(into)] title: String, children: Children) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="scroll-smooth">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin/>
                <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet"/>
                <script src="https://cdn.tailwindcss.com"></script>
                <link rel="stylesheet" href="/static/app.css"/>
                <title>{title}</title>
            </head>
            <body class="bg-slate-950 text-slate-100 antialiased">
                {children()}
            </body>
        </html>
    }
}

#[cfg(feature = "ssr")]
pub fn render_login_page(error: Option<String>) -> String {
    let error = error.unwrap_or_default();
    let owner = Owner::new_root(None);
    owner.with(|| {
        view! {
            <Document title="RustPress - Login">
                <LoginPage error=error.clone() />
            </Document>
        }
        .to_html()
    })
}

#[cfg(feature = "ssr")]
pub fn render_register_page(error: Option<String>) -> String {
    let error = error.unwrap_or_default();
    let owner = Owner::new_root(None);
    owner.with(|| {
        view! {
            <Document title="RustPress - Register">
                <RegisterPage error=error.clone() />
            </Document>
        }
        .to_html()
    })
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en" class="scroll-smooth">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin/>
                <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="bg-slate-950 text-slate-100 antialiased">
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/rustpress.css"/>
        <Title text="RustPress - Modern CMS Built with Rust"/>
        <Meta name="description" content="A high-performance, concurrent Content Management System built entirely in Rust"/>

        <Router>
            <main class="min-h-screen">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("/") view=LandingPage/>
                    <Route path=path!("/login") view=LoginRoute/>
                    <Route path=path!("/register") view=RegisterRoute/>
                </Routes>
            </main>
        </Router>
    }
}
