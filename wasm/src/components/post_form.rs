use client::{blog_client::BlogClient, http_client::HttpClient};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PostFormProps {
    pub on_success: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
    #[props(default = None)]
    pub post_id: Option<String>,
    #[props(default = String::new())]
    pub initial_title: String,
    #[props(default = String::new())]
    pub initial_content: String,
}

#[component]
pub fn PostForm(props: PostFormProps) -> Element {
    let client = use_context::<HttpClient>();

    let mut title = use_signal(|| props.initial_title.clone());
    let mut content = use_signal(|| props.initial_content.clone());
    let mut error_message = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    let is_edit_mode = props.post_id.is_some();

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let client = client.clone();
        let post_id = props.post_id.clone();
        let on_success = props.on_success;

        spawn(async move {
            is_loading.set(true);
            error_message.set(None);

            let title_val = title.read().clone();
            let content_val = content.read().clone();

            let result = if let Some(id) = post_id {
                // Update existing post
                client
                    .update_post(&id, &title_val, &content_val)
                    .await
                    .map(|_| ())
            } else {
                // Create new post
                client
                    .create_post(&title_val, &content_val)
                    .await
                    .map(|_| ())
            };

            match result {
                Ok(_) => {
                    is_loading.set(false);
                    on_success.call(());
                }
                Err(err) => {
                    is_loading.set(false);
                    error_message.set(Some(format!("Failed to save post: {:?}", err)));
                }
            }
        });
    };

    rsx! {
        form {
            class: "space-y-4",
            onsubmit: on_submit,

            if let Some(err) = error_message.read().as_ref() {
                div {
                    class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded",
                    p { "{err}" }
                }
            }

            div {
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Title"
                }
                input {
                    class: "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                    r#type: "text",
                    placeholder: "Enter post title",
                    required: true,
                    value: "{title}",
                    oninput: move |evt| title.set(evt.value().clone()),
                }
            }

            div {
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Content"
                }
                textarea {
                    class: "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                    placeholder: "Write your post content here...",
                    required: true,
                    rows: "8",
                    value: "{content}",
                    oninput: move |evt| content.set(evt.value().clone()),
                }
            }

            div {
                class: "flex space-x-3",
                button {
                    class: "flex-1 justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed",
                    r#type: "submit",
                    disabled: is_loading(),
                    if is_loading() {
                        if is_edit_mode {
                            "Updating..."
                        } else {
                            "Creating..."
                        }
                    } else {
                        if is_edit_mode {
                            "Update Post"
                        } else {
                            "Create Post"
                        }
                    }
                }
                button {
                    class: "flex-1 justify-center py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500",
                    r#type: "button",
                    onclick: move |_| props.on_cancel.call(()),
                    "Cancel"
                }
            }
        }
    }
}
