mod database;
mod ping;
mod utils;

use askama::Template;
use database::DatabaseModel;
use ping::PingerManager;
use rocket::fs::FileServer;
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
        port: data.port,
    };

    let response = match monitor.create(&pool).await {
        Ok(result) => serde_response(Status::Created, serde_json::to_string(&result)),
        Err(_) => json_response(Status::InternalServerError, None),
    };

    let interval = monitor.interval.clone();
    manager
        .add_pinger(ping::Pinger::new(monitor, interval, || {}))
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

#[derive(Template)]
#[template(path = "views/index.html")]
struct IndexTemplate<'a> {
    title: &'a str,
    monitors: Vec<database::Monitor>,
}

#[derive(Template)]
#[template(path = "views/monitor.html")]
struct MonitorViewTemplate<'a> {
    title: &'a str,
    monitor: database::Monitor,
    uptime_graph: Option<Vec<database::MonitorPing>>,
    average_response_time: Option<i64>,
    last_response_time: Option<i64>,
}

#[derive(Template)]
#[template(path = "components/monitor_list.html")]
struct MonitorListComponentTemplate {
    monitors: Vec<database::Monitor>,
}

#[get("/")]
async fn monitor_list<'a>() -> utils::TemplateResponse<'a> {
    let pool = database::initialize().await;
    let monitors = database::Monitor::all(&pool).await;
    let view = MonitorListComponentTemplate { monitors };
    let html = view.render().unwrap();
    pool.close().await;
    utils::template_response(Status::Ok, html)
}

#[derive(Template)]
#[template(path = "components/uptime_graph.html")]
struct UptimeGraphTemplate {
    uptime_graph: Option<Vec<database::MonitorPing>>,
    average_response_time: Option<i64>,
    last_response_time: Option<i64>,
    monitor: database::Monitor,
}

#[get("/<id>/uptime-graph")]
async fn uptime_graph<'a>(id: i64) -> utils::TemplateResponse<'a> {
    let pool = database::initialize().await;
    let uptime_data = database::MonitorPing::last_n(&pool, id, 30).await;
    let average_response_time = uptime_data
        .iter()
        .fold(0, |acc, ping| acc + ping.duration_ms)
        / uptime_data.len() as i64;
    let last_response_time = uptime_data.last().unwrap().duration_ms;

    let view = UptimeGraphTemplate {
        uptime_graph: Some(uptime_data),
        average_response_time: Some(average_response_time),
        last_response_time: Some(last_response_time),
        monitor: database::Monitor::by_id(id, &pool).await.unwrap(),
    };
    let html = view.render().unwrap();
    pool.close().await;
    utils::template_response(Status::Ok, html)
}

#[get("/<id>")]
async fn monitor_view<'a>(id: i64) -> utils::TemplateResponse<'a> {
    let pool = database::initialize().await;
    let monitor = database::Monitor::by_id(id, &pool).await;

    let response = match monitor {
        Some(monitor) => {
            let uptime_data = database::MonitorPing::last_n(&pool, id, 30).await;
            // Divide by zero bug here, fix later!!
            let average_response_time = uptime_data
                .iter()
                .fold(0, |acc, ping| acc + ping.duration_ms)
                / uptime_data.len() as i64;
            let last_response_time = uptime_data.last().unwrap().duration_ms;

            let view = MonitorViewTemplate {
                title: "Monitor",
                monitor,
                uptime_graph: Some(uptime_data),
                average_response_time: Some(average_response_time),
                last_response_time: Some(last_response_time),
            };
            let html = view.render().unwrap();
            utils::template_response(Status::Ok, html)
        }
        None => utils::template_response(Status::NotFound, String::from("Not found")),
    };
    pool.close().await;
    response
}

#[get("/")]
async fn index<'a>() -> utils::TemplateResponse<'a> {
    let pool = database::initialize().await;
    let monitors = database::Monitor::all(&pool).await;
    pool.close().await;

    let hello = IndexTemplate {
        title: "world",
        monitors,
    };
    let html = hello.render().unwrap();

    utils::template_response(Status::Ok, html)
}

#[get("/")]
async fn all_monitors<'a>() -> JsonResponse<'a> {
    let pool = database::initialize().await;
    let monitors = database::Monitor::all(&pool).await;
    pool.close().await;

    serde_response(Status::Ok, serde_json::to_string(&monitors))
}

#[get("/<id>/ping/last/<n>")]
async fn last_pings<'a>(n: i64, id: i64) -> JsonResponse<'a> {
    let pool = database::initialize().await;
    let pings = database::MonitorPing::last_n(&pool, id, n).await;
    pool.close().await;

    serde_response(Status::Ok, serde_json::to_string(&pings))
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
        .mount("/", routes![index])
        .mount("/monitors", routes![monitor_list])
        .mount("/monitor", routes![monitor_view, uptime_graph])
        .mount(
            "/api/monitor",
            routes![get_monitor, create_monitor, last_pings],
        )
        .mount("/api/monitors", routes![all_monitors])
        .mount("/public", FileServer::from("./static"))
        .manage(monitor_pool)
}

fn vec_last<'a, T>(vec: &'a Vec<T>) -> Option<&'a T> {
    vec.last()
}
