use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    bounded::{
        BoundedValue,
        bounds::{DynamicUSize, One},
    },
    fixture::patched::{PatchedFixture, PatchedFixtureId},
};

pub const UNIVERSE_SIZE: usize = 512;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureUniverse {
    #[serde_as(as = "[_; UNIVERSE_SIZE]")]
    pub addresses: [UniverseSlot; UNIVERSE_SIZE],
    pub fixtures: HashMap<PatchedFixtureId, PatchedFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UniverseSlot {
    Unused,
    Consecutive,
    Fixture(PatchedFixtureId),
}

pub type UniverseId = u16;
pub type UniverseAddress = BoundedValue<usize, One, DynamicUSize<{ UNIVERSE_SIZE }>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FixtureAddress {
    universe: UniverseId,
    address: UniverseAddress,
}

impl FixtureAddress {
    pub fn new(universe: UniverseId, address: UniverseAddress) -> Self {
        Self { universe, address }
    }
    pub fn universe(&self) -> UniverseId {
        self.universe
    }
    pub fn address(&self) -> UniverseAddress {
        self.address
    }
}

impl Display for FixtureAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:0<3}", self.universe, self.address)
    }
}

impl FromStr for FixtureAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.split('.').collect::<Vec<_>>();
        match s.as_slice() {
            [u, a] => {
                let u = u.parse::<u16>().map_err(|e| e.to_string())?;
                let a = a.parse::<usize>().map_err(|e| e.to_string())?;
                let a = UniverseAddress::create(a);
                Ok(FixtureAddress::new(u, a))
            }
            _ => Err("String not in the form '<universe>.<address>'".to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::FixtureAddress;

    #[test]
    fn fixture_address_display() {
        let a: FixtureAddress = "0.126".parse().unwrap();
        assert_eq!(format!("{a}"), "0.126");

        let _b: FixtureAddress = "12.512".parse().unwrap();
    }
}
