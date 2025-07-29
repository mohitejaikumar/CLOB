use enum_stringify::EnumStringify;
use serde::Deserialize;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;


#[derive(Deserialize, PartialEq, Eq, Hash, EnumIter, EnumStringify, Clone)]
pub enum RegisteredSymbols {
    SOL_USDT,
    BTC_USDT,
    ETH_USDT,
}

impl RegisteredSymbols {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in RegisteredSymbols::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}