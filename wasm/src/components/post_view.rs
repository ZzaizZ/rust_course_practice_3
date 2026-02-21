use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PostViewProps {
    pub initial_title: String,
    pub initial_content: String,
    pub created_at: String,
    pub updated_at: String,
    pub on_close: EventHandler<()>,
}

#[component]
pub fn PostView(props: PostViewProps) -> Element {
    let title = use_signal(|| props.initial_title.clone());
    let content = use_signal(|| props.initial_content.clone());
    let error_message = use_signal(|| None::<String>);

    rsx! {
        div {
            class: "bg-white rounded-lg shadow-xl max-w-3xl w-full max-h-[90vh] overflow-hidden",
            onclick: move |e| e.stop_propagation(),

            // Header
            div {
                class: "flex items-center justify-between p-6 border-b",
                h2 {
                    class: "text-2xl font-bold text-gray-900",
                    "{title}"
                }
                button {
                    class: "text-gray-400 hover:text-gray-600 transition-colors",
                    onclick: move |_| {
                        props.on_close.call(());
                    },
                    svg {
                        class: "w-6 h-6",
                        xmlns: "http://www.w3.org/2000/svg",
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
            }
            div {
                class: "px-6 text-sm text-gray-500",
                span {
                    class: "mr-4",
                    "Created: {props.created_at}"
                }
                span {
                    "Updated: {props.updated_at}"
                }
            }

            // Content
            div {
                class: "p-6 overflow-y-auto max-h-[calc(90vh-180px)]",

                if let Some(error) = error_message() {
                    div {
                        class: "mb-4 p-4 bg-red-50 border border-red-200 rounded-lg",
                        p {
                            class: "text-red-800 text-sm",
                            "{error}"
                        }
                    }
                }

                div {
                    class: "prose max-w-none",
                    p {
                        class: "text-gray-700 whitespace-pre-wrap",
                        "{content}"
                    }
                }
            }

            // Footer
            div {
                class: "flex justify-end gap-3 p-6 border-t bg-gray-50",
                button {
                    class: "px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors",
                    onclick: move |_| {
                        props.on_close.call(());
                    },
                    "Закрыть"
                }
            }
        }
    }
}
