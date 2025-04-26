use crate::misc::{AdaptNotifier, AdaptScopes, ShutdownHandler, ShutdownPhase};
use crate::project::Project;
use crate::{AServiceImpl, MlcService, MlcServiceResources, MlcServiceSimple};
use mlc_communication::services::general::Info;
use std::pin::Pin;
use tokio::select;
use tracing::{error, info};

pub struct ShutdownService;

impl MlcServiceSimple for ShutdownService {
    fn start(res: &MlcServiceResources) -> impl Future<Output = ()> + Send + 'static {
        create_shutdown_handler(res.service_obj.clone(), res.shutdown.clone())
    }
}

async fn create_shutdown_handler(obj: AServiceImpl, shutdown: ShutdownHandler) {
    shutdown.wait(ShutdownPhase::Phase1).await;
    obj.send_info(Info::Shutdown);

    let mut p = obj.project.write().await;
    if obj.project_valid().await && p.settings.save_on_quit {
        p.save().await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    shutdown.advance().await;
    shutdown.advance().await;
}

pub struct AutosaveService;

impl MlcServiceSimple for AutosaveService {
    fn start(res: &MlcServiceResources) -> impl Future<Output = ()> + Send + 'static {
        autosave_service(
            res.service_obj.clone(),
            res.adapt_notifier.clone(),
            res.shutdown.clone(),
        )
    }
}

async fn autosave_service(
    service_obj: AServiceImpl,
    adapt_notifier: AdaptNotifier,
    shutdown: ShutdownHandler,
) {
    fn save_fut(p: &Project, valid: bool) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        if valid {
            if let Some(d) = &p.settings.autosave {
                Box::pin(tokio::time::sleep(*d))
            } else {
                Box::pin(futures::future::pending())
            }
        } else {
            Box::pin(futures::future::pending())
        }
    }

    loop {
        let duration = save_fut(
            &*service_obj.project.read().await,
            *service_obj.valid_project.read().await,
        );

        select! {
            _ = adapt_notifier.wait(AdaptScopes::SETTINGS) => {
                info!("Adapting autosave interval!");
                continue;
            }
            _ = shutdown.wait(ShutdownPhase::Phase1) => {
                break;
            }
            _ = duration => {
                info!("Autosave triggered!");
                let _ = service_obj.project.write().await.save().await.map_err(|e| error!("{e:?}"));
                service_obj.send_info(Info::Autosaved);
            }
        }

        tokio::task::yield_now().await;
    }
}
