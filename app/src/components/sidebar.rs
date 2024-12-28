use leptos::prelude::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <>
            <label for="sidebar" class="drawer-overlay"></label>
            <ul class="menu bg-base-200 text-base-content min-h-full w-80 p-4">
                <Upload />
                <li><a>Sidebar Item 1</a></li>
                <li><a>Sidebar Item 2</a></li>
            </ul>
        </>
    }
}

#[component]
pub fn Upload() -> impl IntoView {
    view! {
        <input id="upload" type="file" class="hidden" />
        <label for="upload" class="btn btn-secondary mb-10">
        <div class="flex gap-x-2 items-center justify-center">
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
        </div>
        </label>
    }
}
