mod database;
mod ping;
mod utils;

use database::DatabaseModel;
use ping::PingerManager;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use utils::{json_response, serde_response, JsonResponse};

#[macro_use]
extern crate rocket;

#[post("/", data = "<data>")]
async fn create_monitor<'a>(
    data: Json<uptime_rs::CreateMonitor>,
    manager: &State<PingerManager>,
) -> JsonResponse<'a> {
    let pool = database::initialize().await;
    let monitor = database::Monitor {
        interval: data.interval,
        protocol: ping::Protocol::HTTP,
        id: utils::gen_id(),
        name: data.name.clone(),
        ip: data.ip.clone(),
        port: Some(data.port),
    };

    let response = match monitor.create(&pool).await {
        Ok(result) => serde_response(Status::Created, serde_json::to_string(&result)),
        Err(_) => json_response(Status::InternalServerError, None),
    };

    manager
        .add_pinger(ping::Pinger::new(
            monitor,
            u64::from(monitor.interval),
            || {},
        ))
        .await;

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
            let serialized = serde_json::to_string(&monitor);
            match serialized {
                Ok(json) => json_response(Status::Ok, Some(json)),
                Err(_) => json_response(Status::InternalServerError, None),
            }
        }
        None => json_response(Status::NotFound, None),
    }
}

#[launch]
async fn rocket() -> _ {
    let db = database::initialize().await;
    let mut monitor_pool = ping::PingerManager::new();

    for monitor in database::Monitor::all(&db).await {
        let pinger = ping::Pinger::new(monitor, 3, || {});
        monitor_pool.add_pinger(pinger).await;
    }

    db.close().await;
    monitor_pool.start().await;

    rocket::build()
        .mount("/monitor", routes![get_monitor, create_monitor])
        .manage(monitor_pool)
}
