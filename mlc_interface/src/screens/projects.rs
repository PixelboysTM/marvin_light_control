use crate::connect::connect;
use crate::toaster::ToastInfo;
use crate::utils::{navigate, Branding, IconButton, Loader, Modal, ModalResult, ModalVariant, Screen, Symbol};
use dioxus::logger::tracing::{info, warn};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdFileArchive, LdFileJson, LdLightbulb, LdPen, LdPencilRuler, LdPlus, LdSave, LdTrash, LdTriangleAlert};
use dioxus_free_icons::Icon;
use log::error;
use mlc_communication::services::general::{
    GeneralService, GeneralServiceIdent, Info, View as GenView,
};
use mlc_data::project::{ProjectMetadata, ProjectType, ToFileName};
use mlc_communication::services::project_selection::{ProjectIdent, ProjectSelectionService, ProjectSelectionServiceClient, ProjectSelectionServiceIdent};

const PROJECTS_CSS: Asset = asset!("/assets/projects.css");

const CREATE_PROJECT: Symbol = Symbol::create("create-project");
#[component]
pub fn Project() -> Element {
    let m_client = use_resource(async || {
        let res = connect::<GeneralServiceIdent>().await;
        if res.is_err() {
            navigate(Screen::Connect)
        }

        error!("GOT HERE");

        let c = res.expect("Must be");
        if let Ok(false) = c.is_valid_view(GenView::Project).await {
            // navigator().replace("/project/configure");
            navigate(Screen::Configure)
        }

        c
    })
    .suspend()?;

    use_future(move || {
        let value = m_client.clone();

        async move {
            let client = value.read();
            let mut info = client.info().await.expect("Why not?");

            loop {
                info.changed().await.expect("Failed");

                let i = info.borrow_and_update().expect("Failed");

                match *i {
                    Info::Idle => {
                        warn!("Info Idle")
                    }
                    Info::Shutdown => {
                        ToastInfo::info("Shutdown", "The backend shutdown!").post();
                        navigator().replace("/");
                    }
                    Info::Autosaved => {
                        ToastInfo::info("Autosaved", "The backend autosaved").post();
                    }
                }
            }
        }
    });

    let service = use_resource::<ProjectSelectionServiceClient, _>(async || connect::<ProjectSelectionServiceIdent>().await.expect("Handling of connection loss not yet implemented"));

    let service_suspend = service.suspend()?;
    let s2 = service_suspend.clone();
    let s3 = service_suspend.clone();

    let mut projects = use_resource::<Vec<ProjectMetadata>, _>(move || {
        let s2 = s2.clone();
        async move {
            let ps = s2.read().list().await.expect("Couldn't get projects");
            println!("{:#?}", ps.iter().map(|p| &p.file_name).collect::<Vec<_>>());
            ps
        }
    });

    let projects_suspend = projects.suspend()?;
    let p2 = projects_suspend.clone();

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
                    rsx! {
                        Loader {}
                    }
                },
                ProjectList {
                    projects: projects_suspend,
                    onopen: move |ident| {
                        let s3 = s3.clone();
                        async move {
                            info!("Opening project: {:?}", ident);
                            let r = s3.read().open(ident).await;
                            match r {
                                Ok(true) => {
                                    ToastInfo::info("Loaded Project", "Project loading successful").post();
                                    navigate(Screen::Configure);
                                }
                                Ok(false) => {
                                    ToastInfo::info("Project not found", "Requested project could not be opened because it could not be located on disk!").post();
                                    projects.restart();
                                }
                                Err(e) => {
                                    ToastInfo::error("Failed to open project", e.to_string()).post();
                                    projects.restart();
                                }
                            }
                        }
                    }
                }
            }

            Modal {
                title: "Create Project",
                ident: CREATE_PROJECT.clone(),
                icon: LdPlus,
                variant: ModalVariant::OkCancel,
                onexit: move |r| {
                    let s2 = service_suspend.clone();
                    async move {
                    if r == ModalResult::Cancel {
                        return;
                    }

                    let id = s2.read().create(new_project_name.read().clone(), new_project_type.read().clone()).await.expect("Couldn't create project");
                        let r = s2.read().open(id).await.expect("Couldn't open project");
                        if r {
                            navigate(Screen::Configure);
                        } else {
                            ToastInfo::warn("Project not found", "The given project could not be located!").post();
                        }
                }},

                oktext: "Create".to_string(),
                label {
                    "Project Name: "
                    input {
                        r#type: "text",
                        value: new_project_name().clone(),
                        oninput: move |v| *new_project_name.write() = v.value(),
                    }
                }
                p { class: "fileName",
                    span { "File Name: " }
                    span { class: "value", {file_name()} }
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

#[component]
fn ProjectList(projects: MappedSignal<Vec<ProjectMetadata>>, onopen: EventHandler<ProjectIdent>) -> Element {
    rsx! {
        div { class: "projectList",
            for p in projects().into_iter() {
                ProjectListItem {
                    item: p.clone(),
                    onopen: move |_| {
                        onopen.call(p.file_name.clone());
                    }
                }
            }
        }
    }
}

#[component]
fn ProjectListItem(item: ProjectMetadata, onopen: EventHandler) -> Element {
    rsx!{
        div {
            class: "project",
            ondoubleclick: move |_| {
                onopen.call(());
            },

            div { class: "info",
                h1 { {item.name} }
                p { class: "fileName", {item.file_name} }

                div { class: "details",
                    p { class: "createdAt",
                        Icon { icon: LdPencilRuler }
                        {item.created_at.format("%d.%m.%y %H:%M").to_string()}
                    }
                    p { class: "lastSaved",
                        Icon { icon: LdSave }
                        {item.last_saved.format("%d.%m.%y %H:%M").to_string()}
                    }
                    p { class: "fixtureCount",
                        Icon { icon: LdLightbulb }
                        "Fixtures"
                    }
                }

                match item.project_type {
                    ProjectType::Json => rsx! {
                        Icon { icon: LdFileJson, class: "fileType" }
                    },
                    ProjectType::Binary => rsx! {
                        Icon { icon: LdFileArchive, class: "fileType" }
                    },
                    ProjectType::Invalid => rsx! {
                        Icon { icon: LdTriangleAlert, class: "fileType" }}
                    }
            }

            div { class: "actions",
                IconButton {
                    icon: LdPen,
                    onclick: move |_| {
                        onopen.call(());
                    },
                }
                IconButton {
                    icon: LdTrash,
                    class: "delete",
                    onclick: async |_| {
                        info!("Deleting");
                    },
                }
            }
        }
    }
}
