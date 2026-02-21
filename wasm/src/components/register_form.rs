use client::{blog_client::BlogClient, http_client::HttpClient};
use dioxus::prelude::*;

#[component]
pub fn RegisterForm(on_success: EventHandler<()>, on_switch_to_login: EventHandler<()>) -> Element {
    let client = use_context::<HttpClient>();

    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        let client = client.clone();
        spawn(async move {
            is_loading.set(true);
            error_message.set(None);

            let username_val = username.read().clone();
            let email_val = email.read().clone();
            let password_val = password.read().clone();
            let confirm_password_val = confirm_password.read().clone();

            // Validate passwords match
            if password_val != confirm_password_val {
                is_loading.set(false);
                error_message.set(Some("Passwords do not match".to_string()));
                return;
            }

            match client
                .register(&username_val, &email_val, &password_val)
                .await
            {
                Ok(_) => {
                    is_loading.set(false);
                    // После успешной регистрации автоматически логинимся
                    match client.login(&username_val, &password_val).await {
                        Ok(_user_id) => {
                            on_success.call(());
                        }
                        Err(err) => {
                            error_message.set(Some(format!(
                                "Registration successful but login failed: {:?}",
                                err
                            )));
                        }
                    }
                }
                Err(err) => {
                    is_loading.set(false);
                    error_message.set(Some(format!("Registration failed: {:?}", err)));
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
                    "Username"
                }
                input {
                    class: "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                    r#type: "text",
                    placeholder: "Choose a username",
                    required: true,
                    value: "{username}",
                    oninput: move |evt| username.set(evt.value().clone()),
                }
            }

            div {
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Email"
                }
                input {
                    class: "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                    r#type: "email",
                    placeholder: "your@email.com",
                    required: true,
                    value: "{email}",
                    oninput: move |evt| email.set(evt.value().clone()),
                }
            }

            div {
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Password"
                }
                input {
                    class: "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                    r#type: "password",
                    placeholder: "Create a password",
                    required: true,
                    value: "{password}",
                    oninput: move |evt| password.set(evt.value().clone()),
                }
            }

            div {
                label {
                    class: "block text-sm font-medium text-gray-700 mb-1",
                    "Confirm Password"
                }
                input {
                    class: "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm",
                    r#type: "password",
                    placeholder: "Confirm your password",
                    required: true,
                    value: "{confirm_password}",
                    oninput: move |evt| confirm_password.set(evt.value().clone()),
                }
            }

            div {
                button {
                    class: "w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed",
                    r#type: "submit",
                    disabled: is_loading(),
                    if is_loading() {
                        "Creating account..."
                    } else {
                        "Create account"
                    }
                }
            }

            div {
                class: "text-center text-sm text-gray-600",
                "Already have an account? "
                button {
                    class: "font-medium text-blue-600 hover:text-blue-500",
                    r#type: "button",
                    onclick: move |_| on_switch_to_login.call(()),
                    "Sign in"
                }
            }
        }
    }
}
