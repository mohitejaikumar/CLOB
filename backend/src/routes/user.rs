use actix_web::{web, HttpResponse};
use redis::Value;
use serde_json::{from_str, to_string};

use crate::{app::AppState, db::schema::User, routes::{NewUser, UserRequests}};






#[actix_web::post("/user")]
pub async fn new_user(app_state: web::Data<AppState>) -> HttpResponse {
    let s_db = app_state.scylla_db.lock().unwrap();
    let mut con = &mut app_state.redis_connection.lock().unwrap();
    let sub_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
    let req = to_string(
        &UserRequests::NewUser(NewUser{
            sub_id
        })
    ).unwrap();

    let response = redis::cmd("LPUSH").arg("queues:user").arg(req).query::<Value>(con);
    match response {
        Ok(_) => {
            let mut response_result: Option<String> = None;
            loop {
                let result = redis::cmd("RPOP").arg("queues:user").query::<String>(&mut con);
                if let Ok(response) = result {
                    response_result = Some(response);
                    break;
                }
            }
            let response: String = response_result.unwrap();
            match from_str::<User>(&response) {
                Ok(user) => {
                    let _ = s_db.new_user(user.clone()).await;
                    return HttpResponse::Created().json(user);
                }
                Err(err) => HttpResponse::InternalServerError().json(err.to_string())
            }
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string())
    }
}