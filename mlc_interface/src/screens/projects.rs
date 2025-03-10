use chrono::Local;
use dioxus::logger::tracing::info;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdFileArchive, LdFileJson, LdPen, LdPlus, LdTrash};
use mlc_communication::services::general::{GeneralService, GeneralServiceIdent, View as GenView};
use mlc_data::project::{ProjectMetadata, ProjectType, ToFileName};
use uuid::Uuid;
use crate::connect::connect;
use crate::utils::{Branding, IconButton, Loader, Modal, ModalVariant, Symbol};

const PROJECTS_CSS: Asset = asset!("/assets/projects.css");

const CREATE_PROJECT: Symbol = Symbol::create("create-project");
#[component]
pub fn Project() -> Element {
    let _client = use_resource(async || {
        let res = connect::<GeneralServiceIdent>().await;
        if res.is_err() {
            navigator().replace("/");
        }

        let c = res.expect("Must be");
        if let Ok(false) = c.is_valid_view(GenView::Project).await {
            navigator().replace("/project/configure");
        }
        c
    })
        .suspend()?;

    let mut new_project_name = use_signal(|| "New Project".to_string());
    let mut new_project_type = use_signal(|| ProjectType::Json);
    let is_json = use_memo(move || new_project_type.read().eq(&ProjectType::Json));

    let file_name = use_memo(move || {
        format!(
            "{}.{}",
            new_project_name.read().to_project_file_name(),
            new_project_type.read().extension()
        )
    });

    rsx! {
        document::Stylesheet { href: PROJECTS_CSS }
        div { class: "projectsPage",
            nav {
                Branding {}
                h1 { "Project Explorer" }
                div { class: "actions",
                    IconButton {
                        icon: LdPlus,
                        onclick: async |_| {
                            info!("Opening");
                            CREATE_PROJECT.open().await;
                        },
                    }
                }
            }

            SuspenseBoundary {
                fallback: move |_| {
                    rsx!{
                        Loader{}
                    }
                },
                ProjectList {}
            }

            Modal {
                title: "Create Project",
                ident: CREATE_PROJECT.clone(),
                icon: LdPlus,
                variant: ModalVariant::OkCancel,
                onexit: move |_| {},
                oktext: "Create".to_string(),
                label {
                    "Project Name: "
                    input {
                        r#type: "text",
                        value: new_project_name().clone(),
                        oninput: move |v| *new_project_name.write() = v.value(),
                    }
                }
                p {
                    class: "fileName",
                    span {
                        "File Name: "
                    }
                    span {
                        class: "value",
                        {file_name()}
                    }
                }
                label {
                    "Binary: "
                    input {
                        r#type: "checkbox",
                        value: !is_json(),
                        onchange: move |v| {
                            *new_project_type.write() = if v.value() == "true" {
                                ProjectType::Binary
                            } else {
                                ProjectType::Json
                            };
                        },
                    }
                }
            }
        }
    }
}

fn gen_projects(i: usize) -> Vec<ProjectMetadata> {
    (0..i).map(|i| {
        let name = format!("Project {}", i);
        ProjectMetadata {
            name: name.clone(),
            file_name: name.to_project_file_name(),
            project_type: if i % 2 == 0 {ProjectType::Binary} else { ProjectType::Json },
            last_saved: Local::now(),
            id: Uuid::nil(),
        }
    }).collect()
}

#[component]
fn ProjectList() -> Element {
    let projects = use_resource(async ||{
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        gen_projects(30)
    }).suspend()?;

    rsx!{
        div {
            class: "projectList",
            for p in projects().iter() {
                div {
                    class: "project",
                    ondoubleclick: move |_| {
                        info!("Double clicked");
                    },

                    div {
                        class: "info",
                        h1 { {p.name.clone()} }
                        p {
                            class: "fileName",
                            {p.file_name.clone()}
                        }
                        p {
                            class: "lastSaved",
                            {p.last_saved.format("%d.%m.%y %H:%M").to_string()}
                        }

                        match p.project_type {
                            ProjectType::Json => rsx!{Icon{icon: LdFileJson, class: "fileType"}},
                            ProjectType::Binary => rsx!{Icon{icon: LdFileArchive, class: "fileType"}},
                        }
                    }

                    div {
                        class: "actions",
                        IconButton {
                            icon: LdPen,
                            onclick: async |_| {
                                info!("Opening");
                            },
                        }
                        IconButton {
                            icon: LdTrash,
                            class: "delete",
                            onclick: async |_| {
                                info!("Deleting");
                            }
                        }
                    }
                }
            }
        }
    }
}
