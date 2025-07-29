use rust_decimal::Decimal;
use serde::Deserialize;

use crate::enums::asset::Asset;



pub type Symbol = String;
pub type Id = u64;
pub type OrderId = i64;
pub type Quantity = Decimal;
pub type Price = Decimal;


#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize)]
pub struct Exchange {
    pub base: Asset,
    pub quote: Asset,
    pub symbol: String,
}


#[derive(Debug)]
pub enum SymbolError {
    InvalidSymbol,
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
    pub fn from_symbol(symbol: Symbol) -> Result<Exchange, SymbolError> {
        let symbols: Vec<&str> = symbol.split("_").collect();
        let base_str = symbols.get(0).ok_or(SymbolError::InvalidSymbol)?;
        let quote_str = symbols.get(1).ok_or(SymbolError::InvalidSymbol)?;
        let base = Asset::from_str(&base_str).ok_or(SymbolError::InvalidSymbol)?;
        let quote = Asset::from_str(&quote_str).ok_or(SymbolError::InvalidSymbol)?;
        let exchange = Exchange::new(base, quote);
        Ok(exchange)
    }
}

