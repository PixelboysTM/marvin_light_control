use std::time::{Duration, SystemTime};

use dioxus::{logger::tracing::Level, prelude::*};

const TOASTER_DATA: GlobalSignal<Vec<(ToastInfo, Option<SystemTime>)>> = Signal::global(|| {
    vec![
    //     (
    //     ToastInfo {
    //         level: Level::DEBUG,
    //         msg: "This is an example".to_string(),
    //         title: "Moin Meister!".to_string(),
    //     },
    //     None,
    // )
    ]
});

pub struct ToastInfo {
    level: Level,
    title: String,
    msg: String,
}

#[allow(dead_code)]
impl ToastInfo {
    pub fn info(title: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            level: Level::INFO,
            title: title.into(),
            msg: msg.into(),
        }
    }
    pub fn trace(title: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            level: Level::TRACE,
            title: title.into(),
            msg: msg.into(),
        }
    }
    pub fn debug(title: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            level: Level::DEBUG,
            title: title.into(),
            msg: msg.into(),
        }
    }
    pub fn warn(title: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            level: Level::WARN,
            title: title.into(),
            msg: msg.into(),
        }
    }

    pub fn error(title: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            level: Level::ERROR,
            title: title.into(),
            msg: msg.into(),
        }
    }

    pub fn post(self) {
        let mut w = TOASTER_DATA.write();
        w.push((self, Some(SystemTime::now() + Duration::from_secs(5))));
    }
}

const TOASTER_CSS: Asset = asset!(
    "/assets/toaster.css",
    CssAssetOptions::new().with_preload(true)
);

#[component]
pub fn ToasterProvider() -> Element {
    use_future(async || loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut w = TOASTER_DATA.write();
        let now = SystemTime::now();
        w.retain(|(_, t)| t.is_none() || t.as_ref().expect("") > &now);
    });
    rsx! {
        document::Stylesheet { href: TOASTER_CSS }
        div { class: "toasterProvider",
            for (toast , _) in TOASTER_DATA.read().iter() {
                div {
                    class: "toast",
                    class: if toast.level == Level::INFO { "info" },
                    class: if toast.level == Level::DEBUG { "debug" },
                    class: if toast.level == Level::TRACE { "trace" },
                    class: if toast.level == Level::WARN { "warn" },
                    class: if toast.level == Level::ERROR { "error" },
                    div { class: "title",
                        div { class: "dot" }
                        span { {toast.title.clone()} }
                    }
                    span { class: "msg", {toast.msg.clone()} }
                }
            }
        }
    }
}
