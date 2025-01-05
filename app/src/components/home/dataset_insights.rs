use leptos::prelude::*;
use leptos::{component, IntoView};

#[component]
pub fn Insights(dataset_id: String) -> impl IntoView {
    view! {
        <div>
            <h1>{"Dataset Insights"}</h1>
            <p>{"Dataset ID: "} {dataset_id}</p>
        </div>
    }
}
