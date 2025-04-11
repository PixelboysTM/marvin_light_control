use crate::connect::{use_service, RtcSuspend, SClient};
use crate::toaster::ToastInfo;
use crate::utils::{Modal, ModalResult, ModalVariant, Panel, SignalNotify};
use crate::ADD_FIXTURE_MODAL;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdLamp;
use itertools::Itertools;
use mlc_communication::services::project::{ProjectService, ProjectServiceIdent};

pub static BLUEPRINTS_CHANGED: SignalNotify = SignalNotify::create();

#[component]
pub fn Configure() -> Element {
    let prj_service = use_service::<ProjectServiceIdent>()?;

    let mut import_selection = use_signal(Vec::new);
    rsx! {
        div { class: "configure",
            Panel { column: "1 / 4", row: "1 / 9", title: "Fixture Catalog", FixtureCatalog {prj_service} }
            Panel { column: "1 / 13", row: "9 / 13", title: "Fader Panel" }
            Panel { column: "10 / 13", row: "1 / 9", title: "Settings" }
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

    rsx! {
        div {
            class: "fictureCatalog",
            for b in blueprints().iter() {
                div {
                    class: "blueprint",
                    h1 {
                        {b.meta.name.clone()}
                    }
                }
            }
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
