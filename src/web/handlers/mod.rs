pub mod account;
pub mod admin_collaborators;
pub mod admin_content;
pub mod admin_history;
pub mod admin_roles;
pub mod admin_templates;
pub mod admin_users;
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
    admin_history::configure(cfg);
    admin_collaborators::configure(cfg);
    admin_roles::configure(cfg);
    admin_templates::configure(cfg);
    admin_users::configure(cfg);
    account::configure(cfg);
    configuration::configure(cfg);
    sites::configure(cfg);
    themes::configure(cfg);
}

/// Configure the catch-all page route. This MUST be called last.
pub fn configure_catch_all(cfg: &mut web::ServiceConfig) {
    cfg.service(public::page_page);
}
