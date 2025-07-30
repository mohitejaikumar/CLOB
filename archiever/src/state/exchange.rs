use serde::{Deserialize, Serialize};
use crate::state::{asset::Asset, Symbol};
use std::str::FromStr;





#[derive(Debug, Serialize, Deserialize)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
    pub symbol: String,
}
impl Exchange {
    pub fn new(base: Asset, quote: Asset) -> Exchange {
        let base_string = base.to_string();
        let quote_string = quote.to_string();
        let symbol = format!("{}_{}", base_string, quote_string);
        Exchange {
            base,
            quote,
            symbol,
        }
    }
    pub fn from_symbol(symbol: Symbol) -> Exchange {
        let symbols: Vec<&str> = symbol.split("_").collect();
        let base_str = symbols.get(0).unwrap();
        let quote_str = symbols.get(1).unwrap();
        let base = Asset::from_str(&base_str).expect("Incorrect symbol");
        let quote = Asset::from_str(&quote_str).expect("Incorrect symbol");
        Exchange::new(base, quote)
    }
}