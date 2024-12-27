use crate::components::Navbar;
use leptos::{component, view, IntoView};

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Navbar />
    }
}
