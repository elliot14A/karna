use crate::{actions::datasets::{upload_file_system, delete as delete_dataset}, common::models::Dataset};
use leptos::{prelude::*, task::spawn_local};
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use crate::pages::home::Selected;

#[component]
pub fn Sidebar(datasets_res: LocalResource<Vec<Dataset>>, trigger: WriteSignal<i32>, selected: WriteSignal<Selected>) -> impl IntoView {
    let notebook_res = LocalResource::new(|| async move {
                        vec!["Notebook 1".to_owned()]
                    });
    view! {
        <>
            <label for="sidebar" class="drawer-overlay"></label>
            <ul class="menu bg-base-200 text-base-content min-h-full w-80 p-4 rounded-box">
                <Upload trigger=trigger />
                <li>
                    <Notebooks notebooks_res=notebook_res />
                    <Datasets datasets_res=datasets_res trigger=trigger selected=selected />
                </li>
            </ul>
        </>
    }
}

#[component]
pub fn Upload(trigger: WriteSignal<i32>) -> impl IntoView {
    let (is_uploading, set_is_uploading) = signal(false);
    let on_change = move |ev: Event| {
        let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
        if let Some(filelist) = input.files() {
            if let Some(file) = filelist.get(0) {
                set_is_uploading.set(true);
                spawn_local(async move {
                    upload_file_system(file).await.unwrap();
                    trigger.update(|x| *x += 1);
                    set_is_uploading.set(false);
                });
            }
        }
    };

    view! {
        <input
            id="upload"
            type="file"
            class="hidden"
            on:change=on_change
            accept=".csv,.tsv,.parquet"
            disabled=move || is_uploading.get()
        />
        <label for="upload" class="btn btn-secondary mb-6">
            <div class="flex gap-x-2 items-center justify-center">
                {move || {
                    if is_uploading.get() {
                        view! { <p>"Uploading..."</p> }.into_any()
                    } else {
                        view! {
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            >
                                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                                <polyline points="17 8 12 3 7 8" />
                                <line x1="12" y1="3" x2="12" y2="15" />
                            </svg>
                            <p>Upload</p>
                        }
                            .into_any()
                    }
                }}
            </div>
        </label>
    }
}


// Components
#[component]
fn DatasetIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5 mr-1"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
        </svg>
    }
}

#[component]
fn DeleteIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-4 w-4 text-error"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
            />
        </svg>
    }
}

#[component]
fn DatasetItem(id: String,name: String,on_select: Callback<String>,on_delete: Callback<String>,) -> impl IntoView {
    let select_id = id.clone();
    let delete_id = id.clone();

    view! {
        <li class="relative group hover:bg-base-200 rounded-btn">

            <div class="flex items-center justify-between w-full">
                <a class="flex-1" on:click=move |_| on_select.run(select_id.clone())>
                    <div class="flex items-center">
                        <DatasetIcon />
                        <span class="font-medium truncate">{name}</span>
                    </div>
                </a>
                <button
                    class="btn btn-ghost hover:bg-transparent btn-xs opacity-0 group-hover:opacity-100 transition-opacity ml-2 cursor-pointer"
                    title="Delete dataset"
                    on:click=move |_| {
                        leptos::logging::log!("Delete dataset");
                        on_delete.run(delete_id.clone());
                    }
                >
                    <DeleteIcon />
                </button>
            </div>
        </li>
    }
}

#[component]
pub fn DatasetsList(
    datasets: SendWrapper<Vec<Dataset>>,
    on_select: Callback<String>,
    on_delete: Callback<String>,
) -> impl IntoView {
    if datasets.is_empty() {
        view! { <li>"No dataset found"</li> }.into_any()
    } else {
        datasets
            .iter()
            .map(|dataset| {
                view! {
                    <DatasetItem
                        id=dataset.id.clone()
                        name=dataset.name.clone()
                        on_select=on_select.clone()
                        on_delete=on_delete.clone()
                    />
                }
            })
            .collect_view()
            .into_any()
    }
}

#[component]
pub fn Datasets(
    datasets_res: LocalResource<Vec<Dataset>>,
    trigger: WriteSignal<i32>,
    selected: WriteSignal<Selected>
) -> impl IntoView {
    let delete_action = Action::new(move |dataset_id: &String| {
        let dataset_id = dataset_id.to_owned();
        async move {
            delete_dataset(dataset_id).await.unwrap();
            trigger.update(|x| *x += 1);
        }
    });

    let on_select = Callback::new(move |id: String| {
        selected.set(Selected::Dataset(id));
    });

    let on_delete = Callback::new(move |id: String| {
        delete_action.dispatch_local(id);
    });

    view! {
        <h3 class="menu-title text-lg">"Datasets"</h3>
        <ul class="min-h-16 max-h-96 overflow-y-auto">
            <Suspense fallback=move || {
                view! { <span class="loading loading-dots loading-md"></span> }
            }>
                {move || {
                    datasets_res
                        .get()
                        .map(|datasets| {
                            view! {
                                <DatasetsList
                                    datasets=datasets
                                    on_select=on_select.clone()
                                    on_delete=on_delete.clone()
                                />
                            }
                                .into_any()
                        })
                        .unwrap_or_else(|| view! { <li>"No dataset found"</li> }.into_any())
                }}
            </Suspense>
        </ul>
    }
}


#[component]
pub fn Notebooks(
    notebooks_res: LocalResource<Vec<String>>,
) -> impl IntoView {
    view! {
        <h3 class="menu-title text-lg">Notebooks</h3>
        <ul class="min-h-16 max-h-96 overflow-y-auto">
            <Suspense fallback=move || {
                view! { <span class="loading loading-dots loading-md"></span> }
            }>
                {move || {
                    match notebooks_res.get() {
                        Some(notebook) => {
                            if notebook.is_empty() {
                                return view! { <li>"No notebook found"</li> }.into_any();
                            }
                            notebook
                                .iter()
                                .map(|notebook| {
                                    view! {
                                        <li class="relative group hover:bg-base-200 rounded-btn">
                                            <div class="flex items-center justify-between w-full">
                                                <a class="flex-1">
                                                    <div class="flex items-center">
                                                        <svg
                                                            xmlns="http://www.w3.org/2000/svg"
                                                            class="h-5 w-5 mr-1"
                                                            fill="none"
                                                            viewBox="0 0 24 24"
                                                            stroke="currentColor"
                                                        >
                                                            <path
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                                stroke-width="2"
                                                                d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
                                                            />
                                                        </svg>
                                                        <span class="font-medium truncate">
                                                            {notebook.to_string()}
                                                        </span>
                                                    </div>
                                                </a>
                                                <button
                                                    class="btn btn-ghost hover:bg-transparent btn-xs opacity-0 group-hover:opacity-100 transition-opacity ml-2"
                                                    title="Delete notebook"
                                                >
                                                    <svg
                                                        xmlns="http://www.w3.org/2000/svg"
                                                        class="h-4 w-4 text-error"
                                                        fill="none"
                                                        viewBox="0 0 24 24"
                                                        stroke="currentColor"
                                                    >
                                                        <path
                                                            stroke-linecap="round"
                                                            stroke-linejoin="round"
                                                            stroke-width="2"
                                                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                                        />
                                                    </svg>
                                                </button>
                                            </div>
                                        </li>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                        None => view! { <li>"No notebook found"</li> }.into_any(),
                    }
                }}
            </Suspense>
        </ul>
    }
}
