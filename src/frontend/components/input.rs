use leptos::prelude::*;

#[component]
pub fn TextInput(
    #[prop(into)] label: String,
    #[prop(into)] name: String,
    #[prop(into)] placeholder: String,
    #[prop(into)] input_type: String,
    #[prop(optional)] required: bool,
    #[prop(optional, into)] hint: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
) -> impl IntoView {
    let has_hint = !hint.is_empty();

    view! {
        <div>
            <label for=name.clone() class="block text-sm font-medium text-slate-300 mb-2">
                {label}
            </label>
            <input
                type=input_type
                id=name.clone()
                name=name
                placeholder=placeholder
                required=required
                prop:value=move || value.get()
                on:input=move |ev| set_value.set(event_target_value(&ev))
                class="w-full px-4 py-3 rounded-lg bg-slate-800 border border-slate-700
                       text-white placeholder-slate-500
                       focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-transparent
                       transition-all"
            />
            {has_hint.then(|| view! { <p class="mt-1 text-xs text-slate-500">{hint.clone()}</p> })}
        </div>
    }
}

#[component]
pub fn EmailInput(
    #[prop(into)] label: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <TextInput
            label=label
            name="email"
            placeholder="you@example.com"
            input_type="email"
            required=true
            value=value
            set_value=set_value
        />
    }
}

#[component]
pub fn PasswordInput(
    #[prop(into)] label: String,
    #[prop(optional, into)] hint: String,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <TextInput
            label=label
            name="password"
            placeholder="••••••••"
            input_type="password"
            required=true
            hint=hint
            value=value
            set_value=set_value
        />
    }
}
