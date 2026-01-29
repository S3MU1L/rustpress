use leptos::prelude::*;

#[derive(Clone, Copy, Default, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Ghost,
}

#[component]
pub fn Button(
    children: Children,
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] loading: bool,
    #[prop(optional, into)] loading_text: String,
    #[prop(optional, into)] button_type: String,
    #[prop(optional, into)] href: String,
) -> impl IntoView {
    let base_classes = "inline-flex items-center justify-center px-8 py-4 text-lg font-semibold rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-950";

    let variant_classes = match variant {
        ButtonVariant::Primary => "bg-gradient-to-r from-orange-500 to-amber-500 text-white hover:from-orange-600 hover:to-amber-600 hover:shadow-lg hover:shadow-orange-500/25 focus:ring-orange-500",
        ButtonVariant::Ghost => "border border-slate-700 text-slate-300 hover:border-slate-500 hover:text-white hover:bg-slate-800/50 focus:ring-slate-500",
    };

    let is_disabled = disabled || loading;

    let classes = format!(
        "{} {} disabled:opacity-50 disabled:cursor-not-allowed",
        base_classes, variant_classes
    );

    let loading_text_display = if loading_text.is_empty() {
        "Loading...".to_string()
    } else {
        loading_text
    };

    let button_type_val = if button_type.is_empty() {
        "submit".to_string()
    } else {
        button_type
    };

    if !href.is_empty() {
        view! {
            <a href=href class=classes.clone()>
                {children()}
            </a>
        }
        .into_any()
    } else if loading {
        view! {
            <button
                type=button_type_val
                class=classes
                disabled=is_disabled
            >
                <span class="flex items-center justify-center gap-2">
                    <span class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                    {loading_text_display}
                </span>
            </button>
        }.into_any()
    } else {
        view! {
            <button
                type=button_type_val
                class=classes
                disabled=is_disabled
            >
                {children()}
            </button>
        }
        .into_any()
    }
}
