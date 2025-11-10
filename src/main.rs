use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("./templates/index.html"))
}

async fn admin() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("./templates/admin.html"))
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "RustPress CMS"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🦀 Starting RustPress CMS...");
    println!("📍 Server running at http://localhost:8080");
    println!("🔧 Admin console at http://localhost:8080/admin");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/admin", web::get().to(admin))
            .route("/health", web::get().to(health))
            .service(fs::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
