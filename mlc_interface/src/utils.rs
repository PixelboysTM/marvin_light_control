use dioxus::{document::eval, prelude::*};
use dioxus_free_icons::{icons::ld_icons::LdX, Icon, IconShape};

#[component]
pub fn Loader() -> Element {
    rsx! {
        Center {
            div { class: "loaderElement" }
        }
    }
}

#[component]
pub fn Center(children: Element) -> Element {
    rsx! {
        div { class: "centerElement", {children} }
    }
}

const FAVICON: Asset = asset!("/assets/icon.png");

#[component]
pub fn Branding() -> Element {
    rsx! {
        div { class: "brandingElement",
            img { src: FAVICON }
            h1 { class: "txt", "MLC" }
        }
    }
}

#[component]
pub fn IconButton<I: IconShape + Clone + PartialEq + 'static>(
    icon: I,
    class: Option<String>,
    onclick: Option<EventHandler<Event<MouseData>>>,
) -> Element {
    rsx! {
        button {
            class: format!("iconBtn {}", if let Some(c) = class {c} else { "".to_string() }),
            onclick: move |v| {
                if let Some(c) = onclick {
                    c.call(v);
                }
            },
            Icon { icon }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol(&'static str);

impl Symbol {
    pub const fn create(ident: &'static str) -> Self {
        Self(ident)
    }

    pub async fn open(&self) {
        let _ = eval(&format!(
            "document.getElementById('{}').showModal()",
            self.0
        ))
        .await
        .unwrap();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalVariant {
    Ok,
    OkCancel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModalResult {
    Success,
    Cancel,
}

#[component]
pub fn Modal<
    T: Into<String> + Clone + PartialEq + 'static,
    I: IconShape + Clone + PartialEq + 'static,
>(
    title: T,
    ident: Symbol,
    variant: ModalVariant,
    icon: I,
    children: Option<Element>,
    onexit: Option<EventHandler<ModalResult>>,
    oktext: Option<String>,
    canceltext: Option<String>,
) -> Element {
    rsx! {
        dialog { id: ident.0, class: "modalDialog",
            form { class: "dialogForm", method: "dialog",
                div { class: "header",
                    Icon { icon, class: "ico" }
                    h1 { {title.into()} }
                    IconButton {
                        icon: LdX,
                        onclick: move |_| {
                            if let Some(c) = onexit {
                                c.call(ModalResult::Cancel);
                            }
                        },
                    }
                }
                div { class: "content", {children} }
                div { class: "footer",
                    button {
                        onclick: move |_| {
                            if let Some(c) = onexit {
                                c.call(ModalResult::Success);
                            }
                        },
                        {oktext.unwrap_or("Ok".to_string())}
                    }
                    if variant == ModalVariant::OkCancel {
                        button {
                            onclick: move |_| {
                                if let Some(c) = onexit {
                                    c.call(ModalResult::Cancel);
                                }
                            },
                            {canceltext.unwrap_or("Cancel".to_string())}
                        }
                    }
                }
            }
        }
    }
}
