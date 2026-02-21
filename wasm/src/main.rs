use dioxus::prelude::*;

use client::{blog_client::BlogClient, http_client::HttpClient, TokenUpdateEvent};
use components::AuthenticatedApp;
use tokio::sync::mpsc;

mod components;
mod storage;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// Получает URL backend-а из переменных окружения или использует значение по умолчанию
fn get_backend_url() -> String {
    option_env!("BACKEND_URL")
        .unwrap_or("http://localhost:8080")
        .to_string()
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let client_resource = use_resource(|| async move {
        let backend_url = get_backend_url();
        let (token_sender, mut token_receiver) = mpsc::unbounded_channel::<TokenUpdateEvent>();
        let client = HttpClient::new_with_token_notifier(backend_url, token_sender).await?;

        let client_for_storage = client.clone();
        spawn(async move {
            while let Some(_event) = token_receiver.recv().await {
                if let Ok(Some(auth_data)) = client_for_storage.get_auth_data().await {
                    if let Err(e) = storage::save_auth_data(&auth_data) {
                        eprintln!("Failed to save auth data to localStorage: {:?}", e);
                    }
                }
            }
        });

        Ok::<_, client::error::ClientError>(client)
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        match client_resource.read().as_ref() {
            None => rsx! {
                div {
                    class: "flex justify-center items-center min-h-screen",
                    div {
                        class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"
                    }
                }
            },
            Some(Err(err)) => rsx! {
                div {
                    class: "flex justify-center items-center min-h-screen",
                    div {
                        class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded",
                        p { "Failed to initialize client: {err:?}" }
                    }
                }
            },
            Some(Ok(client)) => rsx! {
                { use_context_provider(|| client.clone()); }
                AuthenticatedApp {}
            }
        }
    }
}
