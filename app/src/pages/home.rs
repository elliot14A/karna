use crate::components::{Navbar, Sidebar};
use leptos::prelude::*;
use leptos::{component, view, IntoView};

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <>
            <Navbar />
            <div class="flex flex-1">
                <div class="drawer lg:drawer-open">
                    <input id="sidebar" type="checkbox" class="drawer-toggle" />
                    <div class="drawer-content flex flex-col items-center justify-center">
                        <label for="sidebar" >
                            <p class="text-lg">Welcome to karna</p>
                        </label>
                    </div>
                    <div class="drawer-side">
                        <Sidebar />
                    </div>
                </div>
            </div>
        </>
    }
}
