use connect::connect_url;
use dioxus::desktop::{LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use dioxus::{desktop::Config, logger::tracing::error};
use mlc_communication::services::general::{GeneralService, GeneralServiceIdent, Info};
use std::net::Ipv4Addr;
use std::string::ToString;
use std::time::{Duration, Instant};
use dioxus_free_icons::icons::ld_icons::{LdCloudUpload, LdCog, LdLightbulb, LdPencil, LdSave, LdTabletSmartphone};
use log::{info, warn};
use tokio::select;
use tokio::time::sleep;
use toaster::{ToastInfo, ToasterProvider};
use utils::{Loader};
use screens::projects::Project;
use crate::connect::connect;
use crate::utils::{navigate, Branding, IconButton, Screen};
use mlc_communication::services::general::View as SView;

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


const PROJECT_COMMON: Asset = asset!("/assets/project_common.css");
#[component]
fn ProjectLayout() -> Element {
    let mut delay = use_signal(|| 0);
    let mut status_msg = use_signal(|| "Idling".to_string());

    let gen_client = use_resource(connect::<GeneralServiceIdent>);
    use_future(move || async move {
        let mut needs_connect = true;

        let mut info_sub = None;
        let mut status_sub = None;

        async fn recv<T: Clone>(sub: &mut Option<mlc_communication::remoc::rch::watch::Receiver<T>>) -> T {
            match sub {
                Some(sub) => {
                    sub.changed().await.unwrap();
                    sub.borrow_and_update().unwrap().clone()
                }
                None => {
                    futures::future::pending().await
                }
            }
        }

        let mut counter = 0;
        loop {
            counter = (counter + 1) % 10;

            select! {
                i = recv(&mut info_sub) => {
                    match i {
                        Info::Autosaved => {
                            ToastInfo::info("Autosaved", "The backend autosaved").post();
                        }
                        Info::Shutdown => {
                            ToastInfo::info("Shutdown", "The backend shutdown!s").post();
                            navigate(Screen::ProjectList);
                        }
                        Info::Idle => {}
                    }
                },
                s = recv(&mut status_sub) => {
                    status_msg.set(s);
                },
                _ = sleep(Duration::from_millis(50)) => {
                    match &*gen_client.read() {
                        None => {},
                        Some(Err(e)) => {
                            error!("Error occurred: {e:?}");
                            return;
                        }
                        Some(Ok(c)) => {
                            if needs_connect {
                                needs_connect = false;

                                let is_valid = c.is_valid_view(SView::Edit).await;
                                if !matches!(is_valid, Ok(true)) {
                                    navigate(Screen::ProjectList);
                                }

                                if let Ok(info) = c.info().await {
                                    info_sub = Some(info);
                                }

                                if let Ok(s) = c.status().await {
                                    status_sub = Some(s);
                                }
                            }

                            if counter == 0 {
                                let timer = Instant::now();
                                let _ = c.alive().await;
                                let t = timer.elapsed();
                                *delay.write() = t.as_millis() as u64;
                            }
                        }
                    }
                }
            }
        }
    });

    let r: Route = use_route();
    let extra_actions = match r {
        Route::Configure {} => rsx! {
            IconButton {
                icon: LdCloudUpload,
                onclick: move |_| {
                    info!("Open fixture adder");
                }
            }
        },
        Route::Program {} => rsx! {},
        Route::Show {} => rsx! {
            IconButton {
                icon: LdTabletSmartphone,
                onclick: move |_| {
                    info!("Open mobile connector");
                }
            }
        },
        _ => {
            warn!("Shouldn't be here");
            rsx! {}
        },
    };

    rsx! {
        div {
            class: "projectContainer",
            document::Stylesheet { href: PROJECT_COMMON }
            nav {
                Branding {}
                div {
                    class: "viewSelect",
                    IconButton {
                        class: if matches!(r, Route::Configure {}) {"curr"},
                        style: "--c-cl: var(--c-p);",
                        icon: LdCog,
                        onclick: move |_| {
                            navigate(Screen::Configure);
                        }
                    }
                    IconButton {
                        class: if matches!(r, Route::Program {}) {"curr"},
                        style: "--c-cl: var(--c-s);",
                        icon: LdPencil,
                        onclick: move |_| {
                            navigate(Screen::Program);
                        }
                    }
                    IconButton {
                        class: if matches!(r, Route::Show {}) {"curr"},
                        style: "--c-cl: var(--c-t);",
                        icon: LdLightbulb,
                        onclick: move |_| {
                            navigate(Screen::Show);
                        }
                    }
                }
                div { class: "actions",
                    {extra_actions},
                    IconButton {
                        icon: LdSave,
                        onclick: async |_| {
                            info!("Saving");
                            // CREATE_PROJECT.open().await;
                        },
                    }
                }
            }
            Outlet::<Route> {}
            footer { {format!("Ping: {}ms, Status: {}", delay.read(), status_msg.read()) } }
        }
    }
}