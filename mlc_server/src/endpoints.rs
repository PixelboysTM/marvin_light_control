use crate::endpoints::driver::LogDriver;
use crate::misc::{AdaptNotifier, AdaptScopes, ShutdownHandler, ShutdownPhase};
use crate::universe::UniverseUpdateSubscriber;
use crate::{AServiceImpl, MlcServiceResources, MlcServiceSimple};
use mlc_data::endpoints::{EndpointConfig, EndpointMapping};
use tokio::select;

mod driver;

pub struct EndpointsManagerService;

impl MlcServiceSimple for EndpointsManagerService {
    fn start(self, res: &MlcServiceResources) -> impl Future<Output = ()> + Send + 'static {
        start_endpoint_manager(
            res.service_obj.clone(),
            res.shutdown.clone(),
            res.adapt_notifier.clone(),
        )
    }
}

async fn start_endpoint_manager(
    service_obj: AServiceImpl,
    shutdown_handler: ShutdownHandler,
    adapt_notifier: AdaptNotifier,
) {
    let mut mapping = service_obj.project.read().await.endpoint_mapping.clone();

    let mut drivers = DriverCollection {
        log: LogDriver::new(),
        artnet: LogDriver::new(),
        sacn: LogDriver::new(),
        usb: LogDriver::new(),
    };

    loop {
        adapt_endpoints(service_obj.clone(), &mapping, &mut drivers).await;
        select! {
            _ = shutdown_handler.wait(ShutdownPhase::Phase1) => {
                break;
            }
            _ = adapt_notifier.wait(AdaptScopes::ENDPOINTS) => {
                mapping = service_obj.project.read().await.endpoint_mapping.clone();
            }
        }
    }

    drivers.stop_all().await;
}

struct DriverCollection {
    log: LogDriver,
    artnet: LogDriver,
    sacn: LogDriver,
    usb: LogDriver,
}

impl DriverCollection {
    async fn stop_all(&mut self) {
        self.log.stop_all().await;
        self.artnet.stop_all().await;
        self.sacn.stop_all().await;
        self.usb.stop_all().await;
    }

    async fn apply_config(&mut self, sub: UniverseUpdateSubscriber, config: &EndpointConfig) {
        match config {
            EndpointConfig::Logger => {
                self.log.apply_config(sub, ()).await;
            }
            EndpointConfig::ArtNet => {
                self.artnet.apply_config(sub, ()).await;
            }
            EndpointConfig::Sacn { .. } => {
                self.sacn.apply_config(sub, ()).await;
            }
            EndpointConfig::Usb { .. } => {
                self.usb.apply_config(sub, ()).await;
            }
        }
    }
}

async fn adapt_endpoints(
    service_obj: AServiceImpl,
    mapping: &EndpointMapping,
    drivers: &mut DriverCollection,
) {
    drivers.stop_all().await;

    for (universe, configs) in &mapping.endpoints {
        for config in configs {
            drivers
                .apply_config(
                    service_obj.universe_runtime.subscribe_universe(*universe),
                    config,
                )
                .await;
        }
    }
}

trait EndpointDriver<C> {
    async fn stop_all(&mut self);
    async fn apply_config(&mut self, sub: UniverseUpdateSubscriber, config: C);
}
