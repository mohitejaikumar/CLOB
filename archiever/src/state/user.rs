use std::{collections::HashMap, error::Error, hash::Hash};
use rust_decimal::Decimal;
use scylla::{DeserializeRow, SerializeRow};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use crate::{db::ScyllaDb, state::{asset::Asset, scylla_state::ScyllaUser, Quantity}};




#[derive(Debug)]
pub enum UserError {
    OverWithdrawl,
    AssetNotFound,
    UserNotFound,
}




#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub balance: HashMap<Asset, Quantity>,
    pub locked_balance: HashMap<Asset, Quantity>,
}

impl ScyllaUser {
    fn from_scylla_user(&self) -> User {
        let mut balance_map: HashMap<Asset, Quantity> = HashMap::new();
        for (asset_str, balance) in &self.balance {
            let asset = Asset::from_str(&asset_str).unwrap();
            let balance = Decimal::from_str(&balance).unwrap();
            balance_map.insert(asset, balance);
        }
        let mut locked_balance_map: HashMap<Asset, Quantity> = HashMap::new();
        for (asset_str, locked_balance) in &self.locked_balance {
            let asset = Asset::from_str(&asset_str).unwrap();
            let locked_balance = Decimal::from_str(&locked_balance).unwrap();
            locked_balance_map.insert(asset, locked_balance);
        }

        User {
            id: self.id,
            balance: balance_map,
            locked_balance: locked_balance_map,
        }
    }
}



#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostUsers {
    pub user: User,
    pub client: User,
}


impl User {
    pub fn to_scylla_user(&self) -> ScyllaUser {
        let mut scylla_balance: HashMap<String, String> = HashMap::new();
        for (asset, balance) in &self.balance {
            scylla_balance.insert(asset.to_string(), balance.to_string());
        }
        let mut scylla_locked_balance: HashMap<String, String> = HashMap::new();
        for (asset, balance) in &self.locked_balance {
            scylla_locked_balance.insert(asset.to_string(), balance.to_string());
        }
        ScyllaUser {
            id: self.id,
            balance: scylla_balance,
            locked_balance: scylla_locked_balance,
        }
    }

    pub fn deposit(&mut self, asset: &Asset, quantity: Quantity) {
        let assets_balance = self.balance.get_mut(asset);
        match assets_balance {
            None => {
                self.balance.insert(asset.clone(), quantity);
            }
            Some(mut balance) => {
                balance += quantity;
            }
        }
    }

    pub fn unlock_amount(&mut self, asset: &Asset, quantity: Quantity) {
        let mut locked_balance = self.locked_balance.get_mut(asset).unwrap();
        locked_balance -= quantity;
    }

    pub fn withdraw(&mut self, asset: &Asset, quantity: Quantity) -> Result<(), UserError> {
        let mut assets_balance = self.balance.get_mut(asset).ok_or(UserError::AssetNotFound)?;
        if quantity > *assets_balance {
            return Err(UserError::OverWithdrawl);
        }
        assets_balance -= quantity;
        Ok(())
    }
}


impl ScyllaDb {
    pub async fn get_user(&self, user_id: i64) -> Result<User, Box<dyn Error>> {
        let s =
            r#"
            SELECT
                id,
                balance,
                locked_balance
            FROM keyspace_1.user_table
            WHERE id = ? ;
        "#;
        let res = self.session.query_unpaged(s, (user_id,)).await?;
        let temp = res.into_rows_result().unwrap();
        let mut users = temp.rows::<ScyllaUser>().unwrap();
        let scylla_user = users
            .next()
            .transpose()?
            .ok_or(format!("User does not exist in db: {}", user_id))?;
        let user = scylla_user.from_scylla_user();
        Ok(user)
    }
    pub fn update_user_statement(&self) -> &str {
        let s =
            r#"
            UPDATE keyspace_1.user_table 
            SET
                balance = ?,
                locked_balance = ?
            WHERE id = ?;
        "#;
        s
    }
}
