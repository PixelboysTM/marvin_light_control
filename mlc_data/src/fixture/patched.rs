use feature::FixtureFeature;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::D3Vec;

use super::blueprint::FixtureBlueprint;

pub mod feature;

pub type PatchedFixtureId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchedFixture {
    pub id: PatchedFixtureId,
    pub identifier: String,
    pub config: FixtureBlueprint,
    pub mode_index: usize,
    pub features: Vec<FixtureFeature>,
    pub matrix_features: Option<FeatureMatrix>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMatrix {
    pixels: D3Vec<Vec<FixtureFeature>>,
}
