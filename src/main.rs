mod web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    use actix_files::Files;
    use actix_web::body::BoxBody;
    use actix_web::dev::{ServiceRequest, ServiceResponse};
    use actix_web::http::header::LOCATION;
    use actix_web::middleware::{from_fn, Next};
    use actix_web::{App, Error, HttpResponse, HttpServer};
    use rustpress::db::Database;
    use crate::web::{handlers, AppState};

    async fn admin_auth_guard(
        req: ServiceRequest,
        next: Next<BoxBody>,
    ) -> Result<ServiceResponse<BoxBody>, Error> {
        let path = req.path();

        if path.starts_with("/admin") {
            let has_cookie = req.cookie("rp_uid").is_some();
            if !has_cookie {
                let is_htmx = req
                    .headers()
                    .get("HX-Request")
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|s| s.eq_ignore_ascii_case("true"));

                if is_htmx {
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .insert_header(("HX-Redirect", "/login"))
                            .finish(),
                    ));
                }

                return Ok(req.into_response(
                    HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/login"))
                        .finish(),
                ));
            }
        }

        next.call(req).await
    }

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://...)");
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let db = Database::new(&database_url)
        .await
        .expect("Failed to initialize database");

    let state = actix_web::web::Data::new(AppState {
        pool: db.pool.clone(),
    });

    println!("Starting RustPress (Actix + Askama + HTMX)");
    println!("Server running at http://{bind_addr}");
    println!("Admin console at http://{bind_addr}/admin");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(from_fn(admin_auth_guard))
            .service(Files::new("/static", "./static"))
            .configure(handlers::configure)
    })
    .bind(bind_addr)?
    .run()
    .await
}
