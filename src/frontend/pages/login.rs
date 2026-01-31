use leptos::prelude::*;

use crate::frontend::components::{
    Button, ButtonVariant, EmailInput, ErrorAlert, PasswordInput,
};

#[component]
pub fn LoginPage(#[prop(optional, into)] error: String) -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());

    let error_message = match error.as_str() {
        "" => String::new(),
        "missing" => "Missing email or password".to_string(),
        "invalid" => "Invalid credentials".to_string(),
        "db" => "Database error".to_string(),
        "internal" => "Internal server error".to_string(),
        other => other.to_string(),
    };

    let show_error = !error_message.is_empty();

    view! {
        <div class="min-h-screen flex items-center justify-center px-6 py-12 bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
            <div class="w-full max-w-md">
                <div class="bg-slate-900/80 backdrop-blur-sm border border-slate-800 rounded-2xl p-8 shadow-xl">
                    <div class="text-center mb-8">
                        <a href="/" class="inline-block text-4xl mb-4 hover:animate-bounce">"🦀"</a>
                        <h1 class="text-2xl font-bold text-white">"Login"</h1>
                        <p class="text-slate-400 mt-2">"Welcome back to RustPress"</p>
                    </div>

                    <Show when=move || show_error>
                        <ErrorAlert message=error_message.clone() />
                    </Show>

                    <form method="post" action="/login" class="space-y-6">
                        <EmailInput label="Email" value=email set_value=set_email />
                        <PasswordInput label="Password" value=password set_value=set_password />
                        <Button
                            variant=ButtonVariant::Primary
                            loading=false
                            loading_text=""
                        >
                            "Sign In"
                        </Button>
                    </form>

                    <p class="text-center text-slate-400 mt-6 text-sm">
                        "Don't have an account? "
                        <a href="/register" class="text-orange-400 hover:text-orange-300 font-medium">"Sign up"</a>
                    </p>
                </div>

                <a href="/" class="block text-center text-slate-500 hover:text-slate-300 mt-6 text-sm transition-colors">
                    "← Back to home"
                </a>
            </div>
        </div>
    }
}
