use leptos::prelude::*;

#[component]
pub fn FeatureCard(icon: &'static str, title: &'static str, desc: &'static str) -> impl IntoView {
    view! {
        <div class="group p-6 rounded-xl bg-slate-900/50 border border-slate-800
                    hover:border-orange-500/50 hover:bg-slate-800/50
                    transition-all duration-300 hover:-translate-y-1">
            <span class="text-4xl mb-4 block group-hover:scale-110 transition-transform duration-300">
                {icon}
            </span>
            <h3 class="text-xl font-semibold text-white mb-2">{title}</h3>
            <p class="text-slate-400 text-sm leading-relaxed">{desc}</p>
        </div>
    }
}
