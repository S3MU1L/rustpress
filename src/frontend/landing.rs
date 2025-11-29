use leptos::prelude::*;

/// Feature card component for the features section
#[component]
fn FeatureCard(icon: &'static str, title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="feature-card">
            <span class="feature-icon">{icon}</span>
            <h3>{title}</h3>
            <p>{desc}</p>
        </div>
    }
}

/// Main landing page component with dark terminal-inspired theme
#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <div class="landing">
            // Navigation
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

            // Hero Section
            <section class="hero">
                <div class="terminal-window">
                    <div class="terminal-header">
                        <span class="dot red"></span>
                        <span class="dot yellow"></span>
                        <span class="dot green"></span>
                        <span class="terminal-title">"rustpress — bash"</span>
                    </div>
                    <div class="terminal-body">
                        <p><span class="prompt">"$ "</span>"cargo install rustpress"</p>
                        <p class="output">"    Compiling rustpress v0.1.0"</p>
                        <p class="output">"    Finished release [optimized] target(s)"</p>
                        <p><span class="prompt">"$ "</span>"rustpress serve"</p>
                        <p class="success">"🚀 RustPress running at http://localhost:8080"</p>
                        <p><span class="prompt blink">"$ "</span><span class="cursor">"_"</span></p>
                    </div>
                </div>

                <h1 class="hero-title">
                    "Content Management"<br/>
                    <span class="highlight">"Blazingly Fast"</span>
                </h1>
                <p class="hero-subtitle">
                    "A modern CMS built entirely in Rust. Memory-safe, concurrent, and incredibly fast."
                </p>

                <div class="hero-cta">
                    <a href="/register" class="btn-primary btn-large">"Start Building"</a>
                    <a href="https://github.com/S3MU1L/rustpress" target="_blank" class="btn-ghost btn-large">
                        "View on GitHub"
                    </a>
                </div>
            </section>

            // Features Section
            <section id="features" class="features">
                <h2 class="section-title">"Why RustPress?"</h2>
                <div class="feature-grid">
                    <FeatureCard
                        icon="⚡"
                        title="Lightning Fast"
                        desc="Built with Rust and Actix for maximum performance. Handle thousands of requests with minimal latency."
                    />
                    <FeatureCard
                        icon="🔒"
                        title="Memory Safe"
                        desc="No null pointers, no data races. Rust's guarantees mean your CMS stays stable and secure."
                    />
                    <FeatureCard
                        icon="🎨"
                        title="Template System"
                        desc="Wide range of built-in templates with full support for custom designs and themes."
                    />
                    <FeatureCard
                        icon="🔧"
                        title="Extensible"
                        desc="Modular architecture makes customization and extension straightforward."
                    />
                </div>
            </section>

            // About Section
            <section id="about" class="about">
                <div class="about-content">
                    <h2 class="section-title">"Built for Developers"</h2>
                    <p class="about-text">
                        "RustPress is a high-performance, concurrent Content Management System developed entirely in Rust. "
                        "Inspired by traditional WordPress, RustPress brings the power of Rust's memory safety and concurrency "
                        "to web content management, offering a modern, fast, and reliable platform."
                    </p>
                    <div class="tech-stack">
                        <span class="tech-badge">"Rust"</span>
                        <span class="tech-badge">"Actix"</span>
                        <span class="tech-badge">"Leptos"</span>
                        <span class="tech-badge">"HTMX"</span>
                    </div>
                </div>
            </section>

            // Footer
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
        </div>
    }
}
