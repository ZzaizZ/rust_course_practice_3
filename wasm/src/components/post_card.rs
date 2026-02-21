use dioxus::prelude::*;

type PostViewData = (String, String, String, String, String);
type PostEditData = (String, String, String);

#[component]
pub fn PostCard(
    id: String,
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
    #[props(default = false)] is_authenticated: bool,
    on_view: Option<EventHandler<PostViewData>>,
    on_edit: Option<EventHandler<PostEditData>>,
    on_delete: Option<EventHandler<String>>,
) -> Element {
    rsx! {
        div {
            class: "post-card bg-white rounded-lg shadow-md p-6 mb-4 hover:shadow-lg transition-shadow",
            div {
                class: "post-header mb-3",
                div {
                    class: "flex justify-between items-start",
                    h2 {
                        class: "text-2xl font-bold text-gray-800 flex-1 cursor-pointer hover:text-blue-600 transition-colors",
                        onclick: move |_| {
                            if let Some(on_view_handler) = on_view {
                                on_view_handler.call((id.clone(), title.clone(), content.clone(), created_at.clone(), updated_at.clone()));
                            }
                        },
                        "{title}"
                    }
                    if is_authenticated {
                        div {
                            class: "flex space-x-2 ml-4",
                            if let Some(on_edit_handler) = on_edit {
                                button {
                                    class: "px-3 py-1 text-sm font-medium text-blue-600 bg-blue-50 rounded hover:bg-blue-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                                    onclick: {
                                        let id = id.clone();
                                        let title = title.clone();
                                        let content = content.clone();
                                        move |_| on_edit_handler.call((id.clone(), title.clone(), content.clone()))
                                    },
                                    "Edit"
                                }
                            }
                            if let Some(on_delete_handler) = on_delete {
                                button {
                                    class: "px-3 py-1 text-sm font-medium text-red-600 bg-red-50 rounded hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-red-500",
                                    onclick: {
                                        let id = id.clone();
                                        move |_| on_delete_handler.call(id.clone())
                                    },
                                    "Delete"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "text-sm text-gray-500 mt-2",
                    span {
                        class: "mr-4",
                        "Created: {created_at}"
                    }
                    span {
                        "Updated: {updated_at}"
                    }
                }
            }
            div {
                class: "post-content text-gray-700 line-clamp-2",
                p { "{content}" }
            }
        }
    }
}
