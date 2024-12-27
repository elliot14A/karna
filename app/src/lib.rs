#![allow(dead_code)]

use common::theme::ThemeSwitcher;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use pages::home::HomePage;

mod common;
mod components;
mod error_template;
mod pages;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let theme = ThemeSwitcher::new();
    provide_context(theme.clone());

    view! {
        <Html attr:data-theme={move || theme.current.get().to_string()} {..} class="h-full" />
        <>
            <Stylesheet id="karna" href="/pkg/karna.css" />
            <Router>
                <main>
                    <Routes fallback=|| "Not found.">
                        <Route path=path!("/") view=HomePage />
                    </Routes>
                </main>
            </Router>
        </>
    }
}
