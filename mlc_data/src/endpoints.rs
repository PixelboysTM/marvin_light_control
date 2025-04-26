use crate::project::universe::UniverseId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EndpointMapping {
    pub endpoints: HashMap<UniverseId, Vec<EndpointConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EndpointConfig {
    Logger,
    ArtNet,
    Sacn { universe: u16, speed: EndpointSpeed },
    Usb { port: String, speed: EndpointSpeed },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointSpeed {
    Slow,
    Medium,
    Fast,
    Ultra,
    Custom(u64),
}

impl EndpointSpeed {
    pub fn ms(&self) -> u64 {
        match self {
            EndpointSpeed::Slow => 200,
            EndpointSpeed::Medium => 100,
            EndpointSpeed::Fast => 30,
            EndpointSpeed::Ultra => 5,
            EndpointSpeed::Custom(ms) => *ms,
        }
    }
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.ms())
    }
}
