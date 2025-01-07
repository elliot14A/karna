use leptos::prelude::*;
use leptos::{component, IntoView};
use crate::actions::datasets::details;

// Component for the loading skeleton
#[component]
fn LoadingSkeleton() -> impl IntoView {
    view! {
        <div class="stats shadow animate-pulse">
            <div class="stat">
                <div class="stat-title skeleton h-4 w-20"></div>
                <div class="stat-value skeleton h-8 w-32 mt-2"></div>
            </div>
            <div class="stat">
                <div class="stat-title skeleton h-4 w-20"></div>
                <div class="stat-value skeleton h-8 w-32 mt-2"></div>
            </div>
            <div class="stat">
                <div class="stat-title skeleton h-4 w-20"></div>
                <div class="stat-value skeleton h-8 w-32 mt-2"></div>
            </div>
        </div>
    }
}

#[component]
pub fn Insights(
    #[prop(into)] dataset_id: String,
) -> impl IntoView {
    let id = dataset_id.clone();
    let dataset = LocalResource::new(move || {
        let id = id.clone(); 
        async move {
            details(&id).await.unwrap()     
        }
    });

    view! {
        <Transition fallback=move || view! { <LoadingSkeleton /> }>
            <div>
                {move || match dataset.get() {
                    Some(data) => {
                        let size = (data.size as f64 / 1024.0) / 1024.0;
                        let size = (size * 100.0).round() / 100.0;
                        view! {
                            <div class="flex justify-between px-2 py-4 w-full">
                                <DatasetStats
                                    name=data.name.clone()
                                    row_count=data.row_count
                                    size=size
                                    r#type=data.r#type.clone()
                                />
                                <Download dataset_id=data.id.clone() />
                            </div>
                        }
                            .into_any()
                    }
                    None => view! { <div>"No data available"</div> }.into_any(),
                }}
            </div>
        </Transition>
    }
}

#[component]
fn DatasetStats(name: String, row_count: u64, size:f64, r#type: String) -> impl IntoView {
    view! {
        <div class="stats shadow">
            <div class="stat">
                <div class="stat-title">"Name"</div>
                <div class="stat-value text-2xl">{name}</div>
            </div>
            <div class="stat">
                <div class="stat-title">"Rows"</div>
                <div class="stat-value text-2xl">{row_count}</div>
            </div>
            <div class="stat">
                <div class="stat-title">"Size"</div>
                <div class="stat-value text-2xl">{size}" MB"</div>
            </div>
            <div class="badge badge-primary badge-lg">{r#type.clone()}</div>
        </div>
    }
}

#[component]
fn Download(dataset_id: String) -> impl IntoView {
    view! {
        <button class="btn btn-secondary gap-2">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="w-6 h-6"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3"
                />
            </svg>
            Download
        </button>
    }
}

#[component]
fn DatasetPreview() -> impl IntoView {
    let (is_table_view, set_is_table_view) = signal(true);
    view! {
        <div class="container mx-auto p-4">
            <div class="form-control mb-4">
                <label for="" class="label cursor-pointer justify-start gap4">
                    <input
                        type="checkbox"
                        class="toggle toggle-primary"
                        prop:checked=move || is_table_view.get()
                        on:change=move |_| set_is_table_view.set(!is_table_view.get())
                    />
                    <span class="label-text">
                        {move || if is_table_view.get() { "Table View" } else { "JSON View" }}
                    </span>
                </label>
            </div>
        </div>
    }
}
