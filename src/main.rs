
mod web;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    use actix_files::Files;
    use actix_web::body::BoxBody;
    use actix_web::dev::{ServiceRequest, ServiceResponse};
    use actix_web::http::header::LOCATION;
    use actix_web::middleware::{from_fn, Next};
    use actix_web::{App, Error, HttpMessage, HttpResponse, HttpServer};
    use rustpress::db::Database;
    use crate::web::{handlers, AppState};
    use crate::web::helpers::AdminStatus;
    use rustpress::db::user_is_admin;

    /// Routes that require the admin role (non-admins get 403).
    const ADMIN_ONLY_PREFIXES: &[&str] = &["/admin/configuration", "/admin/users"];

    async fn admin_auth_guard(
        req: ServiceRequest,
        next: Next<BoxBody>,
    ) -> Result<ServiceResponse<BoxBody>, Error> {
        let path = req.path().to_owned();

        if path.starts_with("/admin")
            && !path.starts_with("/admin/login")
            && !path.starts_with("/admin/register")
        {
            // --- authentication check ---
            let uid = req
                .cookie("rp_uid")
                .and_then(|c| uuid::Uuid::parse_str(c.value().trim()).ok());

            if uid.is_none() {
                let is_htmx = req
                    .headers()
                    .get("HX-Request")
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|s| s.eq_ignore_ascii_case("true"));

                if is_htmx {
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .insert_header(("HX-Redirect", "/admin/login"))
                            .finish(),
                    ));
                }

                return Ok(req.into_response(
                    HttpResponse::SeeOther()
                        .insert_header((LOCATION, "/admin/login"))
                        .finish(),
                ));
            }

            // --- role check (compute once per request) ---
            let uid = uid.unwrap();
            let pool = req
                .app_data::<actix_web::web::Data<AppState>>()
                .map(|s| s.pool.clone());

            let is_admin = if let Some(pool) = pool {
                user_is_admin(&pool, uid).await.unwrap_or(false)
            } else {
                false
            };

            // Store for handlers to read via get_is_admin()
            req.extensions_mut().insert(AdminStatus(is_admin));

            // Block non-admins from admin-only routes
            if !is_admin && ADMIN_ONLY_PREFIXES.iter().any(|p| path.starts_with(p)) {
                return Ok(req.into_response(
                    HttpResponse::SeeOther()
                        .insert_header(("Location", "/admin"))
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
            // Catch-all page route MUST be registered last
            .configure(handlers::configure_catch_all)
    })
    .bind(bind_addr)?
    .run()
    .await
}
