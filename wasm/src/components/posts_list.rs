use client::{blog_client::BlogClient, http_client::HttpClient};
use dioxus::document::eval;
use dioxus::prelude::*;

use super::{PostCard, PostForm, PostView};

#[derive(Props, Clone, PartialEq)]
pub struct PostsListProps {
    #[props(default = false)]
    pub is_authenticated: bool,
}

#[derive(Clone, Debug)]
struct EditingPost {
    id: String,
    title: String,
    content: String,
}

#[derive(Clone, Debug)]
struct ViewingPost {
    title: String,
    content: String,
    created_at: String,
    updated_at: String,
}

#[component]
pub fn PostsList(props: PostsListProps) -> Element {
    // Получаем клиента из контекста
    let client = use_context::<HttpClient>();
    let client_for_delete = client.clone();

    const PAGE_SIZE: u32 = 10;

    let mut refresh_trigger = use_signal(|| 0);
    let mut current_page = use_signal(|| 0u32);
    let mut show_create_modal = use_signal(|| false);
    let mut editing_post = use_signal(|| None::<EditingPost>);
    let mut viewing_post = use_signal(|| None::<ViewingPost>);
    let mut deleting_post_id = use_signal(|| None::<String>);

    // Используем use_resource для загрузки постов
    let posts_resource = use_resource(move || {
        let client = client.clone();
        let _ = refresh_trigger();
        let page = current_page();
        async move {
            // Получаем список постов
            client
                .list_posts(PAGE_SIZE, page)
                .await
                .map_err(|e| format!("Failed to fetch posts: {:?}", e))
        }
    });

    let open_create_modal = move |_| {
        show_create_modal.set(true);
    };

    let close_create_modal = move |_| {
        show_create_modal.set(false);
    };

    let close_create_modal_click = move |_evt: Event<MouseData>| {
        show_create_modal.set(false);
    };

    let on_create_success = move |_| {
        show_create_modal.set(false);
        current_page.set(0);
        refresh_trigger.set(refresh_trigger() + 1);
    };

    let on_view = move |(_id, title, content, created_at, updated_at): (
        String,
        String,
        String,
        String,
        String,
    )| {
        viewing_post.set(Some(ViewingPost {
            title,
            content,
            created_at,
            updated_at,
        }));
    };

    let close_view_modal = move |_| {
        viewing_post.set(None);
    };

    let on_edit = move |(id, title, content): (String, String, String)| {
        editing_post.set(Some(EditingPost { id, title, content }));
    };

    let close_edit_modal = move |_| {
        editing_post.set(None);
    };

    let close_edit_modal_click = move |_evt: Event<MouseData>| {
        editing_post.set(None);
    };

    let on_edit_success = move |_| {
        editing_post.set(None);
        refresh_trigger.set(refresh_trigger() + 1);
    };

    let on_delete = move |id: String| {
        deleting_post_id.set(Some(id));
    };

    let cancel_delete = move |_| {
        deleting_post_id.set(None);
    };

    let confirm_delete = move |_| {
        if let Some(id) = deleting_post_id.read().clone() {
            let client = client_for_delete.clone();
            spawn(async move {
                match client.delete_post(&id).await {
                    Ok(_) => {
                        deleting_post_id.set(None);
                        refresh_trigger.set(refresh_trigger() + 1);
                    }
                    Err(err) => {
                        // TODO: Show error message
                        eprintln!("Failed to delete post: {:?}", err);
                        deleting_post_id.set(None);
                    }
                }
            });
        }
    };

    // Фокусируем модальное окно создания при открытии для работы ESC
    use_effect(move || {
        if show_create_modal() {
            eval(
                r#"
                setTimeout(() => {
                    const modals = document.querySelectorAll('.modal-backdrop');
                    if (modals.length > 0) modals[modals.length - 1].focus();
                }, 0);
                "#,
            );
        }
    });

    // Фокусируем модальное окно редактирования при открытии для работы ESC
    use_effect(move || {
        if editing_post.read().is_some() {
            eval(
                r#"
                setTimeout(() => {
                    const modals = document.querySelectorAll('.modal-backdrop');
                    if (modals.length > 0) modals[modals.length - 1].focus();
                }, 0);
                "#,
            );
        }
    });

    // Фокусируем модальное окно удаления при открытии для работы ESC
    use_effect(move || {
        if deleting_post_id.read().is_some() {
            eval(
                r#"
                setTimeout(() => {
                    const modals = document.querySelectorAll('.modal-backdrop');
                    if (modals.length > 0) modals[modals.length - 1].focus();
                }, 0);
                "#,
            );
        }
    });

    // Фокусируем модальное окно просмотра при открытии для работы ESC
    use_effect(move || {
        if viewing_post.read().is_some() {
            eval(
                r#"
                setTimeout(() => {
                    const modals = document.querySelectorAll('.modal-backdrop');
                    if (modals.length > 0) modals[modals.length - 1].focus();
                }, 0);
                "#,
            );
        }
    });

    rsx! {
        div {
            class: "posts-list-container max-w-4xl mx-auto p-6",
            div {
                class: "flex justify-between items-center mb-8",
                h1 {
                    class: "text-4xl font-bold",
                    "Blog Posts"
                }
                if props.is_authenticated {
                    button {
                        class: "px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500",
                        onclick: open_create_modal,
                        "+ Create Post"
                    }
                }
            }

            match posts_resource.read().as_ref() {
                None => rsx! {
                    div {
                        class: "flex justify-center items-center py-12",
                        div {
                            class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"
                        }
                    }
                },
                Some(Err(err)) => rsx! {
                    div {
                        class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded",
                        p { "Error: {err}" }
                    }
                },
                Some(Ok(posts)) => {
                    let has_prev = current_page() > 0;
                    let has_next = posts.len() == PAGE_SIZE as usize;

                    if posts.is_empty() && current_page() == 0 {
                        rsx! {
                            div {
                                class: "text-center py-12",
                                p {
                                    class: "text-gray-600 text-lg",
                                    "No posts found. Create your first post!"
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div {
                                class: "posts-grid",
                                for post in posts {
                                    PostCard {
                                        id: post.id.to_string(),
                                        title: post.title.clone(),
                                        content: post.content.clone(),
                                        created_at: post.created_at.format("%Y-%m-%d %H:%M").to_string(),
                                        updated_at: post.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                                        is_authenticated: props.is_authenticated,
                                        on_view: on_view,
                                        on_edit: on_edit,
                                        on_delete: on_delete,
                                    }
                                }
                            }

                            // Pagination controls
                            div {
                                class: "flex justify-center items-center mt-8 space-x-4",
                                button {
                                    class: "px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: !has_prev,
                                    onclick: move |_| {
                                        if current_page() > 0 {
                                            current_page.set(current_page() - 1);
                                        }
                                    },
                                    "← Previous"
                                }
                                span {
                                    class: "text-sm",
                                    "Page {current_page() + 1}"
                                }
                                button {
                                    class: "px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: !has_next,
                                    onclick: move |_| {
                                        current_page.set(current_page() + 1);
                                    },
                                    "Next →"
                                }
                            }
                        }
                    }
                }
            }

            // Create Post Modal
            if show_create_modal() {
                div {
                    class: "fixed inset-0 modal-backdrop overflow-y-auto h-full w-full z-50 flex items-center justify-center",
                    onclick: close_create_modal_click,
                    onkeyup: move |evt: Event<KeyboardData>| {
                        if evt.key() == Key::Escape {
                            show_create_modal.set(false);
                        }
                    },
                    tabindex: 0,
                    div {
                        class: "relative bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4",
                        onclick: move |evt| evt.stop_propagation(),

                        button {
                            class: "absolute top-4 right-4 text-gray-400 hover:text-gray-600",
                            onclick: close_create_modal_click,
                            svg {
                                class: "h-6 w-6",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M6 18L18 6M6 6l12 12"
                                }
                            }
                        }

                        div {
                            class: "p-6",
                            h2 {
                                class: "text-2xl font-bold text-gray-900 mb-4",
                                "Create New Post"
                            }
                            PostForm {
                                on_success: on_create_success,
                                on_cancel: close_create_modal,
                            }
                        }
                    }
                }
            }

            // Edit Post Modal
            if let Some(post) = editing_post.read().as_ref() {
                div {
                    class: "fixed inset-0 modal-backdrop overflow-y-auto h-full w-full z-50 flex items-center justify-center",
                    onclick: close_edit_modal_click,
                    onkeyup: move |evt: Event<KeyboardData>| {
                        if evt.key() == Key::Escape {
                            editing_post.set(None);
                        }
                    },
                    tabindex: 0,
                    div {
                        class: "relative bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4",
                        onclick: move |evt| evt.stop_propagation(),

                        button {
                            class: "absolute top-4 right-4 text-gray-400 hover:text-gray-600",
                            onclick: close_edit_modal_click,
                            svg {
                                class: "h-6 w-6",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M6 18L18 6M6 6l12 12"
                                }
                            }
                        }

                        div {
                            class: "p-6",
                            h2 {
                                class: "text-2xl font-bold text-gray-900 mb-4",
                                "Edit Post"
                            }
                            PostForm {
                                post_id: post.id.clone(),
                                initial_title: post.title.clone(),
                                initial_content: post.content.clone(),
                                on_success: on_edit_success,
                                on_cancel: close_edit_modal,
                            }
                        }
                    }
                }
            }

            // Delete Confirmation Modal
            if deleting_post_id.read().is_some() {
                div {
                    class: "fixed inset-0 modal-backdrop overflow-y-auto h-full w-full z-50 flex items-center justify-center",
                    onclick: cancel_delete,
                    onkeyup: move |evt: Event<KeyboardData>| {
                        if evt.key() == Key::Escape {
                            deleting_post_id.set(None);
                        }
                    },
                    tabindex: 0,
                    div {
                        class: "relative bg-white rounded-lg shadow-xl max-w-md w-full mx-4",
                        onclick: move |evt| evt.stop_propagation(),

                        div {
                            class: "p-6",
                            h2 {
                                class: "text-xl font-bold text-gray-900 mb-4",
                                "Delete Post"
                            }
                            p {
                                class: "text-gray-600 mb-6",
                                "Are you sure you want to delete this post? This action cannot be undone."
                            }
                            div {
                                class: "flex space-x-3",
                                button {
                                    class: "flex-1 justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500",
                                    onclick: confirm_delete,
                                    "Delete"
                                }
                                button {
                                    class: "flex-1 justify-center py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500",
                                    onclick: cancel_delete,
                                    "Cancel"
                                }
                            }
                        }
                    }
                }
            }

            // View Post Modal
            if let Some(post) = viewing_post() {
                div {
                    class: "fixed inset-0 modal-backdrop overflow-y-auto h-full w-full z-50 flex items-center justify-center",
                    onclick: move |_| {
                        viewing_post.set(None);
                    },
                    onkeyup: move |evt: Event<KeyboardData>| {
                        if evt.key() == Key::Escape {
                            viewing_post.set(None);
                        }
                    },
                    tabindex: 0,
                    PostView {
                        initial_title: post.title,
                        initial_content: post.content,
                        created_at: post.created_at,
                        updated_at: post.updated_at,
                        on_close: close_view_modal,
                    }
                }
            }
        }
    }
}
