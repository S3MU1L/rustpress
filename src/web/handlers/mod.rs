pub mod account;
pub mod admin_content;
pub mod admin_templates;
pub mod auth;
pub mod configuration;
pub mod public;
pub mod sites;
pub mod themes;

use actix_web::web;

/// Configure all routes EXCEPT the catch-all page route.
/// The catch-all must be registered last to avoid matching before specific routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    public::configure(cfg);
    auth::configure(cfg);
    admin_content::configure(cfg);
    admin_templates::configure(cfg);
    account::configure(cfg);
    configuration::configure(cfg);
    sites::configure(cfg);
    themes::configure(cfg);
}

/// Configure the catch-all page route. This MUST be called last.
pub fn configure_catch_all(cfg: &mut web::ServiceConfig) {
    cfg.service(public::page_page);
}
