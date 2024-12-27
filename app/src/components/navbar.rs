use crate::common::theme::{Theme, ThemeSwitcher};
use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn Navbar() -> impl IntoView {
    let theme = expect_context::<ThemeSwitcher>();
    let current_theme = theme.current;

    view! {
        <div class="navbar bg-base-100">
            <div class="flex-1">
                <p class="text-xl font-bold">Karna</p>
            </div>
            <div class="flex-none">
                <button
                    on:click=move |_| theme.switch()
                    class="btn btn-square btn-ghost hover:bg-base-200"
                    aria-label="Toggle theme"
                >
                    {move || match current_theme.get() {
                        Theme::Dark => view! {
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                class="inline-block h-5 w-5 stroke-current"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
                                />
                            </svg>
                        },
                        Theme::Retro => view! {
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                class="inline-block h-5 w-5 stroke-current"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
                                />
                            </svg>
                        }
                    }}
                </button>
            </div>
        </div>
    }
}
