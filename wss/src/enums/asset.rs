use enum_stringify::EnumStringify;
use serde::Deserialize;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, Deserialize, EnumStringify)]
pub enum Asset {
    USDT,
    BTC,
    SOL,
    ETH,
}

impl Asset {
    pub fn from_str(asset_to_match: &str) -> Option<Self> {
        for asset in Asset::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Some(asset);
            }
        }
        None
    }
}