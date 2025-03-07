use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureBlueprint {
    #[serde(flatten)]
    meta: Metadata,
    channels: HashMap<ChannelIdentifier, Channel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    identifier: String,
    manufacturer: String,
}

pub type ChannelIdentifier = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Channel {
    Single {
        #[serde(flatten)]
        channel: CommonChannel,
    },
    Double {
        #[serde(flatten)]
        channel: CommonChannel,
        second_channel_name: ChannelIdentifier,
    },
    Tripple {
        #[serde(flatten)]
        channel: CommonChannel,
        second_channel_name: ChannelIdentifier,
        third_channel_name: ChannelIdentifier,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonChannel {}
