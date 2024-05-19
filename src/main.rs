mod database;
mod ping;
mod utils;

use std::sync::Arc;

use database::DatabaseModel;
use ping::{Pinger, PingerManager};
use rocket::serde::json::Json;
use rocket::State;
use rocket::{futures::lock::Mutex, http::Status};
use utils::{json_response, JsonResponse};

#[macro_use]
extern crate rocket;

#[post("/", data = "<data>")]
async fn create_monitor<'a>(
    data: Json<uptime_rs::CreateMonitor>,
    _manager: &State<PingerManager>,
) -> JsonResponse<'a> {
    let pool = database::initialize().await;
    let monitor = database::Monitor {
        protocol: ping::Protocol::HTTP,
        id: utils::gen_id(),
        name: data.name.clone(),
        ip: data.ip.clone(),
        port: Some(data.port),
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

async fn test_route(manager: &State<PingerManager>) -> JsonResponse {
    let thingy_mebob = Pinger::new(
        database::Monitor {
            protocol: ping::Protocol::HTTP,
            id: utils::gen_id(),
            name: "Test".to_string(),
            ip: "".into(),
            port: Some(80),
        },
        5,
        || {},
    );
    let a = manager.add_pinger(thingy_mebob);
    json_response(Status::Ok, Some("".to_string()))
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
    let monitor_pool = Arc::new(Mutex::new(ping::PingerManager::new()));
    let monitors = database::Monitor::all(&pool).await;
    pool.close().await;

    for monitor in monitors {
        let pinger = ping::Pinger::new(monitor, 3, || {});
        monitor_pool.lock().await.add_pinger(pinger);
    }

    let manager = monitor_pool.clone();
    tokio::spawn(async move {
        manager.lock().await.start().await;
    });

    rocket::build()
        .manage(monitor_pool)
        .mount("/monitor", routes![get_monitor, create_monitor])
}
