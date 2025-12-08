use leptos::prelude::*;

#[derive(Clone, Copy, Default, PartialEq)]
pub enum AlertVariant {
    #[default]
    Success,
    Error,
}

#[component]
pub fn Alert(
    #[prop(into)] message: String,
    #[prop(optional)] variant: AlertVariant,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let (icon, classes) = match variant {
        AlertVariant::Success => (
            "✓",
            "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
        ),
        AlertVariant::Error => ("✕", "bg-red-500/10 border-red-500/30 text-red-400"),
    };

    view! {
        <div class=format!("mb-6 p-4 rounded-lg border text-sm {}", classes)>
            <p class="flex items-center gap-2">
                <span>{icon}</span>
                <span>{message}</span>
            </p>
            {children.map(|c| view! { <div class="mt-3">{c()}</div> })}
        </div>
    }
}

#[component]
pub fn SuccessAlert(
    #[prop(into)] message: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <Alert message=message variant=AlertVariant::Success>
            {children.map(|c| c())}
        </Alert>
    }
}

#[component]
pub fn ErrorAlert(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <Alert message=message variant=AlertVariant::Error />
    }
}
