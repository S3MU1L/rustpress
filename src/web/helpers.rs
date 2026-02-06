use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use askama::Template;
use sqlx::PgPool;
use uuid::Uuid;

use crate::web::templates::{NotFoundTemplate, UnauthorizedTemplate};
use rustpress::models::User;

/// Marker stored in request extensions by the admin middleware.
#[derive(Clone, Copy)]
pub struct AdminStatus(pub bool);

/// Read the admin flag set by middleware. Returns `false` if middleware didn't run.
pub fn get_is_admin(req: &HttpRequest) -> bool {
    req.extensions()
        .get::<AdminStatus>()
        .map(|s| s.0)
        .unwrap_or(false)
}

pub fn is_htmx(req: &HttpRequest) -> bool {
    req.headers()
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| s.eq_ignore_ascii_case("true"))
}

pub fn current_user_id(req: &HttpRequest) -> Option<Uuid> {
    // MVP auth/session.
    // Priority: cookie -> request header -> env var.
    let cookie_val = req
        .cookie("rp_uid")
        .map(|c| c.value().trim().to_string())
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(&s).ok());

    if cookie_val.is_some() {
        return cookie_val;
    }

    let header_val = req
        .headers()
        .get("X-Rustpress-User-Id")
        .or_else(|| req.headers().get("X-User-Id"))
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .and_then(|s| Uuid::parse_str(s).ok());

    header_val.or_else(|| {
        std::env::var("RUSTPRESS_USER_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .and_then(|s| Uuid::parse_str(&s).ok())
    })
}

pub fn require_user(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    match current_user_id(req) {
        Some(uid) => Ok(uid),
        None => {
            if is_htmx(req) {
                Err(HttpResponse::Unauthorized()
                    .insert_header(("HX-Redirect", "/admin/login"))
                    .finish())
            } else {
                Err(HttpResponse::SeeOther()
                    .insert_header(("Location", "/admin/login"))
                    .finish())
            }
        }
    }
}

pub async fn load_user(
    pool: &PgPool,
    uid: Uuid,
) -> Result<User, HttpResponse> {
    let user = sqlx::query_as::<_, User>(
        r#"SELECT * FROM users WHERE id = $1"#,
    )
    .bind(uid)
    .fetch_optional(pool)
    .await;

    match user {
        Ok(Some(u)) => Ok(u),
        Ok(None) => {
            Err(HttpResponse::Unauthorized().body("User not found"))
        }
        Err(e) => Err(HttpResponse::InternalServerError()
            .body(format!("Database error: {e}"))),
    }
}

pub fn render<T: Template>(t: T) -> HttpResponse {
    match t.render() {
        Ok(body) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(body),
        Err(e) => HttpResponse::InternalServerError()
            .content_type("text/plain; charset=utf-8")
            .body(format!("Template error: {e}")),
    }
}

pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Remove `<script ...> ... </script>` blocks from an HTML string.
///
/// This is used for sandboxed iframe previews (`srcdoc` + `sandbox` without `allow-scripts`).
/// Keeping scripts in the HTML would cause browsers to spam the console with
/// "Blocked script execution in 'about:srcdoc' ..." messages.
pub fn strip_script_tags(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len());

    let mut i = 0usize;
    while let Some(rel_start) = lower[i..].find("<script") {
        let start = i + rel_start;
        out.push_str(&html[i..start]);

        // Find the matching closing </script> (case-insensitive because `lower`).
        let after_open = start + "<script".len();
        if let Some(rel_end_tag) =
            lower[after_open..].find("</script")
        {
            let end_tag_start = after_open + rel_end_tag;
            // Skip through the closing '>' of </script ...>
            if let Some(rel_gt) = lower[end_tag_start..].find('>') {
                i = end_tag_start + rel_gt + 1;
                continue;
            } else {
                // Malformed HTML; drop the remainder.
                return out;
            }
        } else {
            // No closing tag found; drop the remainder.
            return out;
        }
    }

    out.push_str(&html[i..]);
    out
}

pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => {
            db_err.code().as_deref() == Some("23505")
        }
        _ => false,
    }
}

pub fn iframe_srcdoc(html: &str) -> String {
    // `srcdoc` is an attribute; escape enough to keep it valid.
    // Browsers will decode entities inside attributes.
    //
    // Also strip `<script>` blocks: this iframe is sandboxed without `allow-scripts`,
    // so leaving scripts in would only create noisy console errors.
    let html = strip_script_tags(html);
    format!(
        // Use `allow-scripts` to avoid browser console spam from scripts being blocked in `about:srcdoc`.
        // We intentionally do NOT set `allow-same-origin`: the iframe remains an opaque origin, so any
        // scripts that do run (e.g., extension content scripts) cannot access RustPress cookies/storage.
        r#"<iframe class="preview-iframe" sandbox="allow-scripts" referrerpolicy="no-referrer" srcdoc="{}"></iframe>"#,
        escape_html(&html)
    )
}

pub fn normalize_builtin_template_html(
    html: &str,
) -> std::borrow::Cow<'_, str> {
    if !(html.contains("\\n") || html.contains("\\t")) {
        return std::borrow::Cow::Borrowed(html);
    }

    // Older built-in templates were inserted via migrations using literal "\\n" sequences.
    // Only normalize built-ins (call sites guard on `is_builtin`) to avoid surprising changes
    // in user-authored templates.
    let html = html.replace("\\n", "\n").replace("\\t", "\t");
    std::borrow::Cow::Owned(html)
}

pub fn apply_site_template(
    template_html: &str,
    title: &str,
    content_html: &str,
    slug: &str,
    kind: &str,
) -> String {
    // Very small placeholder system.
    // - title/slug/kind are escaped
    // - content is inserted as-is (admin-authored)
    let title_e = escape_html(title);
    let slug_e = escape_html(slug);
    let kind_e = escape_html(kind);

    let replacements: [(&str, &str); 8] = [
        ("{{title}}", title_e.as_str()),
        ("{{ title }}", title_e.as_str()),
        ("{{slug}}", slug_e.as_str()),
        ("{{ slug }}", slug_e.as_str()),
        ("{{kind}}", kind_e.as_str()),
        ("{{ kind }}", kind_e.as_str()),
        ("{{content}}", content_html),
        ("{{ content }}", content_html),
    ];

    let mut out = template_html.to_string();
    for (needle, replacement) in replacements {
        out = out.replace(needle, replacement);
    }
    out
}

pub fn render_not_found(req: &HttpRequest) -> HttpResponse {
    let is_admin = get_is_admin(req);
    let template = NotFoundTemplate { is_admin };

    match template.render() {
        Ok(body) => HttpResponse::NotFound()
            .content_type("text/html; charset=utf-8")
            .body(body),
        Err(e) => HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body(format!("404 Not Found (template error: {e})")),
    }
}

pub fn render_unauthorized(req: &HttpRequest) -> HttpResponse {
    let is_admin = get_is_admin(req);
    let template = UnauthorizedTemplate { is_admin };

    match template.render() {
        Ok(body) => HttpResponse::Unauthorized()
            .content_type("text/html; charset=utf-8")
            .body(body),
        Err(e) => HttpResponse::Unauthorized()
            .content_type("text/plain; charset=utf-8")
            .body(format!("401 Unauthorized (template error: {e})")),
    }
}
