pub mod api;
pub mod frontend;
pub mod types;

/// WASM hydration entry point
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(frontend::App);
}
