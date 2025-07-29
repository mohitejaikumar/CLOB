use enum_stringify::EnumStringify;
use serde::Deserialize;
use strum_macros::EnumIter;
use strum::IntoEnumIterator;




#[derive(Debug, Clone, Deserialize, EnumIter, EnumStringify)]
pub enum OrderStatus {
    InProgress,
    Filled,
    PartiallyFilled,
    Cancelled
}

impl OrderStatus {
    pub fn from_str(asset_to_match: &str) -> Result<Self, ()> {
        for asset in OrderStatus::iter() {
            let current_asset = asset.to_string();
            if asset_to_match.to_string() == current_asset {
                return Ok(asset);
            }
        }
        Err(())
    }
}