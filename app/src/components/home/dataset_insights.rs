use std::collections::HashMap;

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use serde_json::Value;
use crate::actions::{datasets::details, queries::query_dataset_with_pagination};

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
                            <div class="flex flex-col w-full">
                                <div class="flex justify-between ">
                                    <DatasetStats
                                        name=data.name.clone()
                                        row_count=data.row_count
                                        size=size
                                        r#type=data.r#type.clone()
                                    />
                                    <Download dataset_id=data.id.clone() />
                                </div>
                                <div>
                                    <DatasetPreview dataset=data.name.clone() />
                                </div>
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
        <div class="stats border-2 border-base-200 rounded-lg">
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
    let _ = dataset_id;
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
fn DatasetPreview(dataset: String) -> impl IntoView {
    let dataset_id = dataset.clone();

    leptos::logging::log!("dataset: {}", dataset);
    let (is_table_view, set_is_table_view) = signal(true);

    let data = LocalResource::new(move || {
        let id = dataset_id.clone();
        async move {
            query_dataset_with_pagination(id.as_ref(), 1, 20).await.unwrap()
        }
    });

    view! {
        <ToggleViewButtons is_table_view=is_table_view set_is_table_view=set_is_table_view />
        <Transition fallback=move || {
            view! { <div>"Loading Bro ..."</div> }
        }>
            {move || {
                match data.get().and_then(process_data) {
                    None => view! { <div>"No data available"</div> }.into_any(),
                    Some((columns, row_values)) => {
                        view! { <TableView columns=columns row_values=row_values /> }.into_any()
                    }
                }
            }}
        </Transition>
    }
}

#[component]
fn TableView(columns: Vec<String>, row_values: Vec<Vec<String>>) -> impl IntoView {
    view! {
        <div class="overflow-auto my-4">
            <table class="table table-xs">
                <thead>
                    <tr>
                        {move || {
                            columns
                                .iter()
                                .map(|col| view! { <th>{col.to_owned()}</th> })
                                .collect_view()
                        }}
                    </tr>
                </thead>
                <tbody>
                    {row_values
                        .iter()
                        .map(|row_data| {
                            view! {
                                <tr>
                                    {row_data
                                        .iter()
                                        .map(|value| view! { <td>{value.clone()}</td> })
                                        .collect_view()}
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn ToggleViewButtons(
    is_table_view: ReadSignal<bool>, 
    set_is_table_view: WriteSignal<bool>
) -> impl IntoView {
    let preview_class = move || {
        if is_table_view.get() {
            "btn btn-secondary"
        } else {
            "btn btn-secondary btn-outline"
        }
    };

    let schema_class = move || {
        if is_table_view.get() {
            "btn btn-primary btn-outline"
        } else {
            "btn btn-primary"
        }
    };

    view! {
        <div class="flex container mx-auto mt-4 gap-2">
            <button class=preview_class on:click=move |_| set_is_table_view.set(true)>
                "Preview"
            </button>
            <button class=schema_class on:click=move |_| set_is_table_view.set(false)>
                "Schema"
            </button>
        </div>
    }
}

fn process_data(data: SendWrapper<Vec<HashMap<String, Value>>>) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    if data.is_empty() {
        return None;
    }

    // Get columns from the first row's keys
    let columns: Vec<String> = data.first()?
        .keys()
        .cloned()
        .collect();

    // Process each row's values in column order
    let row_values: Vec<Vec<String>> = data
        .iter()
        .map(|row| {
            columns.iter()
                .map(|col| {
                    row.get(col)
                        .map(|v| match v {
                            serde_json::Value::Null => "N/A".to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Array(a) => format!("{:?}", a),
                            serde_json::Value::Object(o) => format!("{:?}", o),
                        })
                        .unwrap_or_else(|| "N/A".to_string())
                })
                .collect()
        })
        .collect();

    Some((columns, row_values))
}

