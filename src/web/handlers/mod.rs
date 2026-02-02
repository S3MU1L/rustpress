pub mod account;
pub mod admin_content;
pub mod admin_templates;
pub mod auth;
pub mod public;
pub mod sites;
pub mod themes;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    public::configure(cfg);
    auth::configure(cfg);
    account::configure(cfg);
    sites::configure(cfg);
    themes::configure(cfg);
    admin_content::configure(cfg);
    admin_templates::configure(cfg);
}
