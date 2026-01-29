mod web;

use actix_files::Files;
use actix_web::web::Data;
use actix_web::{App, HttpServer};

use rustpress::db::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://user:pass@localhost/rustpress)");
    let db = Database::new(&database_url)
        .await
        .expect("Failed to connect to database / run migrations");

    let state = Data::new(web::routes::AppState { pool: db.pool });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(web::routes::configure)
            .service(Files::new("/static", "./static").prefer_utf8(true))
    })
    .bind(
        std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
    )?
    .run()
    .await
}
