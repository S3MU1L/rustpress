#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos_actix::{LeptosRoutes, generate_route_list, handle_server_fns};
    use rustpress::frontend::{App, shell};
    use rustpress::db::Database;

    let conf = leptos::config::get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. postgres://user:pass@localhost/rustpress)");
    let db = Database::new(&database_url)
        .await
        .expect("Failed to connect to database / run migrations");
    let pool = db.pool;
    let pool_data = web::Data::new(pool);

    println!("🦀 Starting RustPress CMS...");
    println!("📍 Server running at http://{}", addr);
    println!("🔧 Admin console at http://{}/admin", addr);

    HttpServer::new(move || {
        let routes = generate_route_list(App);
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone();

        App::new()
            .app_data(pool_data.clone())
            // Leptos server functions (backend API)
            .route("/api/{tail:.*}", handle_server_fns())
            .service(Files::new("/pkg", format!("{}/pkg", site_root)))
            .service(Files::new("/static", "./static").prefer_utf8(true))
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            })
            .app_data(web::Data::new(leptos_options.clone()))
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // Client-side only - this shouldn't be called directly
    // The hydrate function in lib.rs handles client-side initialization
}
