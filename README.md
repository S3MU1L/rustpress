# RustPress

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Actix](https://img.shields.io/badge/actix-web-blue)](https://actix.rs/)

**A modern and concurrent Content Management System (CMS) built entirely in Rust**

</div>

## 🚀 Overview

RustPress is a high-performance, concurrent Content Management System (CMS) developed entirely in Rust. Inspired by traditional WordPress, RustPress brings the power of Rust's memory safety and concurrency to web content management, offering a modern, fast, and reliable platform for managing your website content.

## ✨ Features

- **Template System**: Wide range of built-in templates with support for custom template creation
- **Content Management**: Easy-to-use interface for modifying website content, creating pages, and publishing articles
- **Dual Interface**: User-facing website and admin console (similar to WordPress)
- **Extensible**: Modular architecture for easy customization and extension

## 🏗️ Technology Stack

- **[Actix-web](https://actix.rs/)**: Fast, pragmatic web framework
- **[HTMX](https://htmx.org/)**: Modern, hypermedia-driven frontend
- **[Serde](https://serde.rs/)**: Serialization/deserialization framework
- **[Tokio](https://tokio.rs/)**: Asynchronous runtime
- **[Leptos](https://leptos.dev/)**: Rust-based UI components (see `src/frontend/`)

## 📋 Prerequisites

- Rust 1.70.0 or higher
- Cargo (comes with Rust)

## 🚀 Quick Start

### Installation

1. Clone the repository:
```bash
git clone https://github.com/S3MU1L/rustpress.git
cd rustpress
```

2. Run the SSR server (Actix + Askama + HTMX):
```bash
cargo run --features ssr
```

3. Open your browser and navigate to:
   - Website: `http://127.0.0.1:8082/` (or whatever you set in `BIND_ADDR`)
   - Admin: `http://127.0.0.1:8082/admin`
   - Auth: `http://127.0.0.1:8082/register`, `http://127.0.0.1:8082/login`

Note: `/login` and `/register` are rendered from Rust UI components in `src/frontend/` (Leptos), not from HTML template files.

## 🐳 Docker

Run RustPress + Postgres locally:

```bash
docker compose up --build -d
```

Then open:
- Website: `http://localhost:8081/`
- Admin: `http://localhost:8081/admin`
- Auth: `http://localhost:8081/register`, `http://localhost:8081/login`

The compose file provisions a Postgres database with:
- user: `rustpress`
- password: `rustpress`
- db: `rustpress`

Note: Docker builds use SQLx offline metadata stored in `.sqlx/`.

If you want to run without Docker, copy `.env.example` to `.env` and adjust `DATABASE_URL`.

## 🎯 Usage

### Creating Content

1. Navigate to the admin console at `http://localhost:8080/admin`
2. Create new pages or articles
3. Choose from available templates or create custom ones
4. Publish your content

### Templates

RustPress comes with several built-in templates:
- Default blog template
- Portfolio template
- Business template
- Landing page template

You can also create custom templates by adding HTML files to the `templates/` directory.

## 🛠️ Development

### Frontend (Leptos)

The Rust UI components live in `src/frontend/`.

At the moment, the app is a hybrid:
- Admin/content pages use Askama templates in `templates/`
- Auth pages (`/login`, `/register`) are rendered from `src/frontend/` components

### Building from Source

```bash
# Development build
cargo build

# Run in development mode
cargo run --features ssr

# Run tests
cargo test --features ssr

# Format code
cargo fmt

# Run linter
cargo clippy
```

### SQLx offline metadata

This repo uses SQLx compile-time queries, so `.sqlx/` is checked in. Local builds default to offline mode via `.cargo/config.toml`.

If you change SQL queries or migrations, refresh the cache (requires a running Postgres matching `DATABASE_URL`):

```bash
docker compose up -d db
SQLX_OFFLINE=false cargo sqlx prepare -- --tests --features ssr
```

### Project Structure

```
rustpress/
├── src/
│   ├── main.rs          # Application entry point
│   ├── routes/          # HTTP route handlers
│   ├── models/          # Data models
│   ├── templates/       # Template management
│   └── admin/           # Admin console logic
├── static/              # Static files (CSS, JS, images)
├── templates/           # HTML templates
└── Cargo.toml          # Project dependencies
```

## Contact

Samuel Malec - [@is.muni](https://is.muni.cz/auth/osoba/536542)
Peter Rakšány - [@is.muni](https://is.muni.cz/auth/osoba/xraksany)
Matej Vavro - [@is.muni](https://is.muni.cz/auth/osoba/536408)
Vladimír Uhlík - [@is.muni](https://is.muni.cz/auth/osoba/514058)

Project Link: [https://github.com/S3MU1L/rustpress](https://github.com/S3MU1L/rustpress)

---