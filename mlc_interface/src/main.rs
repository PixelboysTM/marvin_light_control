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
// const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    // dioxus::launch(App);
    dioxus::LaunchBuilder::new()
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
                    if let Ok(_) = client.alive().await {
                        *CONNECT_URL.write() = (addr, port);
                        ToastInfo::info(
                            "Connection successful",
                            "Connection to the backend could be established.",
                        )
                            .post();
                        let nav = navigator();
                        nav.replace("/projects");
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
// #[component]
// pub fn Hero() -> Element {
//     rsx! {
//         div { id: "hero",
//             img { src: HEADER_SVG, id: "header" }
//             div { id: "links",
//                 a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
//                 a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
//                 a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
//                 a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
//                 a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus",
//                     "💫 VSCode Extension"
//                 }
//                 a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
//             }
//         }
//     }
// }

// /// Home page
// #[component]
// fn Home() -> Element {
//     // let c: Coroutine<String> = connect("http://localhost:8000/api/test", Callback::new(|rx: String| {
//     //     warn!("Got response: {}", rx);
//     // }));
//     let e = use_resource(async || {
//         let socket = TcpStream::connect((Ipv4Addr::LOCALHOST, 8181))
//             .await
//             .unwrap();

//         let (socket_rx, mut socket_tx) = socket.into_split();
//         socket_tx
//             .write_all(&mlc_communication::ECHO_SERVICE_IDENT)
//             .await
//             .unwrap();

//         let client: EchoServiceClient<remoc::codec::Bincode> =
//             remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
//                 .consume()
//                 .await
//                 .unwrap();

//         client
//     });

//     let a = use_resource(async || {
//         let socket = TcpStream::connect((Ipv4Addr::LOCALHOST, 8181))
//             .await
//             .unwrap();

//         let (socket_rx, mut socket_tx) = socket.into_split();
//         socket_tx
//             .write_all(&mlc_communication::ANOTHER_SERVICE_IDENT)
//             .await
//             .unwrap();

//         let client: AnotherServiceClient<remoc::codec::Bincode> =
//             remoc::Connect::io(remoc::Cfg::default(), socket_rx, socket_tx)
//                 .consume()
//                 .await
//                 .unwrap();

//         client
//     });
//     rsx! {

//         Hero {}
//         button {
//             onclick: move |_| async move {
//                 warn!("Got button");
//                 match &*e.read() {
//                     Some(s) => {
//                         let res = s.echo("Hello".to_string()).await;
//                         info!("Got a result: {:?}", res);
//                     }
//                     None => {
//                         warn!("Service not loaded");
//                     }
//                 }
//             },
//             {"Send".to_string()}
//         }
//         button {
//             onclick: move |_| async move {
//                 warn!("Got button");
//                 match &*a.read() {
//                     Some(s) => {
//                         s.hello().await.unwrap();
//                     }
//                     None => {
//                         warn!("Service not loaded");
//                     }
//                 }
//             },
//             {"Hello".to_string()}
//         }
//     }
// }

// /// Blog page
// #[component]
// pub fn Blog(id: i32) -> Element {
//     rsx! {
//         div { id: "blog",

//             // Content
//             h1 { "This is blog #{id}!" }
//             p {
//                 "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components."
//             }

//             // Navigation links
//             Link { to: Route::Blog { id: id - 1 }, "Previous" }
//             span { " <---> " }
//             Link { to: Route::Blog { id: id + 1 }, "Next" }
//         }
//     }
// }

// /// Shared navbar component.
// #[component]
// fn Navbar() -> Element {
//     rsx! {
//         div { id: "navbar",
//             Link { to: Route::Home {}, "Home" }
//             Link { to: Route::Blog { id: 1 }, "Blog" }
//         }

//         Outlet::<Route> {}
//     }
// }
