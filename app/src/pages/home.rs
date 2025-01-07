use crate::actions::datasets::list;
use crate::components::{Insights, Navbar, Sidebar};
use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[derive(Debug, Clone)]
pub enum Selected {
    Dataset(String),
    Notebook(String),
}

impl Default for Selected {
    fn default() -> Self {
        Self::Notebook("".to_owned())
    }
}

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    let (trigger, set_trigger) = signal(0);
    let datasets_res = LocalResource::new(move || async move {
        trigger.track();
        list().await.unwrap()
    });

    let (selected, set_selected) = signal(Selected::default());

    view! {
        <div class="h-screen flex flex-col overflow-hidden">
            <Navbar />
            <div class="flex flex-1">
                <div class="drawer lg:drawer-open">
                    <input id="sidebar" type="checkbox" class="drawer-toggle" />
                    <div class="drawer-content px-12 py-3">
                        <label for="sidebar">
                            {move || {
                                match selected.get() {
                                    Selected::Dataset(dataset_id) => {
                                        view! { <Insights dataset_id=dataset_id /> }.into_any()
                                    }
                                    Selected::Notebook(_) => {
                                        view! { <h1>{"Notebook"}</h1> }.into_any()
                                    }
                                }
                            }}
                        </label>
                    </div>
                    <div class="drawer-side">
                        <Sidebar
                            datasets_res=datasets_res
                            trigger=set_trigger
                            selected=set_selected
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}
