use leptos::prelude::*;

use crate::api::Register;
use crate::frontend::components::{
    Button, ButtonVariant, EmailInput, ErrorAlert, PasswordInput, SuccessAlert, TextInput,
};

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_action = ServerAction::<Register>::new();

    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());

    let pending = register_action.pending();
    let result = register_action.value();

    let success_message = move || {
        result
            .get()
            .and_then(|r| r.ok().map(|response| response.message))
    };

    let error_message = move || result.get().and_then(|r| r.err().map(|e| e.to_string()));

    view! {
        <div class="min-h-screen flex items-center justify-center px-6 py-12 bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
            <div class="w-full max-w-md">
                <div class="bg-slate-900/80 backdrop-blur-sm border border-slate-800 rounded-2xl p-8 shadow-xl">
                    <div class="text-center mb-8">
                        <a href="/" class="inline-block text-4xl mb-4 hover:animate-bounce">"🦀"</a>
                        <h1 class="text-2xl font-bold text-white">"Create Account"</h1>
                        <p class="text-slate-400 mt-2">"Get started with RustPress"</p>
                    </div>

                    <Show when=move || success_message().is_some()>
                        <SuccessAlert message=success_message().unwrap_or_default()>
                            <a href="/login" class="text-emerald-300 hover:text-emerald-200 font-medium">
                                "Continue to login →"
                            </a>
                        </SuccessAlert>
                    </Show>

                    <Show when=move || error_message().is_some()>
                        <ErrorAlert message=error_message().unwrap_or_default() />
                    </Show>

                    <ActionForm action=register_action attr:class="space-y-5">
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
                            loading=pending.get()
                            loading_text="Creating account..."
                        >
                            "Create Account"
                        </Button>
                    </ActionForm>

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
