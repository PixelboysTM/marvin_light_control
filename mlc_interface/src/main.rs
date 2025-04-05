use connect::connect_url;
use dioxus::desktop::{LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use dioxus::{desktop::Config, logger::tracing::error};
use mlc_communication::services::general::{GeneralService, GeneralServiceIdent};
use std::net::Ipv4Addr;
use std::string::ToString;
use toaster::{ToastInfo, ToasterProvider};
use utils::{Loader};
use screens::projects::Project;
use crate::utils::{navigate, Screen};

mod connect;
mod toaster;
mod utils;

mod screens;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Connect {},
    #[route("/projects")]
    Project {  },
    #[layout(ProjectLayout)]
    #[nest("/project")]
        #[route("/configure")]
        Configure {},
        #[route("/program")]
        Program {},
        #[route("/show")]
        Show {},
    #[end_nest]
    #[route("/view")]
    View {},
}

const FAVICON: Asset = asset!("/assets/icon.png");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const UTILS_CSS: Asset = asset!("/assets/utils.css");

fn main() {
    LaunchBuilder::new()
        .with_cfg(
            Config::default().with_menu(None).with_window(
                WindowBuilder::new()
                    .with_maximized(false)
                    .with_always_on_top(false)
                    .with_inner_size(LogicalSize::new(800, 600))
                    .with_resizable(true)
                    .with_title("Marvin Light Control"),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: UTILS_CSS }
        ToasterProvider {}
        SuspenseBoundary {
            fallback: move |_| {
                rsx! {
                    Loader {}
                }
            },
            Router::<Route> {}
        }
    }
}

const CONNECT_URL: GlobalSignal<(Ipv4Addr, u16)> = Signal::global(|| (Ipv4Addr::LOCALHOST, 8181));

const CONNECT_CSS: Asset = asset!("/assets/connect.css");
#[component]
fn Connect() -> Element {
    let mut connect_addr = use_signal(|| ("127.0.0.1".to_string(), "8181".to_string()));
    let mut loading = use_signal(|| false);

    let connect_handler = move || async move {
        *loading.write() = true;
        let c = connect_addr.read();
        if let (Ok(addr), Ok(port)) = (c.0.parse::<Ipv4Addr>(), c.1.parse::<u16>()) {
            match connect_url::<GeneralServiceIdent>((addr, port)).await {
                Ok(client) => {
                    if client.alive().await.is_ok() {
                        *CONNECT_URL.write() = (addr, port);
                        ToastInfo::info(
                            "Connection successful",
                            "Connection to the backend could be established.",
                        )
                            .post();
                        let nav = navigator();
                        navigate(Screen::ProjectList);
                        // nav.replace("/projects");
                    }
                }
                Err(e) => {
                    ToastInfo::error(
                        "Connection error",
                        "Connecting to backend failed. Please see logs for more information",
                    )
                        .post();
                    error!("Error occurred: {e:?}");
                }
            }
        } else {
            ToastInfo::error(
                "Invalid Address",
                "Not a valid input address provided. Could not try to connect",
            )
                .post();
        }
        *loading.write() = false;
    };

    use_future(connect_handler);

    rsx! {
        match loading() {
            false => rsx! {
                document::Stylesheet { href: CONNECT_CSS }
                fieldset { class: "connect",
                    legend { "Connection Address:" }
                    input {
                        r#type: "text",
                        value: connect_addr.map(|c| &c.0),
                        class: "addr",
                        onchange: move |v| {
                            let mut addr = connect_addr.write();
                            addr.0 = v.value();
                        },
                    }
                    span { class: "divider", ":" }
                    input {
                        r#type: "text",
                        value: connect_addr.map(|c| &c.1),
                        class: "port",
                        onchange: move |v| {
                            let mut addr = connect_addr.write();
                            addr.1 = v.value();
                        },
                    }
                    button {
                        onclick: move |_| connect_handler(),
                        "Connect"
                    }
                }
            },
            true => rsx! {
                Loader {}
            },
        }
    }
}



#[component]
fn Configure() -> Element {
    rsx! { "Configure" }
}

#[component]
fn Program() -> Element {
    rsx! { "Program" }
}

#[component]
fn Show() -> Element {
    rsx! { "Show" }
}

#[component]
fn View() -> Element {
    rsx! { "View" }
}

#[component]
fn ProjectLayout() -> Element {
    rsx! {
        header { "header" }
        Outlet::<Route> {}
        footer { "footer" }
    }
}