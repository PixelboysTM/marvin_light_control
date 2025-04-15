use std::ops::{Deref, DerefMut};

use crate::toaster::ToastInfo;
use dioxus::desktop::use_window;
use dioxus::logger::tracing::info;
use dioxus::{document::eval, prelude::*};
use dioxus_free_icons::{icons::ld_icons::LdX, Icon, IconShape};
use mlc_communication::remoc::rch::mpsc::{Receiver, RecvError};
use uuid::Uuid;

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
    style: Option<String>,
    text: Option<String>,
    onclick: Option<EventHandler<Event<MouseData>>>,
) -> Element {
    rsx! {
        button {
            class: format!("iconBtn {}", if let Some(c) = class { c } else { "".to_string() }),
            style,
            onclick: move |v| {
                if let Some(c) = onclick {
                    c.call(v);
                }
            },
            Icon { icon }
            if let Some(s) = text {
                span {
                    {s}
                }
            }
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
                div { class: "content",
                    ErrorBoundary {
                        handle_error: move |e: ErrorContext| {
                            rsx! {
                                for err in e.errors() {
                                    code { style: "color: var(--c-err);", {err.to_string()} }
                                }
                            }
                        },
                        SuspenseBoundary {
                            fallback: move |_| rsx! {
                                Loader {}
                            },
                            {children}
                        }
                    }
                }
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

#[derive(Debug)]
pub enum Screen {
    Connect,
    ProjectList,
    Configure,
    Program,
    Show,
}

pub fn navigate(screen: Screen) {
    info!("Navigating to {:?}", screen);

    navigator()
        .replace(match screen {
            Screen::Connect => "/",
            Screen::ProjectList => "/projects",
            Screen::Configure => "/project/configure",
            Screen::Program => "/project/program",
            Screen::Show => "/project/show",
        })
        .map(|s| ToastInfo::error("Failed to change screen", s.0));

    use_window().window.set_title(match screen {
        Screen::Connect => "Marvin Light Control",
        Screen::ProjectList => "Marvin Light Control | Project Selection",
        Screen::Configure => "Marvin Light Control | Configure",
        Screen::Program => "Marvin Light Control | Program",
        Screen::Show => "Marvin Light Control | Show",
    });
}

#[component]
pub fn Panel(
    children: Option<Element>,
    column: String,
    row: String,
    title: Option<String>,
    class: Option<String>,
) -> Element {
    let has_title = title.is_some();

    rsx! {
        div {
            class: format!("panel {} {}", if has_title {"withTitle"} else {""}, class.unwrap_or_default()),
            style: format!("grid-column: {column}; grid-row: {row}"),
            if let Some(title) = title {
                h1 { class: "title", {title} }
            }
            SuspenseBoundary {
                fallback: move |_| {
                    rsx! {
                        Loader {}
                    }
                },
                {children}
            }
        }
    }
}

pub trait TabItem: PartialEq + Clone {
    fn get_name(&self) -> String;
}

pub trait TabController: PartialEq {
    type Item: TabItem;
    fn get_options(&self) -> Vec<Self::Item>;
    fn set(&mut self, option: Self::Item);
    fn get(&self) -> Self::Item;
}

#[derive(Copy, Clone, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappedVecTabs<I: TabItem + 'static> {
    options: MappedSignal<Vec<I>>,
    current: I,
}

impl<I: TabItem + 'static> MappedVecTabs<I> {
    pub fn new(options: MappedSignal<Vec<I>>, initial: I) -> MappedVecTabs<I> {
        Self {
            options,
            current: initial,
        }
    }
}

impl<I: TabItem + 'static> TabController for MappedVecTabs<I> {
    type Item = I;

    fn get_options(&self) -> Vec<Self::Item> {
        self.options.read().clone()
    }

    fn set(&mut self, option: Self::Item) {
        self.current = option;
    }

    fn get(&self) -> Self::Item {
        self.current.clone()
    }
}

impl TabItem for u16 {
    fn get_name(&self) -> String {
        self.to_string()
    }
}

#[component]
pub fn Tabs<T: TabController + 'static>(
    controller: Signal<T>,
    orientation: Orientation,
    class: Option<String>,
) -> Element {
    rsx! {
        div {
            class: format!("tabBar {} {}", match orientation {
                Orientation::Horizontal => " tab-orientation-horizontal",
                Orientation::Vertical => "tab-orientation-vertical",
            }, class.unwrap_or_default()),
            for option in controller.read().get_options() {
                button {
                    class: if option == controller.read().get() { "selected"},
                    onclick: move |e| {
                        controller.write().set(option.clone());
                        e.prevent_default();
                    },
                    {option.get_name()}
                }
            }
        }
    }
}

#[component]
pub fn Fader(
    value: MappedSignal<u8>,
    update: EventHandler<u8>,
    orientation: Option<Orientation>,
    class: Option<String>,
) -> Element {
    rsx! {
        // div {
        //     class: format!("widgetFader {} {}", match orientation.unwrap_or(Orientation::Vertical) {
        //         Orientation::Horizontal => " fader-orientation-horizontal",
        //         Orientation::Vertical => "fader-orientation-vertical",
        //     }, class.unwrap_or_default()),
        //     style: format!("--dw-fv: {};", *value.read() as f32 / 255.0 * 100.0),
        //     div {
        //         class: "track"
        //     }
        //     div {
        //         class: "knob"
        //     }
        // }

        input {
            class: format!("fader {}", class.unwrap_or_default()),
            value: 255 - value(),
            onchange: move |d| {
                if let Ok(v) = d.value().parse::<u8>() {
                    update.call(255 - v);
                }
            },
            r#type: "range",
            min: 0,
            max: 255
        }
    }
}

pub struct UniqueEq<T> {
    value: T,
    id: Uuid,
}

impl<T: Clone> Clone for UniqueEq<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            id: self.id,
        }
    }
}

impl<T> PartialEq for UniqueEq<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Deref for UniqueEq<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for UniqueEq<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> From<T> for UniqueEq<T> {
    fn from(value: T) -> Self {
        UniqueEq {
            value,
            id: Uuid::new_v4(),
        }
    }
}

pub struct SignalNotify(GlobalSignal<()>);

impl Deref for SignalNotify {
    type Target = GlobalSignal<()>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SignalNotify {
    pub const fn create() -> Self {
        Self(Signal::global(|| ()))
    }

    pub fn update(&self) {
        *self.0.write() = ()
    }
}

pub async fn some_recv<T>(recv: Option<&mut Receiver<T>>) -> Result<Option<T>, RecvError> {
    if let Some(recv) = recv {
        recv.recv().await
    } else {
        futures::future::pending().await
    }
}
