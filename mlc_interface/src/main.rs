use crate::screens::BLUEPRINTS_CHANGED;
use crate::utils::{navigate, Branding, IconButton, ModalResult, Screen};
use connect::{connect_url, use_service, RtcSuspend, SClient};
use dioxus::desktop::{LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use dioxus::{desktop::Config, logger::tracing::error};
use dioxus_free_icons::icons::ld_icons::{
    LdCloudUpload, LdCog, LdLamp, LdLightbulb, LdPencil, LdSave, LdTabletSmartphone,
};
use itertools::Itertools;
use log::{info, warn};
use mlc_communication::services::general::{GeneralService, GeneralServiceIdent, Info};
use mlc_communication::services::general::{ProjectInfo, View as SView};
use screens::{Configure, Program, Projects, Show};
use std::{
    net::Ipv4Addr,
    string::ToString,
    time::{Duration, Instant},
};
use toaster::{ToastInfo, ToasterProvider};
use tokio::select;
use utils::{Loader, Modal, ModalVariant, Symbol};

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
    Projects {  },
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
                    .with_always_on_top(true)
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
        document::Stylesheet { href: CONNECT_CSS }
        match loading() {
            false => rsx! {
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
                    button { onclick: move |_| connect_handler(), "Connect" }
                }
            },
            true => rsx! {
                Loader {}
            },
        }
    }
}

#[component]
fn View() -> Element {
    rsx! { "View" }
}

pub const ADD_FIXTURE_MODAL: Symbol = Symbol::create("add-fixture-modal");

const PROJECT_COMMON: Asset = asset!("/assets/project_common.css");
const CONFIGURE: Asset = asset!("/assets/configure.css");
const PROGRAM: Asset = asset!("/assets/program.css");
const SHOW: Asset = asset!("/assets/show.css");
#[component]
fn ProjectLayout() -> Element {
    let mut delay = use_signal(|| 0);
    let mut status_msg = use_signal(|| "Idling".to_string());

    let gen_client = use_service::<GeneralServiceIdent>()?;

    use_resource(move || async move {
        let is_valid = gen_client.read().is_valid_view(SView::Edit).await;
        if !matches!(is_valid, Ok(true)) {
            navigate(Screen::ProjectList);
        }
    })
    .suspend()?;

    use_future(move || async move {
        let mut info_sub = if let Ok(info) = gen_client().info().await {
            info
        } else {
            error!("Failed to recieve info sub!");
            return;
        };

        let mut status_sub = if let Ok(status) = gen_client().status().await {
            status
        } else {
            error!("Failed to receive status sub");
            return;
        };

        loop {
            select! {
                Ok(_) = info_sub.changed() => {
                    let info = info_sub.borrow_and_update().unwrap();
                    match &*info {
                        Info::Autosaved => {
                            ToastInfo::info("Autosaved", "The backend autosaved").post();
                        }
                        Info::Shutdown => {
                            ToastInfo::info("Shutdown", "The backend shutdown!s").post();
                            navigate(Screen::Connect);
                        }
                        Info::Idle => {}
                        Info::Saved => {
                            ToastInfo::info("Saved", "The project was successfully written to disk!").post();
                        }
                        Info::Warning {title, msg} => {
                            ToastInfo::warn(title, msg).post();
                        }
                        Info::ProjectInfo{info: pi } => match pi {
                            ProjectInfo::BlueprintsChanged => {
                                BLUEPRINTS_CHANGED.update();
                            }
                        }
                    }
                }
                Ok(_) = status_sub.changed() => {
                    let status = status_sub.borrow_and_update().unwrap().clone();
                    status_msg.set(status);
                }
            }
        }
    });

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let instant = Instant::now();
            let _ = gen_client.read().alive().await;
            *delay.write() = instant.elapsed().as_millis() as u64;
        }
    });

    let r: Route = use_route();
    let extra_actions = match r {
        Route::Configure {} => rsx! {
            IconButton {
                icon: LdCloudUpload,
                onclick: move |_| async {
                    info!("Open fixture adder");
                    ADD_FIXTURE_MODAL.open().await;
                },
            }
        },
        Route::Program {} => rsx! {},
        Route::Show {} => rsx! {
            IconButton {
                icon: LdTabletSmartphone,
                onclick: move |_| {
                    info!("Open mobile connector");
                },
            }
        },
        _ => {
            warn!("Shouldn't be here");
            rsx! {}
        }
    };

    rsx! {
        div { class: "projectContainer",
            document::Stylesheet { href: PROJECT_COMMON }
            document::Stylesheet { href: CONFIGURE }
            document::Stylesheet { href: PROGRAM }
            document::Stylesheet { href: SHOW }
            nav {
                Branding {}
                div { class: "viewSelect",
                    IconButton {
                        class: if matches!(r, Route::Configure {}) { "curr" },
                        style: "--c-cl: var(--c-p);",
                        icon: LdCog,
                        onclick: move |_| {
                            navigate(Screen::Configure);
                        },
                    }
                    IconButton {
                        class: if matches!(r, Route::Program {}) { "curr" },
                        style: "--c-cl: var(--c-s);",
                        icon: LdPencil,
                        onclick: move |_| {
                            navigate(Screen::Program);
                        },
                    }
                    IconButton {
                        class: if matches!(r, Route::Show {}) { "curr" },
                        style: "--c-cl: var(--c-t);",
                        icon: LdLightbulb,
                        onclick: move |_| {
                            navigate(Screen::Show);
                        },
                    }
                }
                div { class: "actions",
                    {extra_actions}
                    IconButton {
                        icon: LdSave,
                        onclick: move |_| async move {
                            info!("Saving");
                            if let Ok(false) = gen_client.read().save().await {
                                ToastInfo::warn("Wrong save", "Save requested, when no project was loaded!").post();
                            }
                        },
                    }
                }
            }
            Outlet::<Route> {}
            footer { {format!("Ping: {}ms, Status: {}", delay.read(), status_msg.read())} }


        }
    }
}
