use serde::{Deserialize, Serialize};

use crate::db::schema::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUserResponse {
    pub user: User,
    pub token: String,
}
