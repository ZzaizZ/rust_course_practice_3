use super::{LoginForm, PostsList, RegisterForm};
use crate::storage;
use client::{blog_client::BlogClient, http_client::HttpClient};
use dioxus::document::eval;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum AuthView {
    Login,
    Register,
}

#[component]
pub fn AuthenticatedApp() -> Element {
    let client = use_context::<HttpClient>();
    let mut is_authenticated = use_signal(|| false);
    let mut is_checking_auth = use_signal(|| true);
    let mut show_auth_modal = use_signal(|| false);
    let mut current_auth_view = use_signal(|| AuthView::Login);

    use_effect(move || {
        let client = client.clone();
        spawn(async move {
            if let Some(auth_data) = storage::load_auth_data() {
                if client.setup_auth_data(&auth_data).await.is_ok() {
                    is_authenticated.set(true);
                    is_checking_auth.set(false);
                    return;
                }
            }

            is_checking_auth.set(false);
        });
    });

    let on_auth_success = move |_| {
        is_authenticated.set(true);
        show_auth_modal.set(false);
    };

    let on_logout = move |_| {
        // Очищаем данные аутентификации из localStorage
        storage::clear_auth_data();
        is_authenticated.set(false);
    };

    let open_login = move |_| {
        current_auth_view.set(AuthView::Login);
        show_auth_modal.set(true);
    };

    let open_register = move |_| {
        current_auth_view.set(AuthView::Register);
        show_auth_modal.set(true);
    };

    let close_modal = move |_| {
        show_auth_modal.set(false);
    };

    // Фокусируем модальное окно при открытии для работы ESC
    use_effect(move || {
        if show_auth_modal() {
            eval(
                r#"
                setTimeout(() => {
                    const modal = document.querySelector('.modal-backdrop');
                    if (modal) modal.focus();
                }, 0);
                "#,
            );
        }
    });

    let switch_to_register = move |_| {
        current_auth_view.set(AuthView::Register);
    };

    let switch_to_login = move |_| {
        current_auth_view.set(AuthView::Login);
    };

    if is_checking_auth() {
        return rsx! {
            div {
                class: "flex justify-center items-center min-h-screen",
                div {
                    class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"
                }
            }
        };
    }

    rsx! {
        div {
            // Header with auth buttons or logout
            nav {
                class: "bg-white shadow-sm border-b border-gray-200",
                div {
                    class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    div {
                        class: "flex justify-between h-16",
                        div {
                            class: "flex items-center",
                            h1 {
                                class: "text-xl font-bold text-gray-900",
                                "Blog Application"
                            }
                        }
                        div {
                            class: "flex items-center space-x-4",
                            if is_authenticated() {
                                button {
                                    class: "px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500",
                                    onclick: on_logout,
                                    "Logout"
                                }
                            } else {
                                button {
                                    class: "px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500",
                                    onclick: open_login,
                                    "Sign In"
                                }
                                button {
                                    class: "px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500",
                                    onclick: open_register,
                                    "Sign Up"
                                }
                            }
                        }
                    }
                }
            }

            // Main content
            main {
                PostsList {
                    is_authenticated: is_authenticated()
                }
            }

            // Auth Modal
            if show_auth_modal() {
                div {
                    class: "fixed inset-0 modal-backdrop overflow-y-auto h-full w-full z-50 flex items-center justify-center",
                    onclick: close_modal,
                    onkeyup: move |evt: Event<KeyboardData>| {
                        if evt.key() == Key::Escape {
                            show_auth_modal.set(false);
                        }
                    },
                    tabindex: 0,
                    div {
                        class: "relative bg-white rounded-lg shadow-xl max-w-md w-full mx-4",
                        onclick: move |evt| evt.stop_propagation(),

                        // Close button
                        button {
                            class: "absolute top-4 right-4 text-gray-400 hover:text-gray-600",
                            onclick: close_modal,
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

                        // Auth form content
                        div {
                            class: "p-6",
                            match current_auth_view() {
                                AuthView::Login => rsx! {
                                    div {
                                        h2 {
                                            class: "text-2xl font-bold text-gray-900 mb-4",
                                            "Sign In"
                                        }
                                        LoginForm {
                                            on_success: on_auth_success,
                                            on_switch_to_register: switch_to_register,
                                        }
                                    }
                                },
                                AuthView::Register => rsx! {
                                    div {
                                        h2 {
                                            class: "text-2xl font-bold text-gray-900 mb-4",
                                            "Create Account"
                                        }
                                        RegisterForm {
                                            on_success: on_auth_success,
                                            on_switch_to_login: switch_to_login,
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}
