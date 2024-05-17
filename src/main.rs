mod database;
mod ping;
mod utils;

use database::DatabaseModel;
use rocket::http::Status;
use rocket::serde::json::Json;
use utils::{json_response, JsonResponse};

#[macro_use]
extern crate rocket;

#[post("/", data = "<data>")]
async fn create_monitor<'a>(data: Json<uptime_rs::CreateMonitor>) -> JsonResponse<'a> {
    let pool = database::initialize().await;
    let monitor = database::Monitor {
        id: utils::gen_id(),
        name: data.name.clone(),
        ip: data.ip.clone(),
        port: data.port,
    };

    let response = match monitor.create(&pool).await {
        Ok(result) => {
            let json = serde_json::to_string(&result).unwrap();
            json_response(Status::Created, Some(json))
        }
        Err(_) => json_response(Status::InternalServerError, None),
    };

    pool.close().await;
    response
}

#[get("/<id>")]
async fn get_monitor<'a>(id: i64) -> JsonResponse<'a> {
    let pool = database::initialize().await;
    let query_result = database::Monitor::by_id(id, &pool).await;
    pool.close().await;

    match query_result {
        Some(monitor) => {
            let json = serde_json::to_string(&monitor).unwrap();
            json_response(Status::Ok, Some(json))
        }
        None => json_response(Status::NotFound, None),
    }
}

#[launch]
async fn rocket() -> _ {
    let pool = database::initialize().await;
    ping::ticker_manager().await;

    // let test = ping::ping("https://www.google.com").await.unwrap();
    // dbg!(&test);
    // match test.create(&pool).await {
    //     Ok(result) => {
    //         let json = serde_json::to_string(&result).unwrap();
    //         println!("{}", json);
    //     }
    //     Err(_) => {
    //         println!("Failed to create ping.");
    //     }
    // }

    pool.close().await;

    rocket::build().mount("/monitor", routes![get_monitor, create_monitor])
}
