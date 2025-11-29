use leptos::prelude::*;

use crate::api::Login;
use crate::frontend::components::{
    Button, ButtonVariant, EmailInput, ErrorAlert, PasswordInput, SuccessAlert,
};

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<Login>::new();

    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());

    let pending = login_action.pending();
    let result = login_action.value();

    let success_message = move || {
        result.get().and_then(|r| {
            r.ok().map(|response| {
                format!(
                    "Welcome back, {}!",
                    response.user.username.unwrap_or(response.user.email)
                )
            })
        })
    };

    let error_message = move || result.get().and_then(|r| r.err().map(|e| e.to_string()));

    view! {
        <div class="min-h-screen flex items-center justify-center px-6 py-12 bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
            <div class="w-full max-w-md">
                <div class="bg-slate-900/80 backdrop-blur-sm border border-slate-800 rounded-2xl p-8 shadow-xl">
                    <div class="text-center mb-8">
                        <a href="/" class="inline-block text-4xl mb-4 hover:animate-bounce">"🦀"</a>
                        <h1 class="text-2xl font-bold text-white">"Login"</h1>
                        <p class="text-slate-400 mt-2">"Welcome back to RustPress"</p>
                    </div>

                    <Show when=move || success_message().is_some()>
                        <SuccessAlert message=success_message().unwrap_or_default() />
                    </Show>

                    <Show when=move || error_message().is_some()>
                        <ErrorAlert message=error_message().unwrap_or_default() />
                    </Show>

                    <ActionForm action=login_action attr:class="space-y-6">
                        <EmailInput label="Email" value=email set_value=set_email />
                        <PasswordInput label="Password" value=password set_value=set_password />
                        <Button
                            variant=ButtonVariant::Primary
                            loading=pending.get()
                            loading_text="Signing in..."
                        >
                            "Sign In"
                        </Button>
                    </ActionForm>

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
