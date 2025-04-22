use crate::connect::{use_service, RtcSuspend, SClient};
use crate::toaster::ToastInfo;
use crate::utils::{
    some_recv, Fader, IconButton, MappedVecTabs, Modal, ModalResult, ModalVariant, Orientation,
    Panel, SignalNotify, Symbol, TabController, TabItem, Tabs,
};
use crate::ADD_FIXTURE_MODAL;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdLamp, LdPencil, LdPencilRuler, LdRoute, LdSearch};
use futures::StreamExt;
use itertools::Itertools;
use mlc_communication::services::project::{ProjectService, ProjectServiceIdent};
use mlc_data::fixture::blueprint::{Channel, FixtureBlueprint};
use mlc_data::misc::ErrIgnore;
use mlc_data::project::universe::{UniverseAddress, UniverseId, UNIVERSE_SIZE};
use tokio::select;

pub static BLUEPRINTS_CHANGED: SignalNotify = SignalNotify::create();

#[component]
pub fn Configure() -> Element {
    let prj_service = use_service::<ProjectServiceIdent>()?;

    let mut import_selection = use_signal(Vec::new);
    rsx! {
        div { class: "configure",
            Panel { column: "1 / 4", row: "1 / 9", title: "Fixture Catalog", FixtureCatalog {prj_service} }
            Panel { column: "1 / 13", row: "9 / 13", FaderPanel {prj: prj_service} }
            Panel { column: "10 / 13", row: "1 / 9", title: "Settings", Settings {prj: prj_service} }
            Panel { column: "4 / 10", row: "1 / 9", title: "Universe Patcher" }
        }

        Modal {
                title: "Fixture Explorer",
                ident: ADD_FIXTURE_MODAL,
                variant: ModalVariant::OkCancel,
                icon: LdLamp,
                oktext: "Import",
                onexit: move |a| async move{
                    if a == ModalResult::Success {
                        if let Err(e) = prj_service.read().import_fixture_blueprints(import_selection.read().clone()).await {
                            ToastInfo::error("Failed to import blueprints!", e.to_string()).post();
                        }
                    }

                    import_selection.write().clear();
                },
                AddFixtureBlueprintModal { project: prj_service, selected: import_selection }
            }
    }
}

const BLUEPRINT_DETAILS: Symbol = Symbol::create("blueprint-inspect");

#[derive(Debug, Copy, Clone, PartialEq)]
enum BlueprintDetailsMode {
    Inspect,
    Patch,
}

impl TabController for BlueprintDetailsMode {
    type Item = Self;

    fn get_options(&self) -> Vec<Self::Item> {
        vec![BlueprintDetailsMode::Inspect, BlueprintDetailsMode::Patch]
    }

    fn set(&mut self, option: Self::Item) {
        *self = option;
    }

    fn get(&self) -> Self::Item {
        *self
    }
}

impl TabItem for BlueprintDetailsMode {
    fn get_name(&self) -> String {
        match self {
            BlueprintDetailsMode::Inspect => "Inspect",
            BlueprintDetailsMode::Patch => "Patch",
        }
        .to_string()
    }
}

#[component]
fn FixtureCatalog(prj_service: SClient<ProjectServiceIdent>) -> Element {
    let mut blueprints_control =
        use_resource(move || async move { prj_service.read().list_blueprints().await });
    let blueprints = blueprints_control.rtc_suspend()?;

    use_effect(move || {
        let _ = BLUEPRINTS_CHANGED.read();
        if *blueprints_control.state().read() != UseResourceState::Pending {
            blueprints_control.restart();
        }
    });

    let details_mode = use_signal(|| BlueprintDetailsMode::Inspect);
    let mut detailed_blueprint_indent = use_signal(String::new);

    rsx! {
        div {
            class: "fixtureCatalog",
            for (i1, i2, b) in blueprints().iter().map(|b| (b.meta.identifier.clone(), b.meta.identifier.clone(), b)) {
                div {
                    class: "blueprint",
                    ondoubleclick: move |_| {
                        let i = i1.clone();
                        async move {
                        *detailed_blueprint_indent.write() = i;
                        BLUEPRINT_DETAILS.open().await;
                    }},
                    h1 {
                        {b.meta.name.clone()}
                    }
                    code {
                        {b.meta.identifier.clone()}
                    }
                    ul {
                        for m in b.modes.iter() {
                            li {
                                {m.name.clone()}
                            }
                        }
                    }

                    IconButton {
                        icon: LdPencil,
                        class: "inspect",
                        onclick: move |_| {
                            let i = i2.clone();
                            async move {
                                *detailed_blueprint_indent.write() = i;
                                BLUEPRINT_DETAILS.open().await;
                            }
                        }
                    }
                }
            }
        }

        Modal {
            title: match details_mode() {
                BlueprintDetailsMode::Inspect => {"Inspect Fixture Blueprint"}
                BlueprintDetailsMode::Patch => {"Patch Fixture Blueprint"}
            },
            ident: BLUEPRINT_DETAILS,
            variant: match details_mode() {
                BlueprintDetailsMode::Inspect => {ModalVariant::Ok}
                BlueprintDetailsMode::Patch => {ModalVariant::OkCancel}
            },
            icon: LdPencilRuler,
            oktext: match details_mode() {
                BlueprintDetailsMode::Inspect => {"Close"}
                BlueprintDetailsMode::Patch => {"Patch"}
            },
            BlueprintDetailsModal {
                blueprints,
                mode: details_mode,
                ident: detailed_blueprint_indent,
            }
        }
    }
}

#[component]
fn BlueprintDetailsModal(
    blueprints: MappedSignal<Vec<FixtureBlueprint>>,
    mode: Signal<BlueprintDetailsMode>,
    ident: Signal<String>,
) -> Element {
    if ident.read().is_empty() {
        return rsx! {};
    }

    let ref_bp = blueprints.map(move |bs| {
        bs.iter()
            .find(|b| b.meta.identifier == *ident.read())
            .expect("Blueprint not found!")
    });
    let r2 = ref_bp.clone();

    let mut sel_mode = use_signal(|| 0);
    let ref_mode = use_memo(move || r2.read().modes.get(*sel_mode.read()).cloned());

    use_effect(move || {
        let _ = ident.read();
        sel_mode.set(0);
    });

    rsx! {
        Tabs {
            controller: mode,
            orientation: Orientation::Horizontal
        },

        match *mode.read() {
            BlueprintDetailsMode::Inspect => {
                rsx!{
                    h1 {{ref_bp.read().meta.name.clone()}}
                    code {{ref_bp.read().meta.identifier.clone()}}
                    div {
                        class: "modes",
                        div {
                            class: "list",
                            for (i, m) in ref_bp.read().modes.iter().enumerate() {
                                button {
                                    class: if *sel_mode.read() == i {"selected"},
                                    onclick: move |e| {
                                        sel_mode.set(i);
                                        e.prevent_default();
                                    },
                                    {m.name.clone()}
                                }
                            }
                        },
                        div {
                            class: "mode",
                            if let Some(m) = &*ref_mode.read() {
                                h1 {"Channels:"},
                                ol {
                                    for channel in m.channels.iter() {
                                        li {
                                            match channel {
                                                None => rsx!{p { "Unused/Blank Channel"}},
                                                Some(channel) => {
                                                    let r = ref_bp.read();
                                                    if let Some((i, c, common)) = r.channels.iter().find(|c| c.0 == channel).map(|(i, c)| (i, c, match c {Channel::Single {channel} => channel, Channel::Double {channel, ..} => channel, Channel::Tripple {channel, ..} => channel} )) {
                                                        rsx!{
                                                            details {
                                                                "name": "channel",
                                                                summary {
                                                                    {i.clone()}
                                                                },
                                                                match c {
                                                                    Channel::Single {..} => rsx!{p {
                                                                            class: "granularity",
                                                                            {"Granularity: 8bit"}
                                                                        }},
                                                                    Channel::Double {second_channel_name, ..}=> {
                                                                        rsx!{p {
                                                                            class: "granularity",
                                                                            {format!("Granularity: 16bit, Aliases: {}", second_channel_name)}
                                                                        }}
                                                                    }
                                                                    Channel::Tripple {second_channel_name, third_channel_name, ..}=> {
                                                                        rsx!{p {
                                                                            class: "granularity",
                                                                            {format!("Granularity: 24bit, Aliases: {} {}", second_channel_name, third_channel_name)}
                                                                        }}
                                                                    }
                                                                }
                                                                p {{format!("Default Value: {:.2}%", common.default_value.take() * 100.0)}}
                                                            }
                                                        }
                                                    } else if let Some(gran) = r.channels.iter().fold( None, |v, (_, c)| v.or( match c {
                                                        Channel::Single { .. } => None,
                                                        Channel::Double {second_channel_name, ..} => if channel == second_channel_name {Some(1)} else {None}
                                                        Channel::Tripple {second_channel_name, third_channel_name, ..} => if channel == second_channel_name {Some(1)} else if channel == third_channel_name {Some(2)} else {None}
                                                    })) {
                                                        rsx! {
                                                            details {
                                                                "name": "channel",
                                                                summary {
                                                                    {channel.clone()}
                                                                },
                                                                p {
                                                                    {format!("Granularity: {}", gran)}
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        rsx!{
                                                            p {"Could not find more information"}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            BlueprintDetailsMode::Patch => {rsx!{}}
        }
    }
}

#[component]
fn AddFixtureBlueprintModal(
    project: SClient<ProjectServiceIdent>,
    mut selected: Signal<Vec<String>>,
) -> Element {
    let available_projects =
        use_resource(
            move || async move { project.read().list_available_fixture_blueprints().await },
        )
        .rtc_suspend()?;

    let mut search = use_signal(|| "".to_string());

    let filtered_projects = use_memo(move || {
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        available_projects
            .read()
            .iter()
            .filter(|p| {
                if search.read().is_empty() {
                    true
                } else {
                    matcher
                        .fuzzy(&p.meta.identifier, &search.read().replace(' ', ""), false)
                        .map(|(s, _)| s > 0)
                        .unwrap_or(false)
                }
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    rsx! {
        input {
            value: search(),
            oninput: move |v| {
                search.set(v.value());
            },
        }
        div { class: "list",
            for p in filtered_projects() {
                div { class: "blueprint",
                    input {
                        class: "import",
                        r#type: "checkbox",
                        checked: selected().contains(&p.meta.identifier),
                        onchange: move |v| {
                            let mut sel = selected.write();
                            if v.checked() {
                                *sel = sel.iter().chain(Some(&p.meta.identifier)).unique().cloned().collect();
                            } else {
                                *sel = sel.iter().filter(|e| *e != &p.meta.identifier).unique().cloned().collect();
                            }
                        }
                    }

                    h1 { {p.meta.name.clone()} }
                    code { {p.meta.identifier.clone()} }
                    details {
                        "name": "bps",
                        summary {
                            "More"
                        },
                        p {
                            class: "modes",
                            "Modes: ",
                            for m in p.modes {
                                "'", span {{m},}, "', "
                            }
                        }
                        p {
                            class: "numChannels",
                            "NumChannels: ",
                            {p.num_channels.to_string()}
                        }
                    }
                }
            }
        }
    }
}

enum Fcc {
    SwitchUniverse(UniverseId),
    SetValue { adds: UniverseAddress, value: u8 },
}

pub const UNIVERSE_LIST_CHANGED: SignalNotify = SignalNotify::create();

#[component]
fn FaderPanel(prj: SClient<ProjectServiceIdent>) -> Element {
    let ul = use_resource(move || async move { prj.read().universe_list().await }).rtc_suspend()?;

    let tabs = use_signal(move || MappedVecTabs::new(ul, 1));

    let mut data = use_signal(|| [0_u8; UNIVERSE_SIZE]);

    let value_setter = use_coroutine(move |mut rx: UnboundedReceiver<Fcc>| async move {
        let mut recv = None;
        let mut send = None;
        loop {
            select! {
                Some(d) = rx.next() => {
                    match d {
                        Fcc::SwitchUniverse(u) => {
                            if let Ok(x) = prj.read().universe_sub(u).await {
                                recv = Some(x.0);
                                send = Some(x.1);
                            }
                        }
                        Fcc::SetValue{adds, value: v } => {
                            if let Some(x) = &send {
                                x.send((adds, v)).await.debug_ignore();
                            }
                        }
                    }
                }
                Ok(r) = some_recv(recv.as_mut()) => {
                    if let Some(u) = r {
                        data.write()[u.0.take() - 1] = u.1;
                    }
                }
            }
        }
    });

    use_effect(move || {
        let u = tabs.read().get();
        value_setter.send(Fcc::SwitchUniverse(u));
    });

    rsx! {
        div { class: "faderContainer",
            Tabs {
                controller: tabs,
                orientation: Orientation::Vertical,
            }
            div {
                class: "faders",
                for i in 0..UNIVERSE_SIZE {
                    div {
                        class: "fader-c",
                        span {
                            class: "value",
                            {format!("{:0>3}",data.read()[i].clone())}
                        }
                        Fader {
                            value: data.map(move |d| &d[i]),
                            update: move |d| {value_setter.send(Fcc::SetValue {
                                adds: UniverseAddress::create(i + 1),
                                value: d,
                            })},
                        }
                        span {
                            class: "adds",
                            {(i + 1).to_string()}
                        }
                    }
                }
            }
        }
    }
}

const ENDPOINT_MAPPING_MODAL: Symbol = Symbol::create("endpoint-mapping");

#[component]
fn Settings(prj: SClient<ProjectServiceIdent>) -> Element {
    rsx! {
        div {
            class: "settingsContainer",
            h3 {
                "Title:"
            }
            p {
                "Test Project"
            }

            h3 {
                "Filename:"
            }

            code {
                "test_project.mlc"
            }

            h3 {
                "Format"
            }

            p {
                "Binary"
            }

            h3 {
                "Save on quit"
            }

            input {
                r#type: "checkbox",
            }

            h3 {
                "Autosave"
            }

            input {
                r#type: "checkbox",
            }

            h3 {
                "Interval"
            }

            p {
                "00:30"
            }

            div {
                class: "buttons",

                IconButton {
                    icon: LdRoute,
                    text: "Endpoint Mappings",
                    onclick: move |_| async move {
                        ENDPOINT_MAPPING_MODAL.open().await;
                    }
                }
            }
        }

        Modal {
            title: "Endpoint Mappings",
            ident: ENDPOINT_MAPPING_MODAL,
            variant: ModalVariant::OkCancel,
            icon: LdRoute,
            oktext: "Save",
            onexit: move |a| async move{
                if a == ModalResult::Success {
                    // if let Err(e) = prj.read().save_endpoint_mappings().await {
                    //     ToastInfo::error("Failed to save endpoint mappings!", e.to_string()).post();
                    // }
                }
            }
        }
    }
}
