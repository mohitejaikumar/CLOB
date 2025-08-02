use actix_web::{
    HttpResponse,
    web::{self, Data, Json},
};
use redis::Value;
use serde_json::{from_str, to_string};

use crate::{
    app::AppState,
    db::schema::User,
    middleware::{Claims, generate_jwt},
    routes::{
        Deposit, GetUserBalances, NewUser, UserId, UserRequests, Withdraw,
        request_output::NewUserResponse,
    },
};

#[actix_web::post("/user")]
pub async fn new_user(app_state: web::Data<AppState>) -> HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let req = to_string(&UserRequests::NewUser(NewUser { sub_id })).unwrap();

    let response = redis::cmd("LPUSH")
        .arg("queues:user")
        .arg(req)
        .query::<Value>(con);
    match response {
        Ok(_) => {
            let mut response_result: Option<String> = None;
            loop {
                let result = redis::cmd("RPOP").arg(sub_id).query::<String>(&mut con);
                if let Ok(response) = result {
                    response_result = Some(response);
                    break;
                }
            }
            let response: String = response_result.unwrap();
            match from_str::<User>(&response) {
                Ok(user) => {
                    let _ = s_db.new_user(user.clone()).await;

                    match generate_jwt(user.id) {
                        Ok(token) => HttpResponse::Created().json(NewUserResponse {
                            user: user,
                            token: token,
                        }),
                        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
                    }
                }
                Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[actix_web::get("")]
pub async fn get_user(claims: Claims, app_state: Data<AppState>) -> actix_web::HttpResponse {
    // lock the redis connection
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let user = from_str::<UserId>(&claims.sub).unwrap();
    let query = GetUserBalances {
        user_id: user.user_id,
        sub_id: sub_id,
    };
    let req = to_string(&UserRequests::GetUserBalances(query)).unwrap();
    let response = redis::cmd("LPUSH")
        .arg("queues:user")
        .arg(req)
        .query::<Value>(&mut con);

    match response {
        Ok(_) => {
            let mut response_result: Option<String> = None;
            loop {
                let result = redis::cmd("RPOP").arg(sub_id).query::<String>(&mut con);
                if let Ok(response) = result {
                    response_result = Some(response);
                    break;
                }
            }
            let response: String = response_result.unwrap();
            match from_str::<User>(&response) {
                Ok(user) => {
                    return HttpResponse::Created().json(user);
                }
                Err(err) => HttpResponse::BadRequest().json(err.to_string()),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[actix_web::post("/deposit")]
pub async fn deposit(
    claims: Claims,
    mut body: Json<Deposit>,
    app_state: Data<AppState>,
) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let user = from_str::<UserId>(&claims.sub).unwrap();
    body.user_id = user.user_id;
    body.sub_id = sub_id;
    let req = to_string(&UserRequests::Deposit(body.0)).unwrap();
    let response = redis::cmd("LPUSH")
        .arg("queues:user")
        .arg(req)
        .query::<Value>(con);
    match response {
        Ok(_) => {
            let mut response_result: Option<String> = None;
            loop {
                let result = redis::cmd("RPOP").arg(sub_id).query::<String>(con);
                if let Ok(response) = result {
                    response_result = Some(response);
                    break;
                }
            }
            let response: String = response_result.unwrap();
            match from_str::<User>(&response) {
                Ok(user) => {
                    let _ = s_db.update_user(&mut user.clone()).await;
                    return HttpResponse::Created().json(user);
                }
                Err(err) => HttpResponse::BadRequest().json(err.to_string()),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[actix_web::post("/withdraw")]
pub async fn withdraw(
    claims: Claims,
    mut body: Json<Withdraw>,
    app_state: Data<AppState>,
) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let user = from_str::<UserId>(&claims.sub).unwrap();
    body.user_id = user.user_id;
    body.sub_id = sub_id;
    let req = to_string(&UserRequests::Withdraw(body.0)).unwrap();
    let response = redis::cmd("LPUSH")
        .arg("queues:user")
        .arg(req)
        .query::<Value>(con);
    match response {
        Ok(_) => {
            let mut response_result: Option<String> = None;
            loop {
                let result = redis::cmd("RPOP").arg(sub_id).query::<String>(con);
                if let Ok(response) = result {
                    response_result = Some(response);
                    break;
                }
            }
            let response: String = response_result.unwrap();
            println!("response: {}", response);
            match from_str::<User>(&response) {
                Ok(user) => {
                    let _ = s_db.update_user(&mut user.clone()).await;
                    return HttpResponse::Created().json(user);
                }
                Err(err) => HttpResponse::BadRequest().json(err.to_string()),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[actix_web::get("/history/orders")]
pub async fn order_history(claims: Claims, app_state: Data<AppState>) -> actix_web::HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let user = from_str::<UserId>(&claims.sub).unwrap();
    let result = s_db.get_user(user.user_id).await;
    match result {
        Ok(user) => {
            let user_orders = s_db.get_users_orders(user.id).await;
            match user_orders {
                Ok(orders) => {
                    return HttpResponse::Ok().json(orders);
                }
                Err(err) => HttpResponse::NotFound().json(format!(
                    "No orders found for user: {} {}",
                    user.id,
                    err.to_string()
                )),
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}
