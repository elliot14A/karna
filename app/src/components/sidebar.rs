use crate::actions::datasets::{list, upload_file_system};
use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, Event};

#[component]
pub fn Sidebar() -> impl IntoView {

    let (trigger, set_trigger) = signal(0);
    let datasets_res = LocalResource::new(move || async move { trigger.track() ;list().await.unwrap() });

    view! {
        <>
            <label for="sidebar" class="drawer-overlay"></label>
            <ul class="menu bg-base-200 text-base-content min-h-full w-80 p-4 rounded-box">
                <Upload trigger=set_trigger />
                <li>
                    <h3 class="menu-title text-lg">Datasets</h3>
                    <ul>
                        <Suspense fallback=move || {
                            view! { <span class="loading loading-dots loading-md"></span> }
                        }>
                            {move || match datasets_res.get() {
                                Some(datasets) => {
                                    datasets
                                        .iter()
                                        .map(|dataset| {

                                            view! {
                                                <li class="relative">
                                                    <a>
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
                                                        <span class="font-medium truncate">
                                                            {dataset.name.clone()}
                                                        </span>
                                                    </a>
                                                </li>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                                None => view! { <li>"No dataset found"</li> }.into_any(),
                            }}
                        </Suspense>
                    </ul>

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
                });
                set_is_uploading.set(false);
            }
        }
    };

    view! {
        <input id="upload" type="file" class="hidden" on:change=on_change />
        <label for="upload" class="btn btn-secondary mb-10">
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
