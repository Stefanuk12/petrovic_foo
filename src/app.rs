use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body class="bg-neutral-950 text-neutral-100 antialiased">
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/petrovic_foo.css"/>
        <Title text="Stefan Petrovic"/>

        <Router>
            <main class="mx-auto max-w-2xl px-6 py-24">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1 class="text-4xl font-bold tracking-tight">"Stefan Petrovic"</h1>
        // rel=external so the router does a full page load; /resume is served by axum, not leptos
        <a href="/resume" rel="external" class="mt-4 inline-block text-neutral-400 underline hover:text-neutral-100">
            "Resume"
        </a>
    }
}
