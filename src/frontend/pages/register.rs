use leptos::prelude::*;

use crate::frontend::components::{
    Button, ButtonVariant, EmailInput, ErrorAlert, PasswordInput, TextInput,
};

#[component]
pub fn RegisterPage(#[prop(optional, into)] error: String) -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());

    let error_message = match error.as_str() {
        "" => String::new(),
        "missing" => "Email required and password must be at least 4 characters".to_string(),
        "exists" => "Email already exists".to_string(),
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
                        <h1 class="text-2xl font-bold text-white">"Create Account"</h1>
                        <p class="text-slate-400 mt-2">"Get started with RustPress"</p>
                    </div>

                    <Show when=move || show_error>
                        <ErrorAlert message=error_message.clone() />
                    </Show>

                    <form method="post" action="/register" class="space-y-5">
                        <TextInput
                            label="Username (optional)"
                            name="username"
                            placeholder="rustacean"
                            input_type="text"
                            value=username
                            set_value=set_username
                        />
                        <EmailInput label="Email" value=email set_value=set_email />
                        <PasswordInput
                            label="Password"
                            hint="Must be at least 4 characters"
                            value=password
                            set_value=set_password
                        />
                        <Button
                            variant=ButtonVariant::Primary
                            loading=false
                            loading_text=""
                        >
                            "Create Account"
                        </Button>
                    </form>

                    <p class="text-center text-slate-400 mt-6 text-sm">
                        "Already have an account? "
                        <a href="/login" class="text-orange-400 hover:text-orange-300 font-medium">"Sign in"</a>
                    </p>
                </div>

                <a href="/" class="block text-center text-slate-500 hover:text-slate-300 mt-6 text-sm transition-colors">
                    "← Back to home"
                </a>
            </div>
        </div>
    }
}
