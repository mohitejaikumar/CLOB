use actix_web::{Error, FromRequest, HttpRequest, dev::Payload};
use chrono::{Duration, Utc};
use futures_util::future::{Ready, ready};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::{db::schema::Id, routes::UserId};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if let Some(header_value) = auth_header {
            if let Ok(auth_str) = header_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str.trim_start_matches("Bearer ").trim();
                    let decoding_key = DecodingKey::from_secret("jk2003".as_ref());
                    let validation = Validation::new(Algorithm::HS256);

                    return match decode::<Claims>(token, &decoding_key, &validation) {
                        Ok(data) => ready(Ok(data.claims)),
                        Err(e) => ready(Err(actix_web::error::ErrorUnauthorized("INVALID JWT"))),
                    };
                }
            }
        }

        ready(Err(actix_web::error::ErrorUnauthorized(
            "Missing Authorization Header",
        )))
    }
}

pub fn generate_jwt(user_id: Id) -> Result<String, jsonwebtoken::errors::Error> {
    
    let expiration = Utc::now()
    .checked_add_signed(Duration::hours(24))
    .expect("valid timestamp")
    .timestamp() as usize;
    let user = UserId {
        user_id: user_id
    };
    let claims = Claims {
        sub: to_string(&user).unwrap(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("jk2003".as_ref())
    )
}
