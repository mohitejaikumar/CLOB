
use enum_stringify::EnumStringify;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize, EnumStringify)]
pub enum Asset{
    USDT,
    BTC,
    SOL,
    ETH
}

impl Asset {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in Asset::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}